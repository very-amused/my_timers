use std::collections::VecDeque;
use std::str::FromStr;

use super::error::CronParseError;
use super::CronValue;
use super::CronInterval;

#[derive(Debug, Clone, Copy)]
pub struct CronRange (u32, u32);

impl CronInterval {
	const fn minute_range() -> CronRange {
		CronRange(0, 59)
	}
	const fn hour_range() -> CronRange {
		CronRange(0, 23)
	}
	const fn day_range() -> CronRange {
		CronRange(1, 31)
	}
	const fn month_range() -> CronRange {
		CronRange(1, 12)
	}
	const fn weekday_range() -> CronRange {
		CronRange(0, 7)
	}
}

impl CronRange {
	fn validate(&self, v: &CronValue) -> Result<(), CronParseError> {
		// Use closure for validating against range values
		let validate = |n: &u32| -> Result<(), CronParseError> {
			if n > &self.1 || n < &self.0 {
				Err(CronParseError::OutOfRange(Box::new(v.clone()), Box::new(self.clone())))
			} else {
				Ok(())
			}
		};
		// Validate cron values
		match v {
			CronValue::Every => Ok(()),
			CronValue::Value(n) => validate(n),
			CronValue::Range((n1, n2)) => validate(n1).and_then(|_| validate(n2)),
			CronValue::Set(s) => {
				for n in s {
					if let Err(e) = validate(n) {
						return Err(e);
					}
				}
				Ok(())
			}
		}
	}
}

impl FromStr for CronInterval {
	type Err = CronParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		// To start parsing off simple, it's good to make sure the right amount of cron values are present
		const CRON_LEN: usize = 5;
		let mut values: VecDeque<&str> = s.splitn(CRON_LEN + 1, ' ').collect();
		if values.len() < CRON_LEN {
			return Err(Self::Err::SyntaxError(
				format!("{} - unexpected number of cron values (expected {}, received {})", s, CRON_LEN, values.len())
			));
		}
		// Next, parse and validate each value according to its expected range
		macro_rules! next {
			($range:path) => {
				{
					let v: CronValue = values.pop_front().unwrap().parse()?;
					$range().validate(&v)?;
					v
				}
			};
		}
		let interval = CronInterval { // Each value is moved to the interval struct
			minute: next!(Self::minute_range),
			hour: next!(Self::hour_range),
			day: next!(Self::day_range),
			month: next!(Self::month_range),
			weekday: next!(Self::weekday_range),
			// Check for an @startup tag at the end
			startup: values.pop_front() == Some("@startup")
		};
		Ok(interval)
	}
}

impl FromStr for CronValue {
	type Err = CronParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s == "*" { // Parse 'every' value
			Ok(Self::Every)
		} else if s.contains(',') { // Parse set
			let s_values: Vec<&str> = s.split(',').collect();
			let mut values: Vec<u32> = Vec::with_capacity(s_values.len());
			for v in s_values {
				let n: u32 = v.parse().map_err(Self::Err::ParseIntError)?;
				values.push(n);
			}
			// Remove duplicate values
			values.sort();
			values.dedup();
			Ok(Self::Set(values))
		} else if s.contains('-') { // Parse range
			let s_values: Vec<&str> = s.splitn(2, '-').collect();
			if s_values.len() != 2 {
				return Err(Self::Err::MalformedTokens("Invalid cron range".into()));
			}
			let mut values: Vec<u32> = Vec::with_capacity(s_values.len());
			for v in s_values {
				let n: u32 = v.parse().map_err(Self::Err::ParseIntError)?;
				values.push(n);
			}
			Ok(Self::Range((values[0], values[1])))
		} else { // Parse individual value
			let n: u32 = s.parse().map_err(Self::Err::ParseIntError)?;
			Ok(Self::Value(n))
		}
	}
}
