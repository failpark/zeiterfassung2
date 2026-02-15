use std::sync::Arc;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
	pub config: Arc<AppConfig>,
}
