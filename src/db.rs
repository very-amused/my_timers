use mysql_async;
use serde::Deserialize;
use self::error::DBConfigError;
use sqlx::{mysql, postgres, sqlite, ConnectOptions};
use std::{collections::HashSet, str::FromStr};
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

		Ok(())
	}

	/// Return an sqlx-compatible DB connection URL
	pub fn sqlx_url(&self) -> Result<String, DBConfigError> {
		match self.driver.as_str() {
			"mariadb" | "mysql" => {
				let mut opts = mysql::MySqlConnectOptions::new()
					.username(&self.user)
					.password(&self.password)
					.database(&self.database);
				// TODO v0.3.1: parse (but don't verify) events before connecting to DB,
				// set statement-cache-capacity such that all queries in events.conf are cached
				if self.protocol == "SOCKET" {
					opts = opts.socket(&self.address);
				} else {
					opts = opts.host(&self.address);
				}
				if self.tls {
					// TODO: implement tls option for VerifyCA
					opts = opts.ssl_mode(mysql::MySqlSslMode::Required);
				}
				Ok(opts.to_url_lossy().into())
			},
			"postgres" => {
				let mut opts = postgres::PgConnectOptions::new()
					.host(&self.address) // Automatically interpreted as a Unix socket if it
																								// starts with a slash
					.username(&self.user)
					.password(&self.password)
					.database(&self.database);
				if self.tls {
					opts = opts.ssl_mode(postgres::PgSslMode::Require);
				}
				Ok(opts.to_url_lossy().into())
			},
			"sqlite" => {
				let opts = sqlite::SqliteConnectOptions::new()
					.filename(&self.address)
					.foreign_keys(true) // sqlx default, but you can never be too safe
					// WAL journaling allows unlimited readers and 1 writer at a given time.
					// This setting is applied to other programs accessing the same DB.
					// Mandatory WAL is 1/2 of what we can do to minimize SQLITE_BUSY
					// locking failures:
					// 1. Ensure reads from my_timers and all other programs never contend (WAL)
					// 2. Ensure writes from my_timers never contend with each other (serialize w/ MPSC)
					.journal_mode(sqlite::SqliteJournalMode::Wal);

					Ok(opts.to_url_lossy().into())
			},
			_ => Err(DBConfigError::InvalidDriver(self.driver.clone()))
		}
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
