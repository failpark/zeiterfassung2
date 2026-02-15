use figment::{
	providers::{
		Env,
		Format,
		Toml,
	},
	Figment,
};
use serde::{
	Deserialize,
	Serialize,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
	pub server: ServerConfig,
	pub cors: CorsConfig,
	#[serde(default = "default_log_level")]
	pub log_level: String,
	#[serde(default = "default_environment")]
	pub environment: String,
}

fn default_log_level() -> String {
	"info".to_string()
}

fn default_environment() -> String {
	"development".to_string()
}

impl AppConfig {
	pub fn is_development(&self) -> bool {
		self.environment == "development"
	}
}

impl Default for AppConfig {
	fn default() -> Self {
		Self {
			server: ServerConfig::default(),
			cors: CorsConfig::default(),
			log_level: default_log_level(),
			environment: default_environment(),
		}
	}
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
	#[serde(default = "default_host")]
	pub host: String,
	#[serde(default = "default_port")]
	pub port: u16,
}

fn default_host() -> String {
	"0.0.0.0".to_string()
}

fn default_port() -> u16 {
	8000
}

impl Default for ServerConfig {
	fn default() -> Self {
		Self {
			host: default_host(),
			port: default_port(),
		}
	}
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct CorsConfig {
	#[serde(default)]
	pub allowed_origins: Vec<String>,
}

#[allow(clippy::result_large_err)] // figment::Error is a third-party type; size not controllable
pub fn load_config() -> Result<AppConfig, figment::Error> {
	dotenvy::dotenv().ok();
	Figment::new()
		.merge(Toml::file("config.toml"))
		.merge(Env::prefixed("APP__").split("__"))
		.extract()
}
