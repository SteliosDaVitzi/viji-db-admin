pub mod adapters;
pub mod application;
pub mod domain;
mod sessions;

use application::*;
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, Query, State},
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
    connections: Option<Arc<dyn ConnectionRepository>>,
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
    router_with_repository(secure_cookies, None)
}
pub fn router_with_repository(
    secure_cookies: bool,
    connections: Option<Arc<dyn ConnectionRepository>>,
) -> Router {
    router_with_services(
        secure_cookies,
        connections,
        DatabaseService::new(vec![
            Arc::new(adapters::postgres::PostgresAdapter),
            Arc::new(adapters::demo::DemoAdapter),
        ]),
    )
}
fn router_with_services(
    secure_cookies: bool,
    connections: Option<Arc<dyn ConnectionRepository>>,
    database: DatabaseService,
) -> Router {
    let state = Arc::new(AppState {
        database,
        sessions: SessionStore::default(),
        secure_cookies,
        operations: tokio::sync::Semaphore::new(16),
        connections,
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
        .route("/api/databases", get(databases).post(select_database))
        .route(
            "/api/connections",
            get(saved_connections).post(save_connection),
        )
        .route(
            "/api/connections/{id}",
            axum::routing::delete(delete_connection),
        )
        .route("/api/connections/{id}/connect", post(connect_saved))
        .route("/api/rows", get(rows))
        .route("/api/query", post(query))
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
    serde_json::json!({ "provider": input.provider, "database": if input.provider == "demo" { "sample_workspace" } else { &input.database }, "allDatabases": input.all_databases, "host": if input.provider == "demo" { "Sample data" } else { &input.host }, "expiresIn": sessions::TTL_SECONDS })
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
    Json(mut input): Json<ConnectionInput>,
) -> AppResult<(HeaderMap, Json<serde_json::Value>)> {
    let _permit = state
        .operations
        .try_acquire()
        .map_err(|_| AppError(ErrorKind::Busy, "Too many requests. Try again shortly."))?;
    input.validate().map_err(invalid)?;
    if input.all_databases && input.provider != "postgres" {
        return Err(invalid(
            "Instance browsing is unavailable for this provider.",
        ));
    }
    if input.all_databases {
        input.database = "postgres".into();
    }
    let adapter = state.database.resolve(&input.provider)?;
    adapter.test(&input).await?;
    let info = session_info(&input);
    let id = state.sessions.insert(input, session_token(&headers))?;
    let mut headers = HeaderMap::new();
    headers.insert("set-cookie", cookie(&state, &id, sessions::TTL_SECONDS));
    Ok((headers, Json(info)))
}

async fn databases(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<String>>> {
    let _permit = state
        .operations
        .try_acquire()
        .map_err(|_| AppError(ErrorKind::Busy, "Too many requests. Try again shortly."))?;
    let input = state.sessions.get(session_token(&headers))?;
    if !input.all_databases {
        return Ok(Json(vec![input.database]));
    }
    Ok(Json(
        state
            .database
            .resolve(&input.provider)?
            .databases(&input)
            .await?,
    ))
}
async fn select_database(
    State(state): State<Shared>,
    headers: HeaderMap,
    Json(selection): Json<SelectDatabase>,
) -> AppResult<(HeaderMap, Json<serde_json::Value>)> {
    let _permit = state
        .operations
        .try_acquire()
        .map_err(|_| AppError(ErrorKind::Busy, "Too many requests. Try again shortly."))?;
    let mut input = state.sessions.get(session_token(&headers))?;
    if !input.all_databases {
        return Err(invalid("Connect in instance mode to switch databases."));
    }
    let adapter = state.database.resolve(&input.provider)?;
    if !adapter
        .databases(&input)
        .await?
        .contains(&selection.database)
    {
        return Err(invalid("Choose an accessible database from this instance."));
    }
    input.database = selection.database;
    adapter.test(&input).await?;
    let info = session_info(&input);
    let id = state.sessions.insert(input, session_token(&headers))?;
    let mut headers = HeaderMap::new();
    headers.insert("set-cookie", cookie(&state, &id, sessions::TTL_SECONDS));
    Ok((headers, Json(info)))
}
fn repository(state: &AppState) -> AppResult<&dyn ConnectionRepository> {
    state.connections.as_deref().ok_or_else(|| {
        invalid("Saving connections requires WEBDB_VAULT_KEY and WEBDB_STORE_PATH on the backend.")
    })
}
async fn saved_connections(State(state): State<Shared>) -> AppResult<Json<serde_json::Value>> {
    let items = match &state.connections {
        Some(store) => store.list().await?,
        None => vec![],
    };
    Ok(Json(
        serde_json::json!({"enabled": state.connections.is_some(), "connections": items}),
    ))
}
async fn save_connection(
    State(state): State<Shared>,
    headers: HeaderMap,
    Json(request): Json<SaveConnection>,
) -> AppResult<StatusCode> {
    let input = state.sessions.get(session_token(&headers))?;
    let name = request.name.trim();
    if name.is_empty()
        || name.len() > 128
        || name.chars().any(char::is_control)
        || input.provider == "demo"
    {
        return Err(invalid(
            "Enter a connection name of at most 128 characters for a real database.",
        ));
    }
    repository(&state)?.save(name, &input).await?;
    Ok(StatusCode::CREATED)
}
async fn delete_connection(
    State(state): State<Shared>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    repository(&state)?.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn connect_saved(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> AppResult<(HeaderMap, Json<serde_json::Value>)> {
    let input = repository(&state)?.load(&id).await?;
    connect(State(state), headers, Json(input)).await
}
async fn disconnect(State(state): State<Shared>, headers: HeaderMap) -> (HeaderMap, StatusCode) {
    state.sessions.remove(session_token(&headers));
    let mut headers = HeaderMap::new();
    headers.insert("set-cookie", cookie(&state, "", 0));
    (headers, StatusCode::NO_CONTENT)
}
async fn tables(
    State(state): State<Shared>,
    headers: HeaderMap,
    Query(query): Query<DatabaseTablesQuery>,
) -> AppResult<Json<Vec<Table>>> {
    let _permit = state
        .operations
        .try_acquire()
        .map_err(|_| AppError(ErrorKind::Busy, "Too many requests. Try again shortly."))?;
    let mut input = state.sessions.get(session_token(&headers))?;
    if let Some(database) = query.database
        && database != input.database
    {
        if !input.all_databases
            || !state
                .database
                .resolve(&input.provider)?
                .databases(&input)
                .await?
                .contains(&database)
        {
            return Err(invalid("Choose an accessible database from this instance."));
        }
        input.database = database;
    }
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

async fn query(
    State(state): State<Shared>,
    headers: HeaderMap,
    Json(query): Json<SqlQuery>,
) -> AppResult<Json<TablePage>> {
    let _permit = state
        .operations
        .try_acquire()
        .map_err(|_| AppError(ErrorKind::Busy, "Too many requests. Try again shortly."))?;
    let input = state.sessions.get(session_token(&headers))?;
    query.bounds().map_err(invalid)?;
    Ok(Json(
        state
            .database
            .resolve(&input.provider)?
            .query(&input, &query)
            .await?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    struct InstanceAdapter;
    #[async_trait::async_trait]
    impl DatabaseAdapter for InstanceAdapter {
        fn provider(&self) -> Provider {
            Provider {
                id: "postgres",
                name: "Test instance",
            }
        }
        async fn test(&self, input: &ConnectionInput) -> AppResult<()> {
            if input.database == "unavailable" {
                return Err(invalid("Database unavailable."));
            }
            Ok(())
        }
        async fn databases(&self, _: &ConnectionInput) -> AppResult<Vec<String>> {
            Ok(vec![
                "postgres".into(),
                "app\"db".into(),
                "unavailable".into(),
            ])
        }
        async fn tables(&self, input: &ConnectionInput) -> AppResult<Vec<Table>> {
            Ok(vec![Table {
                schema: "public".into(),
                name: input.database.clone(),
                kind: "BASE TABLE".into(),
            }])
        }
        async fn rows(&self, _: &ConnectionInput, _: &BrowseQuery) -> AppResult<TablePage> {
            Err(invalid("Unused"))
        }
        async fn query(&self, _: &ConnectionInput, _: &SqlQuery) -> AppResult<TablePage> {
            Err(invalid("Unused"))
        }
    }
    async fn call(
        app: &Router,
        method: &str,
        path: &str,
        token: &str,
        value: serde_json::Value,
    ) -> (StatusCode, String, serde_json::Value) {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header("cookie", token)
                    .header("content-type", "application/json")
                    .body(Body::from(value.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let cookie = response
            .headers()
            .get("set-cookie")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(token)
            .split(';')
            .next()
            .unwrap()
            .to_owned();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            cookie,
            serde_json::from_slice(&bytes).unwrap_or_default(),
        )
    }
    #[tokio::test]
    async fn instance_switching_and_saved_connections_never_expose_credentials() {
        let vault = Arc::new(
            adapters::vault::SqliteVault::open(":memory:", &[1; 32])
                .await
                .unwrap(),
        );
        let app = router_with_services(
            false,
            Some(vault),
            DatabaseService::new(vec![Arc::new(InstanceAdapter)]),
        );
        let input = serde_json::json!({"provider":"postgres","host":"localhost","port":5432,"database":"","allDatabases":true,"username":"user","password":"private-password","ssl":true});
        let mut single = input.clone();
        single["allDatabases"] = false.into();
        assert_eq!(
            call(&app, "POST", "/api/session", "", single).await.0,
            StatusCode::BAD_REQUEST
        );
        let (status, token, session) = call(&app, "POST", "/api/session", "", input).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(session["database"], "postgres");
        assert_eq!(session["allDatabases"], true);
        assert!(session.get("password").is_none());
        let (status, _, expanded) = call(
            &app,
            "GET",
            "/api/tables?database=app%22db",
            &token,
            serde_json::Value::Null,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(expanded[0]["name"], "app\"db");
        assert_eq!(
            call(&app, "GET", "/api/session", &token, serde_json::Value::Null)
                .await
                .2["database"],
            "postgres",
            "Expanding another database must preserve the active session"
        );
        assert_eq!(
            call(
                &app,
                "GET",
                "/api/tables?database=outside-instance",
                &token,
                serde_json::Value::Null
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            call(
                &app,
                "GET",
                "/api/tables?database=app%22db",
                "",
                serde_json::Value::Null
            )
            .await
            .0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            call(
                &app,
                "GET",
                "/api/databases",
                &token,
                serde_json::Value::Null
            )
            .await
            .2
            .as_array()
            .unwrap()
            .len(),
            3
        );
        for database in ["outside-instance", "unavailable"] {
            assert_eq!(
                call(
                    &app,
                    "POST",
                    "/api/databases",
                    &token,
                    serde_json::json!({"database":database})
                )
                .await
                .0,
                StatusCode::BAD_REQUEST
            );
            assert_eq!(
                call(&app, "GET", "/api/session", &token, serde_json::Value::Null)
                    .await
                    .2["database"],
                "postgres"
            );
        }
        let (status, next, switched) = call(
            &app,
            "POST",
            "/api/databases",
            &token,
            serde_json::json!({"database":"app\"db"}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(switched["database"], "app\"db");
        assert_eq!(
            call(&app, "GET", "/api/tables", &token, serde_json::Value::Null)
                .await
                .0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            call(&app, "GET", "/api/tables", &next, serde_json::Value::Null)
                .await
                .2[0]["name"],
            "app\"db"
        );
        assert_eq!(
            call(
                &app,
                "POST",
                "/api/connections",
                "",
                serde_json::json!({"name":"Work"})
            )
            .await
            .0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            call(
                &app,
                "POST",
                "/api/connections",
                &next,
                serde_json::json!({"name":"Work"})
            )
            .await
            .0,
            StatusCode::CREATED
        );
        let list = call(&app, "GET", "/api/connections", "", serde_json::Value::Null)
            .await
            .2;
        assert!(!list.to_string().contains("private-password"));
        assert_eq!(list["connections"][0].as_object().unwrap().len(), 2);
        let id = list["connections"][0]["id"].as_str().unwrap();
        let (status, _, restored) = call(
            &app,
            "POST",
            &format!("/api/connections/{id}/connect"),
            &next,
            serde_json::Value::Null,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(restored["allDatabases"], true);
        assert!(restored.get("password").is_none());
        assert_eq!(
            call(
                &app,
                "DELETE",
                &format!("/api/connections/{id}"),
                "",
                serde_json::Value::Null
            )
            .await
            .0,
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            call(
                &app,
                "POST",
                &format!("/api/connections/{id}/connect"),
                "",
                serde_json::Value::Null
            )
            .await
            .0,
            StatusCode::NOT_FOUND
        );
    }
}
