#[macro_use]
extern crate slog;
extern crate slog_config;

use std::{io, fs};
use std::io::Read;
use std::path::Path;

use slog::Logger;


fn read_to_string(path: &str) -> io::Result<String> {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .open(Path::new(path))?;

    let mut s = String::new();

    let _ = file.read_to_string(&mut s)?;

    Ok(s)
}

fn main() {
    let config = read_to_string("examples/term-compact.toml").unwrap();

    let drain = slog_config::from_config(&config).unwrap();
    let logger = Logger::root(slog::Fuse(drain), o!("test" => "term_compact"));

    warn!(logger, "test warning");

    info!(logger, "test complete");
}
