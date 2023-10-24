use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::fmt::{FormatEvent, format, FormatFields};
use tracing_subscriber::registry;
use tracing_subscriber::prelude::*;

use super::{BoxedLayer, time_format};

pub use format::format as fmt;

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
macro_rules! fmt_layer {
	($non_blocking:ident,$_guard:ident,$log_fmt:expr,$field_fmt:expr) => {
		(Some(fmt_layer($non_blocking, config_fmt!($log_fmt), $field_fmt)), Some($_guard))
	}
}

/// Create a guarded formatting layer
pub fn guarded_fmt_layer<S: tracing::Subscriber + for<'a> registry::LookupSpan<'a>>(writer: NonBlocking, _guard: WorkerGuard, config_format: Option<&str>) -> (Option<BoxedLayer<S>>, Option<WorkerGuard>) {
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


/// Formatting layer using log_format as formatter
pub fn fmt_layer<S, N, E>(writer: NonBlocking, log_fmt: E, field_fmt: N) -> BoxedLayer<S>
where
	S: tracing::Subscriber + for<'a> registry::LookupSpan<'a>,
	N: for<'a> FormatFields<'a> + Send + Sync + 'static,
	E: FormatEvent<S, N> + Send + Sync + 'static
{
	tracing_subscriber::fmt::layer()
		.with_writer(writer)
		.fmt_fields(field_fmt)
		.event_format(log_fmt)
		.boxed()
}

