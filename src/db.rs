use serde::Deserialize;
use self::error::DBConfigError;
use sqlx::{mysql, postgres, sqlite, ConnectOptions};
use std::collections::HashSet;
use lazy_static::lazy_static;

pub mod error;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
	user: String,
	password: String,
	pub database: String,

	// Connection protocol to use, TCP or SOCKET are supported
	protocol: String,
	address: String,
	tls: bool,

	// Database driver; "mysql"/"mariadb", "postgres", or "sqlite" is supported
	pub driver: String
}

impl Config {
	/// Set default address (if one was not provided) to connect via Unix socket. Should be run before
	/// validation. Does nothing if the configured driver doesn't have a default address.
	pub fn set_default_address(&mut self) {
		match self.driver.as_str() {
			"mariadb" | "mysql" => self.address = "/var/run/mysqld/mysqld.sock".into(),
			"postgres" => self.address = "".into(), // FIXME: Figure out default socket addr for postgres
			_ => ()
		}
	}

	pub fn validate(&self) -> Result<(), DBConfigError> {
		lazy_static! {
			// Valid connection protocols
			static ref PROTOCOLS: HashSet<&'static str> = HashSet::from(["TCP", "SOCKET"]);
		}
		if !PROTOCOLS.contains(self.protocol.as_str()) {
			return Err(DBConfigError::InvalidProtocol(self.protocol.clone()));
		}
		// Validate driver and driver specific opts
		match self.driver.as_str() {
			"mariadb" | "mysql" | "postgres" => {
				if self.user.is_empty() {
					return Err(DBConfigError::MissingField("user".into()));
				}
				if self.database.is_empty() {
					return Err(DBConfigError::MissingField("database".into()));
				}
			},
			"sqlite" => {
				// HACK: We're discarding the default unix socket path when using sqlite. It would be
				// better to never assign that default
				if self.address.ends_with(".sock") || self.address.is_empty() {
					return Err(DBConfigError::MissingField("address".into()));
				}
			},
			_ => {
				return Err(DBConfigError::InvalidDriver(self.driver.clone()));
			}
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

	// Print-friendly identifier containing protocol, address, database, and tls options
	pub fn pretty_name(&self) -> String {
		// Format here is inspired by mysql DSNs
		if self.protocol == "SOCKET" {
			format!("{} via unix socket", self.database)
		} else {
			format!("{}/{} via {} (tls {})", self.address, self.database, self.protocol, if self.tls { "enabled" } else { "disabled" })
		}
	}
}

impl Default for Config {
	fn default() -> Self {
		Self {
			user: String::new(),
			password: String::new(),
			database: String::new(),

			protocol: "SOCKET".into(),
			address: String::new(),
			tls: false,

			driver: "mariadb".into()
		}
	}
}
