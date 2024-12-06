use mysql_async;
use serde::Deserialize;
use self::error::DBConfigError;
use std::collections::HashSet;
use lazy_static::lazy_static;

pub mod error;

#[derive(Deserialize)]
pub struct Config {
	user: String,
	#[serde(default)]
	password: String,

	// Connection protocol to use, TCP or SOCKET are supported
	#[serde(default = "Config::default_protocol")]
	protocol: String,

	#[serde(default = "Config::default_address")]
	address: String,

	pub database: String,

	#[serde(default = "Config::default_tls")]
	tls: bool,

	// Database driver; "mysql"/"mariadb", "postgres", or "sqlite" is supported
	#[serde(default = "Config::default_driver")]
	driver: String
}

impl Config {
	pub fn validate(&self) -> Result<(), DBConfigError> {
		lazy_static! {
			// Valid DB drivers
			static ref DRIVERS: HashSet<&'static str> = HashSet::from(["mariadb", "mysql", "postgres", "sqlite"]);
			// Valid connection protocols
			static ref PROTOCOLS: HashSet<&'static str> = HashSet::from(["TCP", "SOCKET"]);
		}
		if !DRIVERS.contains(self.driver.as_str()) {
			return Err(DBConfigError::InvalidDriver(self.driver.clone()));
		}
		if !PROTOCOLS.contains(self.protocol.as_str()) {
			return Err(DBConfigError::InvalidProtocol(self.protocol.clone()))
		}

		if self.driver != "sqlite" {
		}

		Ok(())
	}

	#[deprecated]
	pub fn mysql_opts(&self) -> mysql_async::Opts {
		// Configure user, password, db
		let mut builder = mysql_async::OptsBuilder::default()
			.user(Some(&self.user))
			.pass(Some(&self.password))
			.db_name(Some(&self.database));
		// Configure connection addr
		builder = if self.protocol == "SOCKET" {
			builder.socket(Some(&self.address))
		} else {
			builder.ip_or_hostname(&self.address).prefer_socket(false)
		};
		// Configure TLS
		if self.tls {
			builder = builder.ssl_opts(mysql_async::SslOpts::default());
		}
		builder.into()
	}

	// Print-friendly identifier containing protocol, address, database, and tls options
	pub fn pretty_name(&self) -> String {
		// Format here is inspired by mysql DSNs
		if self.protocol == "SOCKET" {
			format!("{} via unix socket", self.database)
		} else {
			format!("{}/{} via {} (tls {})", self.address, self.database, self.protocol, if self.tls { "enabled" } else { "disabled" })
		}
	}

	fn default_driver() -> String {
		"mariadb".into()
	}
	fn default_protocol() -> String {
		"SOCKET".into()
	}
	fn default_address() -> String {
		"/var/run/mysqld/mysqld.sock".into()
	}
	fn default_tls() -> bool {
		false
	}
}
