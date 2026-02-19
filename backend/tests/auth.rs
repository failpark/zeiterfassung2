use std::sync::Arc;

use axum::{
	body::Body,
	http::{
		Request,
		StatusCode,
	},
};
use http_body_util::BodyExt;
use tower::ServiceExt;
use zeiterfassung_backend::{
	app,
	config::AppConfig,
	AppState,
};

fn test_state() -> AppState {
	// Use a placeholder URL — pool.build() does not connect; connections are lazy.
	// Tests that do not exercise DB routes do not call pool.get(), so no connection is made.
	let manager = deadpool_diesel::mysql::Manager::new(
		"mysql://test:test@localhost/test",
		deadpool_diesel::Runtime::Tokio1,
	);
	let db_pool = deadpool_diesel::mysql::Pool::builder(manager)
		.max_size(1)
		.build()
		.expect("Failed to build test pool");
	AppState {
		config: Arc::new(AppConfig::default()),
		db_pool,
	}
}

#[tokio::test]
async fn ping_without_auth_returns_401() {
	let response = app(test_state())
		.oneshot(
			Request::builder()
				.uri("/api/ping")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body_str = std::str::from_utf8(&body).unwrap();
	assert!(
		body_str.contains("UNAUTHORIZED"),
		"body should contain UNAUTHORIZED code: {body_str}"
	);
}

#[tokio::test]
async fn ping_with_valid_bearer_returns_200() {
	let response = app(test_state())
		.oneshot(
			Request::builder()
				.uri("/api/ping")
				.header("Authorization", "Bearer test-token")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);

	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body_str = std::str::from_utf8(&body).unwrap();
	assert_eq!(body_str, "pong");
}

#[tokio::test]
async fn ping_with_empty_bearer_returns_401() {
	let response = app(test_state())
		.oneshot(
			Request::builder()
				.uri("/api/ping")
				.header("Authorization", "Bearer ")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn ping_with_basic_auth_returns_401() {
	let response = app(test_state())
		.oneshot(
			Request::builder()
				.uri("/api/ping")
				.header("Authorization", "Basic dXNlcjpwYXNz")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn health_without_auth_returns_200() {
	let response = app(test_state())
		.oneshot(
			Request::builder()
				.uri("/health")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
}
