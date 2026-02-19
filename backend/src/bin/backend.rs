use std::sync::Arc;

use zeiterfassung_backend::{
	app,
	AppState,
};

fn redact_password(url: &str) -> String {
	if let Some(at_pos) = url.find('@') {
		if let Some(colon_pos) = url[..at_pos].rfind(':') {
			let mut redacted = url.to_string();
			redacted.replace_range((colon_pos + 1)..at_pos, "***");
			return redacted;
		}
	}
	url.to_string()
}

#[tokio::main]
async fn main() {
	let config = zeiterfassung_backend::config::load_config().expect("Failed to load configuration");

	zeiterfassung_backend::tracing::init_tracing(config.environment != "development");

	tracing::info!(
		"Starting server on {}:{}",
		config.server.host,
		config.server.port
	);

	let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	let manager = deadpool_diesel::mysql::Manager::new(
		database_url.clone(),
		deadpool_diesel::Runtime::Tokio1,
	);
	let db_pool = deadpool_diesel::mysql::Pool::builder(manager)
		.max_size(10)
		.build()
		.expect("Failed to build database pool");

	tracing::info!(
		pool_size = 10,
		connection = %redact_password(&database_url),
		"Database pool initialized"
	);

	// Run pending migrations at startup — before accepting any requests
	{
		use diesel_migrations::{
			embed_migrations,
			EmbeddedMigrations,
			MigrationHarness,
		};
		const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations");

		let conn = db_pool
			.get()
			.await
			.expect("Could not get DB connection for migrations");
		conn.interact(|conn| {
			conn.run_pending_migrations(MIGRATIONS).map(|_| ())
		})
		.await
		.expect("Migration interact() failed")
		.expect("Migration execution failed");
		tracing::info!("Database migrations applied successfully");
	}

	let state = AppState {
		config: Arc::new(config),
		db_pool,
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
