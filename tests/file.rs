#[macro_use]
extern crate slog;
extern crate slog_config;

use std::{fs, io, thread, time};
use std::io::Read;
use slog::{Drain, Logger};

use std::path::Path;

fn read_to_string(path : &str) -> io::Result<String> {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .open(Path::new(path))?;

    let mut s = String::new();

    let _= file.read_to_string(&mut s)?;

    Ok(s)
}


#[test]
fn file_json() {

    let _ = fs::remove_file("target/file-json.log");

    let config = read_to_string("tests/file-json.toml").unwrap();

    let drain = slog_config::from_config(&config).unwrap();

    let logger = Logger::root(drain.fuse(), o!("test" => "file_json"));

    info!(logger, "test complete");

    // TODO: This is lame and unreliable to wait for the log to actually get written
    thread::sleep(time::Duration::from_millis(1000));

    let log = read_to_string("target/file-json.log").unwrap();

    // TODO: Britle. Parsing json and comparing kv-s would be more reliable
    assert_eq!(log, "{\"test\":\"file_json\"}\n");
}
