use std::path::Path;

use serde::Deserialize;
use tracing_appender::{non_blocking, non_blocking::{WorkerGuard}, rolling};
use tracing_subscriber::{Layer, FmtSubscriber};

#[derive(Deserialize)]
pub struct Config {
	pub file: Option<FileConfig>
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

pub type GuardedLayer<S> = (Box<dyn Layer<S> + Send + Sync + 'static>, WorkerGuard);

impl FileConfig {
	pub fn layer(&self) -> Option<GuardedLayer<FmtSubscriber>> {
		if !self.enabled {
			return None;
		}
		let path = Path::new(&self.path);
		let dir = match path.parent() {
			Some(d) => d,
			None => return None
		};
		let file = match path.file_name() {
			Some(d) => d,
			None => return None
		};

		let file_appender = match self.rotation.as_str() {
			"diraily" => rolling::daily(dir, file),
			"hourly" => rolling::hourly(dir, file),
			"minutely" => rolling::minutely(dir, file),
			_ => rolling::never(dir, file)
		};
		let (non_blocking, _guard) = non_blocking(file_appender);
		let s = tracing_subscriber::fmt::layer()
			.with_writer(non_blocking)
			.boxed();
		Some((s, _guard))
	}
}

fn default_enabled() -> bool {
	true
}
