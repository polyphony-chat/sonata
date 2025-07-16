// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use log::info;
use poem::{
	EndpointExt, IntoResponse, Response, Route, Server,
	error::ResponseError,
	handler,
	http::{Method, StatusCode},
	listener::TcpListener,
	middleware::{Cors, NormalizePath},
	web::Json,
};
use serde_json::json;

use crate::{
	config::ApiConfig,
	database::{Database, tokens::TokenStore},
	errors::Error,
};

/// Admin-only functionality.
pub(super) mod admin;
/// Authentication functionality.
mod auth;
/// Routes coveringthe "federated identity" section of the polyproto-core
/// specification.
mod federated_identity;
/// Custom middlewares, such as authentication and active-user.
pub(crate) mod middlewares;
/// API models, such as response schemas
pub(crate) mod models;

#[allow(clippy::expect_used)]
#[cfg_attr(coverage_nightly, coverage(off))]
/// Build the API [Route]s and start a `tokio::task`, which is a poem [Server]
/// processing incoming HTTP API requests.
pub(super) fn start_api(
	api_config: ApiConfig,
	db: Database,
	token_store: TokenStore,
) -> tokio::task::JoinHandle<()> {
	let routes = Route::new()
		.at("/healthz", healthz)
		.nest("/.p2/core/", setup_p2_core_routes())
		.nest("/.p2/auth/", auth::setup_routes())
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
		.data(db)
		.data(token_store);

	let api_config_clone = api_config.clone();
	let handle = tokio::task::spawn(async move {
		Server::new(TcpListener::bind((api_config.host.as_str().trim(), api_config.port)))
			.run(routes)
			.await
			.expect("Failed to start HTTP server");
		log::info!("HTTP Server stopped");
	});
	info!("Started HTTP API server at {}, port {}", api_config_clone.host, api_config_clone.port);
	handle
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[handler]
fn healthz() -> impl IntoResponse {
	Response::builder().status(StatusCode::OK).finish()
}

#[cfg_attr(coverage_nightly, coverage(off))]
/// All routes under `/.p2/core/`.
fn setup_p2_core_routes() -> Route {
	Route::new()
}
