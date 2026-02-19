use std::sync::Arc;

use axum::{
	body::Body,
	http::{
		Request,
		StatusCode,
	},
};
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
async fn health_returns_200_without_auth() {
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
