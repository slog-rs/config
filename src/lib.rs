//! `slog-config` allows logging configuration based on text description in TOML
//! format, typically loaded from file.
//!
//! The resulting configuration is returned as a slog-rs `Drain`.

#![feature(custom_derive)]
#![warn(missing_docs)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

extern crate slog;
extern crate slog_stream;
extern crate slog_json;
extern crate slog_term;

use std::result::Result;
use std::fs::OpenOptions;

/// Type of Drain returned by this library
pub type Drain = Box<slog::Drain<Error=Box<std::error::Error>>>;

pub mod config {
    use std::collections::BTreeMap;

    #[derive(Deserialize)]
    pub struct Config {
        pub output : BTreeMap<String, Output>,
    }

    #[derive(Deserialize)]
    pub struct Output {
        pub values : BTreeMap<String, String>,
    }
}

/// `Drain` type handler
///
/// This is an interface between `slog-config` and handler of particular type of the drain that can
/// be described in the config.
///
/// `slog-config` builds a `Drain` by quering each element of list of `DrainFactory`-ies.
///
/// By implementing this type external code can be used to extend possible interpret a piece of config and produce
/// each piece of a resulting `Drain`.
///
/// See `from_config_with`
pub trait DrainFactory {
    /// Attempt to produce a `Drain` out of `Output` config description
    ///
    /// Returns:
    ///
    /// * `Ok(Box<Drain>)` if succesfully built a `Drain`
    /// * `Ok(None)` if output type does not match
    /// * `Err(String)` on error
    fn from_config(&self, config : &config::Output) -> Result<Option<Drain>, String>;
}

/// List of all `DrainFactory`-ies currently implemented by `slog-config`
///
/// Note that adding a `Factory` to `ALL_FACTORIES` is not considering a breaking
/// change in SemVer.
pub static ALL_FACTORIES : [Box<DrainFactory+Sync>; 0] = [];

pub fn all_factories() -> Vec<Box<DrainFactory>> {
    vec![]
}

/// Produce a `Drain` described by the the given `config_str`
///
/// `ALL_FACTORIES` will be used.
pub fn from_config(config_str : &str) -> Result<Drain, String> {
    from_config_with(config_str,  &all_factories())
}

/// `Drain` that logs to multiple sub-`Drain`s
struct DuplicateMultiple {
    drains : Vec<Drain>,
}

impl slog::Drain for DuplicateMultiple {

    type Error = Box<std::error::Error>;

    fn log(&self, info: &slog::Record, kv : &slog::OwnedKeyValueList) -> std::result::Result<(), Self::Error> {
        for d in &self.drains {
            d.log(info, kv)?
        }
        Ok(())
    }
}

/// Wrap another `slog::Drain` and box error returned by it;
struct BoxErrorDrain<D>(D)
where D : slog::Drain;

impl<D, E> slog::Drain for BoxErrorDrain<D>
where D : slog::Drain<Error=E>,
      E : std::error::Error+'static {
    type Error = Box<std::error::Error>;

    fn log(&self, info: &slog::Record, kv : &slog::OwnedKeyValueList) -> std::result::Result<(), Self::Error> {
        self.0.log(info, kv).map_err(|e| Box::new(e) as  Box<std::error::Error>)
    }
}


/// Produce a `Drain` in the given `io`
///
/// Unlike `from_config` this allows manually specifing the list
/// of `DrainFactory`-ies to querry.
pub fn from_config_with(config_str : &str, factories : &[Box<DrainFactory>]) -> Result<Drain, String> {

    let config : config::Config = toml::decode_str(config_str).ok_or("couldn't decode configuration")?;

    let mut sub_drains = vec![];

    for output in &config.output {
        for factory in factories {
            match factory.from_config(output.1)? {
                Some(drain) => {
                    sub_drains.push(drain);
                    break;
                },
                None => {},
            }
        }
    }

    Ok(Box::new(DuplicateMultiple{drains: sub_drains}))
}

struct FileDrainFactory;
impl DrainFactory for FileDrainFactory {
    fn from_config(&self, config : &config::Output) -> Result<Option<Drain>, String> {
        let type_ = config.values.get("type").ok_or("output type missing")?;


        if type_ != "file" {
            return Ok(None)
        }
        let path = config.values.get("path").ok_or("file path missing")?;
        let format_str = config.values.get("format").ok_or("format missing")?;

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path).unwrap();

        let format = match format_str.as_str() {
            "json" => slog_json::Format::new().build(),
            _ => return Err(format!("unkown file format: {}", format_str)),
        };
        let drain = slog_stream::stream(file, format);

        Ok(Some(Box::new(BoxErrorDrain(drain))))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
