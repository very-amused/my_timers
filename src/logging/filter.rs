use tracing::Level;
use tracing_subscriber::filter::Targets;

pub fn filter() -> Targets {
	if cfg!(debug_assertions) {
		Targets::default()
			.with_default(Level::DEBUG)
			.with_target("my_timers", Level::TRACE)
	} else {
		Targets::default()
			.with_default(Level::INFO)
			.with_target("my_timers", Level::DEBUG)
	}
}
