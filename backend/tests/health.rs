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
	AppState {
		config: Arc::new(AppConfig::default()),
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
