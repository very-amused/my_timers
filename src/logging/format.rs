use tracing_appender::non_blocking::NonBlocking;
use tracing_subscriber::fmt::{FormatEvent, format::{self, DefaultFields}};
use tracing_subscriber::registry;
use tracing_subscriber::prelude::*;

use super::BoxedLayer;

pub use format::format as fmt;

macro_rules! config_fmt {
	($fmt:ident) => {
		$fmt
			.with_timer(time_format::timer())
			.with_line_number(false)
			.with_file(false)
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

