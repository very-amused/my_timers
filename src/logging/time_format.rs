use tracing_subscriber::fmt::time::LocalTime;
use time::format_description::FormatItem;
use time::macros::format_description;

pub type Time = LocalTime<&'static [FormatItem<'static>]>;

// Create a timer using my_timers' timestamp formatting.
pub fn timer() -> Time {
	LocalTime::new(format_description!("[day] [month repr:short] [year] [hour]:[minute]:[second].[subsecond digits:3]"))
}
