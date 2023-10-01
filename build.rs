use std::{fs, io};

fn main() -> io::Result<()> {
    let nsis_version_def = format!("!define DISPLAY_VERSION {}", env!("CARGO_PKG_VERSION"));
		fs::write("installer/version.nsh", &nsis_version_def)
}
