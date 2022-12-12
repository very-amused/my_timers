use std::{str::{FromStr, Split}, error::Error, num::ParseIntError, fmt::{Display, self}};

use lazy_static::lazy_static;
use regex::Regex;

pub struct CronInterval {
	minute: CronValue,
	hour: CronValue,
	day: CronValue, // day of month
	month: CronValue,
	weekday: CronValue,
	pub startup: bool // Whether the interval should fire immediately when my_timers starts
}

#[derive(Debug, Clone, Copy)]
struct CronRange (u32, u32);
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

impl CronInterval {
	// Validation

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

impl FromStr for CronInterval {
	type Err = CronParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		// To start parsing off simple, it's good to make sure the right amount of cron values are present
		const CRON_LEN: usize = 5;
		let values: Vec<&str> = s.splitn(CRON_LEN, ' ').collect();
		if values.len() != CRON_LEN {
			return Err(Self::Err::SyntaxError(
				format!("{} - unexpected number of cron values (expected {}, received {})", s, CRON_LEN, values.len())
			));
		}
		// Next, parse and validate each value according to its expected range

		todo!()
	}
}

#[derive(Debug, Clone)]
pub enum CronValue {
	Every, // Most common cron value, parsed from an asterisk
	Value(u32), // Single number
	Set(Vec<u32>), // Comma-separated values
	Range((u32, u32)), // Range (start, stop)
}

#[derive(Debug)]
pub enum CronParseError {
	MalformedTokens(String),
	ParseIntError(ParseIntError),
	OutOfRange(Box<CronValue>, Box<CronRange>),
	SyntaxError(String)
}

impl Display for CronParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::MalformedTokens(e) => write!(f, "Malformed tokens: {}", e),
			Self::ParseIntError(e) => e.fmt(f),
			Self::OutOfRange(value, range) => {
				Self::SyntaxError(format!("value {:?} exceeds field range {:?}", value, range)).fmt(f)
			},
			Self::SyntaxError(e) => write!(f, "Invalid cron syntax: {}", e)
		}
	}
}

impl Error for CronParseError {}

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
