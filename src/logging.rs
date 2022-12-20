use std::{path::Path, io::Write};

use serde::Deserialize;
use tracing_appender::{non_blocking, non_blocking::{WorkerGuard}, rolling};
use tracing_subscriber::{Layer, registry, Registry};
use tracing_subscriber::prelude::*;

mod time_format;
mod filter;

#[derive(Deserialize)]
pub struct Config {
	pub file: Option<FileConfig>
}

impl Config {
	// Initialize tracing
	pub fn init(&self) -> Option<WorkerGuard> {
		let (file_log, _guard) = match &self.file {
			Some(f) => f.layer::<Registry>(),
			None => (None, None)
		};
		registry()
			.with(file_log)
			// Filter logs
			.with(filter::filter())
			.init();
		_guard
	}
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

pub type BoxedLayer<S> = Box<dyn Layer<S> + Send + Sync + 'static>;
pub trait RegLayer { // Registry layer, used for multi-layer logging composition
	fn layer<S: tracing::Subscriber + for<'a> registry::LookupSpan<'a>>(&self) -> Option<BoxedLayer<S>>;
}
// Guarded registry layer for non-blocking logging threads
// (i.e for allowing multiple processes to write to the same file)
pub trait GuardedRegLayer { 
	fn layer<S: tracing::Subscriber + for<'a> registry::LookupSpan<'a>>(&self) -> (Option<BoxedLayer<S>>, Option<WorkerGuard>);
}

// File logging should be done on a non-blocking layer, which must be created (along with its corresponding guard variable)
// after calling this function.
impl GuardedRegLayer for FileConfig {
	fn layer<S: tracing::Subscriber + for<'a> registry::LookupSpan<'a>>(&self) -> (Option<BoxedLayer<S>>, Option<WorkerGuard>) {
		if !self.enabled {
			return (None, None);
		}
		let path = Path::new(&self.path);
		let dir = match path.parent() {
			Some(d) => d,
			None => return (None, None)
		};
		let file = match path.file_name() {
			Some(d) => d,
			None => return (None, None)
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
				eprintln!("Failed to clear logfile {}: {}", &self.path, e);
			}
		}
		let (non_blocking, _guard) = non_blocking(file_appender);
		let layer = tracing_subscriber::fmt::layer()
			.with_writer(non_blocking)
			.with_timer(time_format::timer())
			.boxed();
		(Some(layer), Some(_guard))
	}
}

fn default_enabled() -> bool {
	true
}
