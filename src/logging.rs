use std::{path::Path, io::Write};

use serde::Deserialize;
use tracing_appender::{non_blocking, non_blocking::{WorkerGuard, NonBlocking}, rolling};
use tracing_subscriber::{Layer, registry};
use tracing_subscriber::prelude::*;

mod time_format;
mod filter;

#[derive(Deserialize)]
pub struct Config {
	file: Option<FileConfig>,
	stdio: Option<StdioConfig>
}

impl Config {
	// Initialize tracing
	pub fn init(&self, verbose: bool) -> Vec<Option<WorkerGuard>> {
		let mut _guards: Vec<Option<WorkerGuard>> = Vec::with_capacity(2);

		macro_rules! layer {
			($config:expr) => {
				{
					if let Some(c) = $config {
						let (layer, _guard) = c.layer();
						_guards.push(_guard);
						layer
					} else { None }
				}
			}
		}

		let file_log = layer!(&self.file);
		let stdio_log = layer!(&self.stdio);

		registry()
			.with(file_log)
			.with(stdio_log)
			// Filter logs
			.with(filter::filter(verbose))
			.init();
		_guards
	}
}

pub type BoxedLayer<S> = Box<dyn Layer<S> + Send + Sync + 'static>;
pub trait RegLayer { // Tracing layer, used for multi-layer logging composition
	fn layer<S: tracing::Subscriber + for<'a> registry::LookupSpan<'a>>(&self) -> Option<BoxedLayer<S>>;
}
// Guarded registry layer for non-blocking logging threads
// (i.e for allowing multiple processes to write to the same file)
pub trait GuardedRegLayer { 
	fn layer<S: tracing::Subscriber + for<'a> registry::LookupSpan<'a>>(&self) -> (Option<BoxedLayer<S>>, Option<WorkerGuard>);
}

// Apply consistent formatting to output layers
fn fmt_layer<S>(writer: NonBlocking) -> BoxedLayer<S>
where S: tracing::Subscriber + for<'a> registry::LookupSpan<'a>
{
	tracing_subscriber::fmt::layer()
		.with_writer(writer)
		.with_timer(time_format::timer())
		.with_target(false)
		.boxed()
}

// Config for file logging (with optional rotation)
#[derive(Deserialize)]
pub struct FileConfig {
	#[serde(default = "default_enabled")]
	enabled: bool,
	path: String,
	#[serde(default = "FileConfig::default_rotation")]
	rotation: String
}

impl FileConfig {
	fn default_rotation() -> String {
		"daily".into()
	}
}

// File logging is done on a non-blocking layer
impl GuardedRegLayer for FileConfig {
	fn layer<S: tracing::Subscriber + for<'a> registry::LookupSpan<'a>>(&self) -> (Option<BoxedLayer<S>>, Option<WorkerGuard>) {
		if !self.enabled {
			return (None, None);
		}
		let path = Path::new(&self.path);
		let dir = match path.parent() {
			Some(d) => d,
			None => {
				eprintln!("No directory provided for logfile: {}", &self.path);
				return (None, None);
			}
		};
		let file = match path.file_name() {
			Some(d) => d,
			None => {
				eprintln!("No filename provided for logfile: {}", &self.path);
				return (None, None);
			}
		};

		let mut file_appender = match self.rotation.as_str() {
			"diraily" => rolling::daily(dir, file),
			"hourly" => rolling::hourly(dir, file),
			"minutely" => rolling::minutely(dir, file),
			_ => rolling::never(dir, file)
		};
		if cfg!(debug_assertions) {
			// Clear screen for observers such as tail
			const CLS: &str = "\x1b[H\x1b[J";
			if let Err(e) = write!(&mut file_appender, "{}", CLS) {
				eprintln!("Failed to clear logfile view {}: {}", &self.path, e);
			}
		}
		let (non_blocking, _guard) = non_blocking(file_appender);
		(Some(fmt_layer(non_blocking)), Some(_guard))
	}
}

fn default_enabled() -> bool {
	true
}
// Config for logging to stdout/stderr
#[derive(Deserialize)]
pub struct StdioConfig {
	#[serde(default = "default_enabled")]
	enabled: bool,
	#[serde(default = "StdioConfig::default_stream")]
	stream: String
}

impl StdioConfig {
	fn default_stream() -> String {
		"stdout".to_string()
	}
}

impl GuardedRegLayer for StdioConfig {
	fn layer<S: tracing::Subscriber + for<'a> registry::LookupSpan<'a>>(&self) -> (Option<BoxedLayer<S>>, Option<WorkerGuard>) {
		if !self.enabled {
			return (None, None);
		}
		let stream: Box<dyn Write + Send + Sync + 'static> = match self.stream.as_str() {
			"stdout" => Box::new(std::io::stdout()),
			"stderr" => Box::new(std::io::stderr()),
			s => {
				eprintln!("Invalid stdio stream: {}. Defaulting to stdout", s);
				Box::new(std::io::stdout())
			}
		};
		let (non_blocking, _guard) = non_blocking(stream);	
		(Some(fmt_layer(non_blocking)), Some(_guard))
	}
}