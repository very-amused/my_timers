use std::process::exit;
use std::env;

/// Program arguments gathered from CLI args and/or env variables
pub struct Args {
	pub verbose: bool,
	pub config_path: String,
	pub events_path: String
}

const CONFIG_PATH_ENV: &str = "MY_TIMERS_CONFIG";
const CONFIG_PATH_DEFAULT: &str = "config.json";
const EVENTS_PATH_ENV: &str = "MY_TIMERS_EVENTS";
const EVENTS_PATH_DEFAULT: &str = "events.conf";

/// Parse program arguments
pub fn args() -> Args {
	// Argument values to be set
	let mut verbose = false;
	let mut config_path: Option<String> = None;
	let mut events_path: Option<String> = None;

	let args: Vec<String> = env::args().collect();
	for (i, arg) in args.iter().enumerate() {
		match arg.as_str() {
			"-h" | "--help" => {
				const USAGE: &str = "my_timers [-c/--config /path/to/config.json] [-e/--events /path/to/events.conf] [-v/--verbose] [-h/--help]";
				println!("{}", "Usage:".to_owned() + "\n\t" + USAGE);
				exit(0);
			},
			"-v" | "--verbose" => {
				verbose = true;
			},
			"-c" | "--config" if i < args.len() - 1 => {
				config_path = Some(args[i+1].to_string());
			},
			"-e" | "--events" if i < args.len() - 1 => {
				events_path = Some(args[i+1].to_string());
			}
			_ => {}
		}
	}
	// Set unspecified flags from env vars
	if config_path == None {
		if let Ok(path) = env::var(CONFIG_PATH_ENV) {
			config_path = Some(path);
		} else if verbose {
			eprintln!("{} is not set, defaulting to {}", CONFIG_PATH_ENV, CONFIG_PATH_DEFAULT);
		}
	}
	if events_path == None {
		if let Ok(path) = env::var(EVENTS_PATH_ENV) {
			events_path = Some(path);
		} else if verbose {
			eprintln!("{} is not set, defaulting to {}", EVENTS_PATH_ENV, EVENTS_PATH_DEFAULT);
		}
	}

	Args {
		verbose,
		config_path: if let Some(path) = config_path { path } else { CONFIG_PATH_DEFAULT.to_owned() },
		events_path: if let Some(path) = events_path { path } else { EVENTS_PATH_DEFAULT.to_owned() }
	}
}
