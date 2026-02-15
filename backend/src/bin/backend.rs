use std::sync::Arc;

use zeiterfassung_backend::{
	app,
	AppState,
};

#[tokio::main]
async fn main() {
	let config = zeiterfassung_backend::config::load_config()
		.expect("Failed to load configuration");

	zeiterfassung_backend::tracing::init_tracing(config.environment != "development");

	tracing::info!(
		"Starting server on {}:{}",
		config.server.host,
		config.server.port
	);

	let state = AppState {
		config: Arc::new(config),
	};

	let listener = tokio::net::TcpListener::bind(format!(
		"{}:{}",
		state.config.server.host, state.config.server.port
	))
	.await
	.expect("Failed to bind");

	tracing::info!("Listening on {}", listener.local_addr().unwrap());

	axum::serve(listener, app(state))
		.await
		.expect("Server error");
}
