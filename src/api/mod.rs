// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use log::info;
use poem::http::{Method, StatusCode};
use poem::listener::TcpListener;
use poem::middleware::{Cors, NormalizePath};
use poem::web::Json;
use poem::{EndpointExt, IntoResponse, Response, Route, Server, handler};
use serde_json::json;

use crate::config::ApiConfig;
use crate::database::Database;
use crate::database::tokens::TokenStore;

/// Admin-only functionality.
pub(super) mod admin;
/// Authentication functionality.
mod auth;
/// Custom middlewares, such as authentication and active-user.
pub(crate) mod middlewares;
pub(crate) mod models;

#[allow(clippy::expect_used)]
#[cfg_attr(coverage_nightly, coverage(off))]
/// Build the API [Route]s and start a `tokio::task`, which is a poem [Server] processing incoming
/// HTTP API requests.
pub(super) fn start_api(
    api_config: ApiConfig,
    db: Database,
    token_store: TokenStore,
) -> tokio::task::JoinHandle<()> {
    let routes = Route::new()
        .at("/healthz", healthz)
        .nest("/.p2/core/", setup_p2_core_routes())
        .with(NormalizePath::new(poem::middleware::TrailingSlash::Trim))
        .with(Cors::new().allow_methods(&[
            Method::CONNECT,
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ]))
        .catch_all_error(custom_error)
        .data(db)
        .data(token_store);

    let handle = tokio::task::spawn(async move {
        Server::new(TcpListener::bind((api_config.host.as_str().trim(), api_config.port)))
            .run(routes)
            .await
            .expect("Failed to start HTTP server");
        log::info!("HTTP Server stopped");
    });
    info!("Started HTTP API server");
    handle
}

/// Catch-all fallback error.
async fn custom_error(err: poem::Error) -> impl IntoResponse {
    Json(json! ({
        "success": false,
        "message": err.to_string(),
    }))
    .with_status(err.status())
}

#[handler]
fn healthz() -> impl IntoResponse {
    Response::builder().status(StatusCode::OK).finish()
}

/// All routes under `/.p2/core/`.
fn setup_p2_core_routes() -> Route {
    Route::new()
}
