//! ## 🏄‍♂️ Overview
//!
//! This crate uses the [Double Submit Cookie Pattern][owasp-double-submit] to mitigate CSRF.
//!
//! ### How it works
//!
//! - **Secret key**: You provide a **secret key** used to sign CSRF tokens (See: [OWASP's Cryptographic Storage Cheat Sheet][owasp-cryptographic-storage]).
//! - **Token creation**:
//!   - We generate a **message** by combining a unique **session identifier** with a cryptographically secure **random value** (using the [`rand`][crate-rand] crate).
//!   - We then create an **signature** using the **secret key** and the **message**.
//!   - The token is formed by concatenating the **signature** and the **message**.
//! - **Token storage**:
//!   - The server sends the token to the client in two ways:
//!     - As a cookie (handled by us).
//!     - In the header of subsequent requests (handled by you).
//! - **Token validation**:
//!   - For each incoming request:
//!     - We extract the token from the request headers.
//!     - We split the token into the **signature** and the **message**.
//!     - We recalculate the **signature** using the **secret key** and compare them.
//!   - If the **signature** is valid and the token matches the value stored in the cookie, the request is allowed to proceed.
//!
//! ### Cookies
//!
//! By default the cookies are set to `HTTPOnly`, `SameSite: Strict`, and `Secure`.
//!
//! ## 🗝️ Usage
//!
//! ### With [`axum`][crate-axum]
//!
//! ```rust, no_run
//! use std::net::SocketAddr;
//!
//! use axum::{routing::get, Router};
//! use http::StatusCode;
//! use tower_surf::{Surf, Token};
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = Router::new()
//!         .route("/login", get(login)).route("/logout", get(logout))
//!         .layer(Surf::new("secret-key").secure(false));
//!
//!     let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//!     let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
//!
//!     axum::serve(listener, app.into_make_service())
//!         .await
//!         .unwrap();
//! }
//!
//! async fn login(token: Token) -> Result<StatusCode, StatusCode> {
//!     token.set("unique-session-id").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
//!
//!     Ok(StatusCode::OK)
//! }
//!
//! async fn logout(token: Token) -> StatusCode {
//!     token.reset();
//!
//!     StatusCode::OK
//! }
//! ```
//!
//! [crate-axum]: https://github.com/tokio-rs/axum
//! [crate-rand]: https://github.com/rust-random/rand
//! [crate-tower]: https://github.com/tower-rs/tower
//! [owasp-cryptographic-storage]: https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html
//! [owasp-double-submit]: https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html#alternative-using-a-double-submit-cookie-pattern
//! [owasp-login-csrf]: https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html#possible-csrf-vulnerabilities-in-login-forms

use hmac::Hmac;
use sha2::Sha256;

pub(crate) type HmacSha256 = Hmac<Sha256>;

pub use error::Error;
pub use surf::Surf;
pub use token::Token;

mod error;
mod guard;
mod surf;
mod token;

#[cfg(feature = "axum")]
mod extract;
