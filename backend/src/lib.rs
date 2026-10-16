pub mod adapters;
pub mod application;
pub mod domain;
mod sessions;

use application::*;
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use domain::*;
use sessions::SessionStore;
use std::sync::Arc;

struct AppState {
    database: DatabaseService,
    sessions: SessionStore,
    secure_cookies: bool,
    operations: tokio::sync::Semaphore,
}
type Shared = Arc<AppState>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            ErrorKind::InvalidInput => StatusCode::BAD_REQUEST,
            ErrorKind::NotConnected => StatusCode::UNAUTHORIZED,
            ErrorKind::NotFound => StatusCode::NOT_FOUND,
            ErrorKind::Busy => StatusCode::TOO_MANY_REQUESTS,
            ErrorKind::Upstream => StatusCode::BAD_GATEWAY,
            ErrorKind::Timeout => StatusCode::GATEWAY_TIMEOUT,
            ErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(serde_json::json!({ "message": self.1 }))).into_response()
    }
}
fn session_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("cookie")?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|part| part.trim().strip_prefix("webdb_session="))
}

pub fn router(secure_cookies: bool) -> Router {
    let state = Arc::new(AppState {
        database: DatabaseService::new(vec![
            Arc::new(adapters::postgres::PostgresAdapter),
            Arc::new(adapters::demo::DemoAdapter),
        ]),
        sessions: SessionStore::default(),
        secure_cookies,
        operations: tokio::sync::Semaphore::new(16),
    });
    let weak_state = Arc::downgrade(&state);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let Some(state) = weak_state.upgrade() else {
                break;
            };
            state.sessions.purge_expired();
        }
    });
    Router::new()
        .route(
            "/api/health",
            get(|| async { Json(serde_json::json!({"status":"ok"})) }),
        )
        .route("/api/providers", get(providers))
        .route(
            "/api/session",
            post(connect).get(current_session).delete(disconnect),
        )
        .route("/api/tables", get(tables))
        .route("/api/rows", get(rows))
        .layer(DefaultBodyLimit::max(16 * 1024))
        .layer(middleware::from_fn(security_headers))
        .with_state(state)
}
async fn security_headers(request: axum::extract::Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    response.headers_mut().insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    response
}
async fn providers(State(state): State<Shared>) -> Json<Vec<Provider>> {
    Json(state.database.providers())
}

fn session_info(input: &ConnectionInput) -> serde_json::Value {
    serde_json::json!({ "provider": input.provider, "database": if input.provider == "demo" { "sample_workspace" } else { &input.database }, "host": if input.provider == "demo" { "Sample data" } else { &input.host }, "expiresIn": sessions::TTL_SECONDS })
}
async fn current_session(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(session_info(
        &state.sessions.get(session_token(&headers))?,
    )))
}

fn cookie(state: &AppState, id: &str, max_age: u64) -> HeaderValue {
    HeaderValue::from_str(&format!(
        "webdb_session={id}; HttpOnly; SameSite=Strict; Path=/api; Max-Age={max_age}{}",
        if state.secure_cookies { "; Secure" } else { "" }
    ))
    .expect("generated cookie is valid")
}
async fn connect(
    State(state): State<Shared>,
    headers: HeaderMap,
    Json(input): Json<ConnectionInput>,
) -> AppResult<(HeaderMap, Json<serde_json::Value>)> {
    let _permit = state
        .operations
        .try_acquire()
        .map_err(|_| AppError(ErrorKind::Busy, "Too many requests. Try again shortly."))?;
    input.validate().map_err(invalid)?;
    let adapter = state.database.resolve(&input.provider)?;
    adapter.test(&input).await?;
    let info = session_info(&input);
    let id = state.sessions.insert(input, session_token(&headers))?;
    let mut headers = HeaderMap::new();
    headers.insert("set-cookie", cookie(&state, &id, sessions::TTL_SECONDS));
    Ok((headers, Json(info)))
}
async fn disconnect(State(state): State<Shared>, headers: HeaderMap) -> (HeaderMap, StatusCode) {
    state.sessions.remove(session_token(&headers));
    let mut headers = HeaderMap::new();
    headers.insert("set-cookie", cookie(&state, "", 0));
    (headers, StatusCode::NO_CONTENT)
}
async fn tables(State(state): State<Shared>, headers: HeaderMap) -> AppResult<Json<Vec<Table>>> {
    let _permit = state
        .operations
        .try_acquire()
        .map_err(|_| AppError(ErrorKind::Busy, "Too many requests. Try again shortly."))?;
    let input = state.sessions.get(session_token(&headers))?;
    Ok(Json(
        state
            .database
            .resolve(&input.provider)?
            .tables(&input)
            .await?,
    ))
}
async fn rows(
    State(state): State<Shared>,
    headers: HeaderMap,
    Query(query): Query<BrowseQuery>,
) -> AppResult<Json<TablePage>> {
    let _permit = state
        .operations
        .try_acquire()
        .map_err(|_| AppError(ErrorKind::Busy, "Too many requests. Try again shortly."))?;
    query.bounds().map_err(invalid)?;
    let input = state.sessions.get(session_token(&headers))?;
    Ok(Json(
        state
            .database
            .resolve(&input.provider)?
            .rows(&input, &query)
            .await?,
    ))
}
