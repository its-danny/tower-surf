<div align="center">
  <h1>🌊 tower-surf</h1>

  A stateless CSRF middleware for [tower][crate-tower].

  [![Crates](https://img.shields.io/crates/v/tower-surf.svg)](https://crates.io/crates/tower-surf)
  ![MSRV](https://img.shields.io/crates/msrv/tower-surf)
  [![Docs](https://docs.rs/tower-surf/badge.svg)](https://docs.rs/tower-surf)
  ![CI](https://github.com/its-danny/tower-surf/actions/workflows/ci.yml/badge.svg)
</div>

## 🏄‍♂️ Overview

This crate uses the [Double Submit Cookie Pattern][owasp-double-submit] to mitigate CSRF.

### How it works

- **Secret key**: You provide a **secret key** used to sign CSRF tokens. This secret is secured by [secstr][crate-secstr] and only
in memory as plaintext during the signing and validating processes.
For more information on managing your secret key, see [OWASP's Cryptographic Storage Cheat Sheet][owasp-cryptographic-storage]).
- **Token creation**:
  - We generate a **message** by combining a unique **session identifier** with a cryptographically secure **random value** (using the [`rand`][crate-rand] crate).
  - We then create an **signature** using the **secret key** and the **message**.
  - The token is formed by concatenating the **signature** and the **message**.
- **Token storage**:
  - The server sends the token to the client in two ways:
    - As a cookie (handled by us).
    - In the header of subsequent requests (handled by you).
- **Token validation**:
  - For each incoming request that would mutate state:
    - We extract the token from the request headers.
    - We split the token into the **signature** and the **message**.
    - We recalculate the **signature** using the **secret key** and compare them.
  - If the **signature** is valid and the token matches the value stored in the cookie, the request is allowed to proceed.

### Cookies

By default, the cookies are set to `HTTPOnly`, `SameSite: Strict`, and `Secure`.

## 📦 Install

```toml
[dependencies]
tower-surf = "0.3.0"
```

## 🗝️ Usage

### With [`axum`][crate-axum]

```rust
use std::net::SocketAddr;

use axum::{routing::get, Router};
use http::StatusCode;
use tower_surf::{Surf, Token};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/login", get(login)).route("/logout", get(logout))
        .layer(Surf::new("secret-key").secure(false));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn login(token: Token) -> Result<StatusCode, StatusCode> {
    token.set("unique-session-id").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

async fn logout(token: Token) -> StatusCode {
    token.reset();

    StatusCode::OK
}
```

> [!NOTE]
> See the [examples][examples] for a full example.

## 🥰 Thank you

- I read a lot of the [tower-sessions](https://github.com/maxcountryman/tower-sessions) codebase to figure out how to make a tower project.
- The [tokio community](https://discord.com/invite/tokio) answered a lot of silly questions.

[crate-axum]: https://github.com/tokio-rs/axum
[crate-rand]: https://github.com/rust-random/rand
[crate-tower]: https://github.com/tower-rs/tower
[crate-secstr]: https://codeberg.org/valpackett/secstr
[examples]: https://github.com/its-danny/tower-surf/tree/main/examples
[owasp-cryptographic-storage]: https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html
[owasp-double-submit]: https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html#alternative-using-a-double-submit-cookie-pattern
