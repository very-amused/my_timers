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
	let cli_args: Vec<String> = env::args().collect();
	{
		let show_help = cli_args.iter().any(|a| a == "-h" || a == "--help");
		if show_help {
			const USAGE: &str = "my_timers [-c/--config /path/to/config.json] [-e/--events /path/to/events.conf] [-v/--verbose] [-h/--help]";
			println!("{}", "Usage:".to_owned() + "\n\t" + USAGE);
			exit(0);
		}
	}
	let verbose = cli_args.iter().any(|a| a == "-v" || a == "--verbose");

	// Try to get config/events paths from CLI args/env
	let mut config_path: Option<String> = None;
	if let Some((i, _)) = cli_args.iter().enumerate()
		.find(|a| a.1 == "-c" || a.1 == "--config") {
		if i < cli_args.len() {
			config_path = Some(cli_args[i+1].to_owned());
		}
	} else if let Ok(path) = env::var(CONFIG_PATH_ENV) {
		config_path = Some(path);
	} else if verbose {
		eprintln!("{} is not set, defaulting to {}", CONFIG_PATH_ENV, CONFIG_PATH_DEFAULT);
	}

	let mut events_path: Option<String> = None;
	if let Some((i, _)) = cli_args.iter().enumerate()
		.find(|a| a.1 == "-e" || a.1 == "--events") {
		if i < cli_args.len() {
			events_path = Some(cli_args[i+1].to_owned());
		}
	} else if let Ok(path) = env::var(EVENTS_PATH_ENV) {
		events_path = Some(path);
	} else if verbose {
		eprintln!("{} is not set, defaulting to {}", EVENTS_PATH_ENV, EVENTS_PATH_DEFAULT);
	}

	Args {
		verbose,
		config_path: if let Some(path) = config_path { path } else { CONFIG_PATH_DEFAULT.to_owned() },
		events_path: if let Some(path) = events_path { path } else { EVENTS_PATH_DEFAULT.to_owned() }
	}
}
