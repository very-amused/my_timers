use std::error::Error;
use std::fmt::Display;
use std::num::ParseIntError;

use super::CronValue;
use super::parsing::CronRange;

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
