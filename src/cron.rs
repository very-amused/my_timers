pub mod parsing;
pub mod error;

#[derive(Debug)]
pub struct CronInterval {
	minute: CronValue,
	hour: CronValue,
	day: CronValue, // day of month
	month: CronValue,
	weekday: CronValue,
	pub startup: bool // Whether the interval should fire immediately when my_timers starts
}

#[derive(Debug, Clone)]
pub enum CronValue {
	Every, // Most common cron value, parsed from an asterisk
	Value(u32), // Single number
	Set(Vec<u32>), // Comma-separated values
	Range((u32, u32)), // Range (start, stop)
}
