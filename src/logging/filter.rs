use tracing::Level;
use tracing_subscriber::filter::Targets;

pub fn filter(verbose: bool) -> Targets {
	Targets::default()
		.with_default(Level::INFO) // Block debug logs from other crates
		.with_target("my_timers", if verbose { Level::TRACE } else { Level::DEBUG })
}
