use std::str::FromStr;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use axum_test::{TestServer, TestServerConfig};
use http::{HeaderName, HeaderValue, StatusCode};
use tower_surf::{Surf, Token};

#[tokio::test]
async fn creates_initial_cookie() -> Result<()> {
    let app = Router::new()
        .route("/", get(|| async {}))
        .layer(Surf::new("secret-key"));

    let config = TestServerConfig::builder().save_cookies().build();
    let server = TestServer::new_with_config(app, config)?;

    let cookies = server.get("/").await.cookies();
    let cookie = cookies.get("__HOST-csrf_token");
    assert!(cookie.is_some());

    Ok(())
}

#[tokio::test]
async fn updates_cookie() -> Result<()> {
    async fn set_token(token: Token) -> Result<StatusCode, StatusCode> {
        token
            .set("unique-session-id")
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(StatusCode::OK)
    }

    let app = Router::new()
        .route("/", get(|| async {}))
        .route("/a", get(set_token))
        .layer(Surf::new("secret-key"));

    let config = TestServerConfig::builder().save_cookies().build();
    let server = TestServer::new_with_config(app, config)?;

    server.get("/").await;

    let cookies = server.get("/a").await.cookies();
    let cookie = cookies.get("__HOST-csrf_token").expect("cookie not found.");
    assert!(cookie.value().contains("unique-session-id"));

    Ok(())
}

#[tokio::test]
async fn resets_cookie() -> Result<()> {
    async fn set_token(token: Token) -> Result<StatusCode, StatusCode> {
        token
            .set("unique-session-id")
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(StatusCode::OK)
    }

    async fn reset_token(token: Token) -> Result<StatusCode, StatusCode> {
        token.reset();

        Ok(StatusCode::OK)
    }

    let app = Router::new()
        .route("/", get(|| async {}))
        .route("/a", get(set_token))
        .route("/b", get(reset_token))
        .layer(Surf::new("secret-key"));

    let config = TestServerConfig::builder().save_cookies().build();
    let server = TestServer::new_with_config(app, config)?;

    server.get("/").await;
    server.get("/a").await;

    let cookies = server.get("/b").await.cookies();
    let cookie = cookies.get("__HOST-csrf_token").expect("cookie not found.");
    assert!(!cookie.value().contains("unique-session-id"));

    Ok(())
}

#[tokio::test]
async fn guards_mutation() -> Result<()> {
    let app = Router::new()
        .route("/", get(|| async {}))
        .route("/", post(|| async {}))
        .layer(Surf::new("secret-key"));

    let config = TestServerConfig::builder().save_cookies().build();
    let mut server = TestServer::new_with_config(app, config)?;

    let cookies = server.get("/").await.cookies();
    let cookie = cookies.get("__HOST-csrf_token").expect("cookie not found.");

    // Correct token sent.

    server.add_header(
        HeaderName::from_str("X-CSRF-Token").expect("couldn't create HeaderName"),
        HeaderValue::from_str(cookie.value()).expect("couldn't create HeaderValue"),
    );

    server.post("/").await.assert_status_ok();

    // Incorrect token sent.

    server.clear_headers();

    server.add_header(
        HeaderName::from_str("X-CSRF-Token").expect("couldn't create HeaderName"),
        HeaderValue::from_str("oh howdy doody").expect("couldn't create HeaderValue"),
    );

    server.post("/").await.assert_status_forbidden();

    // No token sent.

    server.clear_headers();

    server.post("/").await.assert_status_forbidden();

    Ok(())
}
