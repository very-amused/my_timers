use std::{error::Error, fs::File, io::{BufReader, BufRead}, collections::VecDeque};

use crate::cron;

// A database event
pub struct Event {
	label: String,
	interval: cron::CronInterval,
	body: Vec<mysql::Statement>
}

fn parse_event(evt_parts: &mut VecDeque<String>) -> Option<Event> {
	todo!();
	let mut evt = Event::default(); 
	// Label
	match evt_parts.pop_front() {
		Some(label) => evt.label = label,
		None => return None
	}
	// Interval
	match evt_parts.pop_front() {
		Some(interval) => {
			todo!()
		},
		None => return None
	}
	

	Some(evt)
}

pub fn parse_events(path: &str) -> Result<Vec<Event>, Box<dyn Error>> {
	// Open file reader
	let file = File::open(path)?;
	let reader = BufReader::new(file);

	// Iterate over lines
	let mut evt_parts: VecDeque<String> = VecDeque::with_capacity(3);
	for l in reader.lines().map(|l| -> String {
		l.unwrap_or("".into())
	}) {
		match evt_parts.len() {
			0 => { // Label (maybe interval)
				let mut l_parts: VecDeque<&str> = l.splitn(2, ':').collect();
				if let Some(label) = l_parts.pop_front() {
					evt_parts.push_back(label.into());
				}
				if let Some(interval) = l_parts.pop_front() {
					evt_parts.push_back(interval.into());
				}
			},
			1 => { // Continue multiline label (maybe interval)
				let mut l_parts: VecDeque<&str> = l.splitn(2, ':').collect();
				if let Some(label) = l_parts.pop_front() {
					evt_parts[0].push(' ');
					evt_parts[0].push_str(label);
				}
				if let Some(interval) = l_parts.pop_front() {
					evt_parts.push_back(interval.into())
				}
			},
			2 => { // Continue/end multiline interval
				// An indented line marks the beginning of an event body
				if l.starts_with('\t') || l.starts_with("  ") {
					evt_parts.push_back(l.into());
				} else {
					evt_parts[1].push_str(&l);
				}
			},
			3 => { // Body
				if l.starts_with('\t') || l.starts_with("  ") {
					evt_parts[2].push('\n');
					evt_parts[2].push_str(&l);
				} else {
					// Parse event
					let evt = parse_event(&mut evt_parts);
				}
			},
			_ => panic!("event parsing dequeue exceeded max size of 3")
		};
	}


	let mut events: Vec<Event> = Vec::new();

	

	todo!()
}