use std::{fs, error::Error, process::Command};
use chrono::Local;

fn main() -> Result<(), Box<dyn Error>> {
	// Provide git commit hash
	println!("cargo:rustc-rerun-if-changed=.git/HEAD");
	let git_head = fs::read_to_string(".git/HEAD")?;
	if git_head.starts_with("ref:") {
		if let Some(head_path) = git_head.split_ascii_whitespace().last() {
			if head_path.starts_with("refs") {
				println!("cargo:rustc-rerun-if-changed=.git/{}", head_path);
			}
		}
	}
	let cmd = Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output()?;	
	let commit_hash = String::from_utf8(cmd.stdout)?;
	println!("cargo:rustc-env=COMMIT_HASH={}", commit_hash);

	// Provide build date
	let build_date = Local::now().format("%Y-%m-%d").to_string();
	println!("cargo:rustc-env=BUILD_DATE={}", build_date);


	// Write NSIS version header
	let nsis_version_def = format!("!define DISPLAY_VERSION {}", env!("CARGO_PKG_VERSION"));
	fs::write("installer/version.nsh", &nsis_version_def)?;

	Ok(())
}
