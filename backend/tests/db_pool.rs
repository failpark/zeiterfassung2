//! Integration tests for deadpool-diesel pool connectivity.
//!
//! These tests require a running MariaDB instance and DATABASE_URL env var.
//! Run with: just test db_pool (or cargo test --test db_pool)

use diesel::prelude::*;

#[tokio::test]
async fn pool_executes_select_1() {
	dotenvy::dotenv().ok();
	let database_url = std::env::var("DATABASE_URL")
		.expect("DATABASE_URL must be set for this test");

	let manager = deadpool_diesel::mysql::Manager::new(
		database_url,
		deadpool_diesel::Runtime::Tokio1,
	);
	let pool = deadpool_diesel::mysql::Pool::builder(manager)
		.max_size(2)
		.build()
		.expect("Failed to build test pool");

	let conn = pool.get().await.expect("Failed to get connection from pool");
	let result: i32 = conn
		.interact(|conn| {
			diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("1"))
				.get_result(conn)
		})
		.await
		.expect("interact() failed")
		.expect("SELECT 1 query failed");

	assert_eq!(result, 1);
}

#[tokio::test]
async fn pool_can_get_multiple_connections() {
	dotenvy::dotenv().ok();
	let database_url = std::env::var("DATABASE_URL")
		.expect("DATABASE_URL must be set for this test");

	let manager = deadpool_diesel::mysql::Manager::new(
		database_url,
		deadpool_diesel::Runtime::Tokio1,
	);
	let pool = deadpool_diesel::mysql::Pool::builder(manager)
		.max_size(4)
		.build()
		.expect("Failed to build test pool");

	// Get two connections concurrently
	let (conn1, conn2) = tokio::join!(pool.get(), pool.get());
	let conn1 = conn1.expect("Failed to get first connection");
	let conn2 = conn2.expect("Failed to get second connection");

	let (r1, r2) = tokio::join!(
		conn1.interact(|conn| {
			diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("1"))
				.get_result::<i32>(conn)
		}),
		conn2.interact(|conn| {
			diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("2"))
				.get_result::<i32>(conn)
		}),
	);

	assert_eq!(r1.expect("interact 1 failed").expect("query 1 failed"), 1);
	assert_eq!(r2.expect("interact 2 failed").expect("query 2 failed"), 2);
}
