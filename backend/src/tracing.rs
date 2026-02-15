use tracing_subscriber::{
	fmt,
	layer::SubscriberExt,
	util::SubscriberInitExt,
	EnvFilter,
};

pub fn init_tracing(is_production: bool) {
	let env_filter = EnvFilter::try_from_default_env()
		.unwrap_or_else(|_| EnvFilter::new("info"));

	let registry = tracing_subscriber::registry().with(env_filter);

	if is_production {
		registry.with(fmt::layer().json()).init();
	} else {
		registry.with(fmt::layer().pretty()).init();
	}
}
