use std::{error::Error, time::Duration};
use chrono::{Timelike, Local};
use tokio::{time, task::JoinSet, signal as tokio_signal};
use tracing::{event, Level, span, Instrument, instrument};

mod config;
mod db;
mod logging;
mod events;
mod cron;
mod args;
mod signal;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	// Parse CLI args
	let args = args::args();

	// Parse config
	let config = config::parse(&args.config_path).or_else(|err| {
		eprintln!("Failed to parse {}:", &args.config_path);
		Err(err)
	})?;


	// Initialize logging destinations
	let _guards = config.log.init(args.verbose);
	event!(Level::INFO, "my_timers started");

	// Connect to DB
	let opts = config.db.mysql_opts();
	let pool = mysql_async::Pool::new(opts);
	{
		let span = span!(Level::DEBUG, "Connecting to DB");
		async {
			event!(Level::DEBUG, "Connecting to database {}", config.db.pretty_name());
			pool.get_conn().await?;
			event!(Level::DEBUG, "Connected");
			Ok::<(), mysql_async::Error>(())
		}.instrument(span).await?;
	}

	// Listen for ctrl+c/SIGINT to safely shutdown.
	// tokio::select! must be used to catch signals for all future awaits
	// on the main thread
	let ctrl_c = tokio_signal::ctrl_c();
	let mut sigterm_channel = signal::new(signal::SignalKind::SIGTERM).or_else(|err| {
		eprintln!("Failed to create SIGTERM channel");
		Err(err)
	})?;
	let sigterm = sigterm_channel.recv();
	tokio::pin!(ctrl_c);
	tokio::pin!(sigterm);

	// Read events from config
	let events = tokio::select! {
		evts = events::parse(&args.events_path, pool.clone()) => evts.or_else(|err| {
			eprintln!("Failed to parse {}:", &args.events_path);
			Err(err)
		})?,
		Ok(_) = &mut ctrl_c => return shutdown(None, pool).await,
		Some(_) = &mut sigterm => return shutdown(None, pool).await
	};


	// Initialize task joinset
	let mut event_threads = JoinSet::<()>::new();

	// Immediately run @startup events
	event!(Level::INFO, "Running @startup events");
	for evt in &events {
		if evt.interval.startup {
			let pool = pool.clone();
			let evt = unsafe { (&**evt as *const events::Event).as_ref() }.unwrap();
			event_threads.spawn(async move {
				evt.run(pool).await.ok();
			});
		}
	}

	// Wait until next minute at 00 seconds to start event loop
	let mut interval = {
		let mut to_minute = time::interval(to_next_minute());
		to_minute.tick().await;
		event!(Level::INFO, "Starting event loop in {:?}", to_minute.period());
		tokio::select! {
			_ = to_minute.tick() => {
				// Start minute interval ticker
				time::interval(Duration::from_secs(60))
			},
			Ok(_) = &mut ctrl_c => return shutdown(Some(event_threads), pool).await,
			Some(_) = &mut sigterm => return shutdown(Some(event_threads), pool).await
		}
	};
	event!(parent: None, Level::INFO, "Starting event loop");

	// Event loop
	loop {
		// Wait for the next minute, breaking the loop if a signal is caught
		tokio::select! {
			_ = interval.tick() => {},
			Ok(_) = &mut ctrl_c => return shutdown(Some(event_threads), pool).await,
			Some(_) = &mut sigterm => return shutdown(Some(event_threads), pool).await
		}
		// Iterate through each event, run the ones that match
		let now = Local::now();
		for evt in &events {
			if evt.interval.match_time(&now) {
				let pool = pool.clone();
				let evt = unsafe { (&**evt as *const events::Event).as_ref() }.unwrap();
				event_threads.spawn(async move {
					// Error logging is handled in the event's tracing span
					evt.run(pool).await.ok();
				});	
			}
		}
	}
}

/// Duration until next minute
fn to_next_minute() -> Duration {
	// Ceil to nearest ms + 1
	let mut next = chrono::Utc::now();
	next = if next.minute() == 59 {
		next.with_hour(next.hour()+1).unwrap()
		.with_minute(0).unwrap()
	} else {
		next.with_minute(next.minute()+1).unwrap()
	};
	next = next.with_second(0).unwrap()
		.with_nanosecond(0).unwrap();

	let mut now = chrono::Utc::now(); // Get duration against current time
	// Round up to next ms
	now = now.with_nanosecond(1_000_000 * (now.nanosecond() / 1_000_000) + 2_000_000).unwrap();
	(next - now).to_std().unwrap()
}


/// Safely shutdown the main thread
#[instrument(name = "Shutting down", skip_all, err)]
async fn shutdown(event_threads: Option<JoinSet<()>>, pool: mysql_async::Pool) -> Result<(), Box<dyn Error>> {
	event!(Level::INFO, "Shutting down");
	if let Some(mut threads) = event_threads {
		event!(Level::DEBUG, "Stopping event threads");
		threads.shutdown().await;
	}
	event!(Level::DEBUG, "Disconnecting from database");
	pool.disconnect().await?;
	event!(Level::INFO, "Done");
	Ok(())
}
