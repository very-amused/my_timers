use chrono::{DateTime, Local, Timelike, Datelike};

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

impl CronInterval {
	pub fn match_time(&self, now: &DateTime<Local>) -> bool {
		self.minute.compare(now.minute()) &&
		self.hour.compare(now.hour()) &&
		// See https://crontab.guru/cron-bug.html
		if self.day == CronValue::Every || self.weekday == CronValue::Every { // If date or weekday is *, evaluate them as an intersection
			self.day.compare(now.day()) &&
			(
				self.weekday.compare(now.weekday().number_from_monday() % 7) || // 0 == Sunday
				self.weekday.compare(now.weekday().number_from_monday()) // 7 == Sunday
			)
		} else { // If neither date or weekday is *, evaluate them as a union
			self.day.compare(now.day()) ||
			self.weekday.compare(now.weekday().number_from_monday() % 7) ||
			self.weekday.compare(now.weekday().number_from_monday())
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum CronValue {
	Every, // Most common cron value, parsed from an asterisk
	Value(u32), // Single number
	Set(Vec<u32>), // Comma-separated values
	Range((u32, u32)), // Range (start, stop)
}

impl CronValue {
	// Compare a time value with a cron field
	fn compare(&self, value: u32) -> bool {
		match self {
			Self::Every => true,
			Self::Value(n) => &value == n,
			Self::Set(set) => set.binary_search(&value).is_ok(),
			Self::Range((start, end)) => &value >= start && &value <= end
		}
	}
}
