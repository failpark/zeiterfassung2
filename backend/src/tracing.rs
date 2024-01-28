use std::io;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
	filter::filter_fn,
	fmt,
	layer::{
		Layer,
		SubscriberExt,
	},
	EnvFilter,
};

pub fn init() -> WorkerGuard {
	let dir = "/var/log/zeiterfassung";

	let file_appender = tracing_appender::rolling::daily(dir, "server.log");
	let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

	let filter_bin =
		filter_fn(|metadata: &tracing::Metadata<'_>| metadata.target().starts_with("zeiterfassung"));

	let subscriber = tracing_subscriber::registry()
		.with(EnvFilter::from_default_env().add_directive(tracing::Level::TRACE.into()))
		.with(
			fmt::Layer::new()
				.with_writer(io::stdout)
				.pretty()
				.with_filter(filter_bin.clone()),
		)
		.with(
			fmt::Layer::new()
				.with_writer(non_blocking)
				.json()
				.with_filter(filter_bin),
		);
	tracing::subscriber::set_global_default(subscriber).expect("Unable to set a global subscriber");
	guard
}
