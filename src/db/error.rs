use std::error::Error;
use std::fmt::Display;

/// A database configuration error
#[derive(Debug)]
pub enum DBConfigError {
	InvalidDriver(String),
	InvalidProtocol(String),
	/// Missing a required field
	MissingField(String)
}

impl Display for DBConfigError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::InvalidDriver(e) => write!(f, "Invalid driver: {}", e),
			Self::InvalidProtocol(e) => write!(f, "Invalid protocol: {}", e),
			Self::MissingField(e) => write!(f, "Missing field: {}", e)
		}
	}
}

impl Error for DBConfigError {}
