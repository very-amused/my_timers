pub struct CronInterval {
	minute: CronValue,
	hour: CronValue,
	day: CronValue, // day of month
	month: CronValue,
	weekday: CronValue,
	pub startup: bool // Whether the interval should fire immediately when my_timers starts
}

type AllowedRange = (u32, u32);
impl CronInterval {

	const fn minute_allowed() -> AllowedRange {
		(0, 59)
	}
	const fn hour_allowed() -> AllowedRange {
		(0, 23)
	}
	const fn day_allowed() -> AllowedRange {
		(1, 31)
	}
	const fn month_allowed() -> AllowedRange {
		(1, 12)
	}
	const fn weekday_allowed() -> AllowedRange {
		(0, 7)
	}
}

pub enum CronValue {
	Every, // Most common cron value, parsed from an asterisk
	Value(u32), // Single number
	Set(Vec<u32>), // Comma-separated values
	Range((u32, u32)), // Range (start, stop)
	SpecialValue(CronSpecialValue)
}

pub enum CronSpecialValue {
	Yearly,
	Monthly,
	Weekly,
	Daily,
	Hourly,
	EveryMinute,
	EverySecond
}

