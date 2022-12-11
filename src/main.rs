use std::env;

mod config;

fn main() {
	let config_path = env::var("MY_TIMERS_CONFIG")
		.expect("Failed to get config path");
	let conf = config::parse(&config_path);

}
