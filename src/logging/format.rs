use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::fmt::format;
use tracing_subscriber::registry;
use tracing_subscriber::prelude::*;

use super::{BoxedLayer, time_format};

pub use format::format as fmt;

// Configure logging format specifics
macro_rules! config_fmt {
	($fmt:expr) => {
		$fmt
			.with_timer(time_format::timer())
			.with_line_number(false)
			.with_file(false)
			.with_source_location(false)
			.with_target(false)
	}
}

// Create a new guarded formatting layer tuple
macro_rules! fmt_layer {
	($writer:ident,$_guard:ident,$log_fmt:expr,$field_fmt:expr) => {
		(Some(
			tracing_subscriber::fmt::layer()
				.with_writer($writer)
				.fmt_fields($field_fmt)
				.event_format(config_fmt!($log_fmt))
				.boxed()
		), Some($_guard))
	}
}

/// Create a guarded formatting layer
pub fn guarded_fmt_layer<S>(writer: NonBlocking, _guard: WorkerGuard, config_format: Option<&str>) -> (Option<BoxedLayer<S>>, Option<WorkerGuard>)
where S: tracing::Subscriber + for<'a> registry::LookupSpan<'a>
{
	if let Some(config_format) = config_format {
		match config_format {
			"pretty" => fmt_layer!(writer,_guard,fmt().pretty(),format::PrettyFields::new()),
			// Default log format w/ pretty field format
			"pretty_fields" => fmt_layer!(writer,_guard,fmt(),format::PrettyFields::new()),
			"compact" => fmt_layer!(writer,_guard,fmt().compact(),format::DefaultFields::new()),
			"json" => fmt_layer!(writer,_guard,fmt().json(),format::JsonFields::new()),
			_ => fmt_layer!(writer,_guard,fmt(),format::DefaultFields::new())
		}
	} else {
		fmt_layer!(writer,_guard,fmt(),format::DefaultFields::new())
	}
}
