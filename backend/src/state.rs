use std::sync::Arc;

use crate::config::AppConfig;

pub type DbPool = deadpool_diesel::mysql::Pool;

#[derive(Clone)]
pub struct AppState {
	pub config: Arc<AppConfig>,
	pub db_pool: DbPool,
}
