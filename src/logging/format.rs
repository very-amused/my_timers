use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::fmt::{FormatEvent, format::{self, DefaultFields}};
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
	}
}
macro_rules! fmt_layer {
	($non_blocking:ident,$_guard:ident,$fmt:expr) => {
		(Some(fmt_layer($non_blocking, config_fmt!($fmt))), Some($_guard))
	}
}

/// Create a guarded formatting layer
pub fn guarded_fmt_layer<S: tracing::Subscriber + for<'a> registry::LookupSpan<'a>>(writer: NonBlocking, _guard: WorkerGuard, config_format: Option<&str>) -> (Option<BoxedLayer<S>>, Option<WorkerGuard>) {
	if let Some(config_format) = config_format {
		match config_format {
			"pretty" => fmt_layer!(writer,_guard,fmt().pretty()),
			"compact" => fmt_layer!(writer,_guard,fmt().compact()),
			"json" => fmt_layer!(writer,_guard,fmt().json()),
			_ => fmt_layer!(writer,_guard,fmt())
		}
	} else {
		fmt_layer!(writer,_guard,fmt())
	}
}


/// Formatting layer using log_format as formatter
pub fn fmt_layer<S, E>(writer: NonBlocking, log_format: E) -> BoxedLayer<S>
where
	S: tracing::Subscriber + for<'a> registry::LookupSpan<'a>,
	E: FormatEvent<S, DefaultFields> + Send + Sync + 'static
{
	tracing_subscriber::fmt::layer()
		.with_writer(writer)
		.event_format(log_format)
		.boxed()
}

