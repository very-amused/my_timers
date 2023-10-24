use std::{path::Path, io::Write};

use serde::Deserialize;
use tracing_appender::{non_blocking, non_blocking::WorkerGuard, rolling};
use tracing_subscriber::{Layer, registry};
use tracing_subscriber::prelude::*;

mod time_format;
mod filter;
mod format;

#[derive(Deserialize)]
pub struct Config {
	format: Option<String>,
	file: Option<FileConfig>,
	stdio: Option<StdioConfig>
}

pub trait LogFormat {
	/// Return the log format which should be used for the destination, respecting format option
	/// hierarchy
	fn get_format(&self) -> Option<&str>;
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

	/// Copy the global format option to be referred to by destination configs
	pub fn init_format_opts(&mut self) {
		if let Some(global_format) = &self.format {
			if let Some(file) = &mut self.file {
				file.global_format = Some(global_format.into());
			}
			if let Some(stdio) = &mut self.stdio {
				stdio.global_format = Some(global_format.into());
			}
		}
	}
}

pub type BoxedLayer<S> = Box<dyn Layer<S> + Send + Sync + 'static>;

/// Guarded tracing layer for non-blocking write threads
pub trait GuardedRegLayer : LogFormat { 
	fn layer<S: tracing::Subscriber + for<'a> registry::LookupSpan<'a>>(&self) -> (Option<BoxedLayer<S>>, Option<WorkerGuard>);
}

const fn default_enabled() -> bool {
	true
}

/// Config for logfile output (with optional rotation)
#[derive(Deserialize)]
pub struct FileConfig {
	#[serde(default = "default_enabled")]
	enabled: bool,
	format: Option<String>,
	global_format: Option<String>,
	path: String,
	#[serde(default = "FileConfig::default_rotation")]
	rotation: String
}

impl FileConfig {
	fn default_rotation() -> String {
		"daily".into()
	}
}

impl LogFormat for FileConfig {
	fn get_format(&self) -> Option<&str> {
		if let Some(fmt) = &self.global_format {
			Some(fmt)
		}	else if let Some(fmt) = &self.format {
			Some(fmt)
		} else {
			None
		}
	}
}

/// Tracing layer for file logging
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
		// Apply formatting
		format::guarded_fmt_layer(non_blocking, _guard, self.get_format())
	}
}

/// Config for stdout/stderr output
#[derive(Deserialize)]
pub struct StdioConfig {
	#[serde(default = "default_enabled")]
	enabled: bool,
	#[serde(default = "StdioConfig::default_format")]
	format: Option<String>,
	global_format: Option<String>,
	#[serde(default = "StdioConfig::default_stream")]
	stream: String
}

impl StdioConfig {
	fn default_format() -> Option<String> {
		Some("pretty".to_string())
	}
	fn default_stream() -> String {
		"stdout".to_string()
	}
}

impl LogFormat for StdioConfig {
	fn get_format(&self) -> Option<&str> {
		if let Some(fmt) = &self.global_format {
			Some(fmt)
		} else if let Some(fmt) = &self.format {
			Some(fmt)
		} else {
			None
		}
	}
}

/// Tracing layer for stdio logging
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
		// Apply formatting
		format::guarded_fmt_layer(non_blocking, _guard, self.get_format())
	}
}
