use std::net::SocketAddr;

use axum::{
    response::Redirect,
    routing::{get, post},
    Router,
};
use http::StatusCode;
use maud::{html, Markup};
use tower_surf::{Surf, Token};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/submit", post(submit))
        .route("/login", get(login))
        .route("/logout", get(logout))
        .layer(Surf::new("secret-key").secure(false));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn root(token: Token) -> Result<Markup, StatusCode> {
    let token = token.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(html! {
        link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css";
        script src="https://unpkg.com/htmx.org@2.0.2" {}

        main class="container" {
            nav {
                ul {
                    li { a href="/login" { "Login" } }
                    li { a href="/logout" { "Logout" } }
                }
            }

            p { mark { "Open the Network tab in your dev console." } }
            p { small { kbd { (token) } } }

            div class="grid" {
                div {
                    form hx-post="/submit" hx-swap="none" "hx-on::config-request"={"event.detail.headers['X-CSRF-Token'] = \"" (token) "\""} {
                        label for="hotdogs" { "How do you like your hotdogs?" }

                        select name="hotdogs" value="ketchup" {
                            option value="ketchup" { "Ketchup" }
                            option value="ketchup-again" { "Ketchup" }
                            option value="more-ketchup" { "More ketchup" }
                        }

                        button type="submit" { "Submit with token" }
                    }
                }

                div {
                    form hx-post="/submit" {
                        label for="hotdogs" { "How do you like your hotdogs?" }

                        select name="hotdogs" value="ketchup" {
                            option value="ketchup" { "Ketchup" }
                            option value="still-ketchup" { "Still ketchup" }
                            option value="always-ketchup" { "It'll always be ketchup!" }
                        }

                        button type="submit" { "Submit without token" }
                    }
                }
            }
        }
    })
}

async fn submit() -> (StatusCode, &'static str) {
    (StatusCode::OK, "Success!")
}

async fn login(token: Token) -> Result<Redirect, (StatusCode, String)> {
    token
        .set("secret-session-identifier")
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(Redirect::to("/"))
}

async fn logout(token: Token) -> Redirect {
    token.reset();

    Redirect::to("/")
}
