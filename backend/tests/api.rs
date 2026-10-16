use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn session_browsing_validation_and_disconnect() {
    let app = webdb::router(false);
    let unauthorized = app
        .clone()
        .oneshot(Request::get("/api/tables").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
    let connection = r#"{"provider":"demo","host":"","port":5432,"database":"","username":"","password":"","ssl":true}"#;
    let connected = app
        .clone()
        .oneshot(
            Request::post("/api/session")
                .header("content-type", "application/json")
                .body(Body::from(connection))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(connected.status(), StatusCode::OK);
    let cookie = connected.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();
    assert!(
        connected.headers()["set-cookie"]
            .to_str()
            .unwrap()
            .contains("HttpOnly; SameSite=Strict")
    );
    for (path, status) in [
        ("/api/tables", StatusCode::OK),
        (
            "/api/tables?database=outside-session",
            StatusCode::BAD_REQUEST,
        ),
        (
            "/api/rows?schema=public&table=customers&pageSize=5",
            StatusCode::OK,
        ),
        (
            "/api/rows?schema=public&table=customers&pageSize=101",
            StatusCode::BAD_REQUEST,
        ),
        (
            "/api/rows?schema=public&table=customers&sort=missing",
            StatusCode::BAD_REQUEST,
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::get(path)
                    .header("cookie", &cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), status, "{path}");
        if path.contains("pageSize=5") {
            let body = response.into_body().collect().await.unwrap().to_bytes();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(json["rows"].as_array().unwrap().len(), 5);
            assert_eq!(json["hasMore"], true);
        }
    }
    for (sql, page, page_size, status, expected_rows, has_more) in [
        (
            "SELECT * FROM \"public\".\"customers\";",
            0,
            5,
            StatusCode::OK,
            5,
            true,
        ),
        (
            "select * from public.orders LIMIT 7",
            1,
            5,
            StatusCode::OK,
            2,
            false,
        ),
        (
            "SELECT * FROM products LIMIT 0",
            0,
            5,
            StatusCode::OK,
            0,
            false,
        ),
        (
            "SELECT * FROM products LIMIT 7",
            2,
            5,
            StatusCode::OK,
            0,
            false,
        ),
        ("", 0, 5, StatusCode::BAD_REQUEST, 0, false),
        (
            "SELECT * FROM customers",
            0,
            101,
            StatusCode::BAD_REQUEST,
            0,
            false,
        ),
        (
            "DELETE FROM customers",
            0,
            5,
            StatusCode::BAD_REQUEST,
            0,
            false,
        ),
        (
            "SELECT * FROM customers; DELETE FROM customers",
            0,
            5,
            StatusCode::BAD_REQUEST,
            0,
            false,
        ),
        (
            "SELECT name FROM customers",
            0,
            5,
            StatusCode::BAD_REQUEST,
            0,
            false,
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::post("/api/query")
                    .header("cookie", &cookie)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({"sql": sql, "page": page, "pageSize": page_size})
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), status, "{sql}");
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        if status == StatusCode::OK {
            assert_eq!(json["rows"].as_array().unwrap().len(), expected_rows);
            assert_eq!(json["hasMore"], has_more);
            assert_eq!(json["page"], page);
        } else {
            assert!(json["message"].is_string());
        }
    }
    let unauthorized_query = app
        .clone()
        .oneshot(
            Request::post("/api/query")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"sql":"SELECT * FROM customers"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauthorized_query.status(), StatusCode::UNAUTHORIZED);
    let response = app
        .clone()
        .oneshot(
            Request::delete("/api/session")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let response = app
        .oneshot(
            Request::get("/api/tables")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
