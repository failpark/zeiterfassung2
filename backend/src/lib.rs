use std::time::Duration;

use axum::{
	http::{
		header,
		HeaderValue,
		Method,
	},
	routing::get,
	Router,
};
use tower::ServiceBuilder;
use tower_http::{
	cors::{
		AllowOrigin,
		CorsLayer,
	},
	request_id::{
		MakeRequestUuid,
		PropagateRequestIdLayer,
		SetRequestIdLayer,
	},
	trace::{
		DefaultOnResponse,
		TraceLayer,
	},
};

pub mod config;
pub mod error;
mod routes;
pub mod state;
pub mod tracing;

// TODO: Port to Axum in Phase 3+
// mod auth;
// mod catchers;
// mod db;
// mod guard;
// mod schema;
// #[cfg(test)] mod test;

pub use config::AppConfig;
pub use error::{
	Error,
	Result,
};
pub use state::AppState;

fn cors_layer(config: &AppConfig) -> CorsLayer {
	let origins = if config.is_development() {
		AllowOrigin::mirror_request()
	} else {
		AllowOrigin::list(
			config
				.cors
				.allowed_origins
				.iter()
				.map(|o| o.parse::<HeaderValue>().expect("invalid CORS origin")),
		)
	};

	CorsLayer::new()
		.allow_origin(origins)
		.allow_methods([
			Method::GET,
			Method::POST,
			Method::PUT,
			Method::DELETE,
			Method::OPTIONS,
		])
		.allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
		.allow_credentials(true)
		.max_age(if config.is_development() {
			Duration::from_secs(0)
		} else {
			Duration::from_secs(600)
		})
}

pub fn app(state: AppState) -> Router {
	let cors = cors_layer(&state.config);

	let middleware_stack = ServiceBuilder::new()
		.layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
		.layer(
			TraceLayer::new_for_http()
				.make_span_with(|request: &axum::http::Request<_>| {
					let request_id = request
						.headers()
						.get("x-request-id")
						.and_then(|v| v.to_str().ok())
						.unwrap_or("unknown");
					::tracing::info_span!(
						"http_request",
						method = %request.method(),
						uri = %request.uri(),
						request_id = request_id,
					)
				})
				.on_response(DefaultOnResponse::new().level(::tracing::Level::INFO)),
		)
		.layer(cors)
		.layer(PropagateRequestIdLayer::x_request_id());

	let public = Router::new().route("/health", get(routes::health::health));

	Router::new()
		.merge(public)
		.layer(middleware_stack)
		.with_state(state)
}
