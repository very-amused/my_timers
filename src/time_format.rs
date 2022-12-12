use tracing_subscriber::fmt::time::LocalTime;
use time::format_description::FormatItem;
use time::macros::format_description;

// Create a timer using my_timers' timestamp formatting.
pub fn timer<'a>() -> LocalTime<&'a [FormatItem<'static>]> {
	LocalTime::new(format_description!("[day] [month repr:short] [year] [hour]:[minute]:[second].[subsecond digits:3]"))
}
