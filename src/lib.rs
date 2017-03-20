//! `slog-config` allows logging configuration based on text description in TOML
//! format, typically loaded from file.
//!
//! The resulting configuration is returned as a slog-rs `Drain`.

#![feature(custom_derive)]
#![warn(missing_docs)]

extern crate isatty;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

extern crate slog;
extern crate slog_json;
extern crate slog_term;

use std::result::Result;
use std::fs::OpenOptions;
use std::sync::Mutex;
use std::panic::{RefUnwindSafe, UnwindSafe};

/// Type of Drain returned by this library
pub type Drain = Box<SendSyncDrain<Ok=(), Err=Box<std::error::Error>>>;

/// Bounds of `Drain` returned by this crate
pub trait SendSyncDrain: slog::Drain + Send + Sync + RefUnwindSafe + UnwindSafe {}

impl<T> SendSyncDrain for T
    where T: slog::Drain + Send + Sync + RefUnwindSafe + UnwindSafe + ?Sized
{
}

/// Configuration serialization format datastructeres
pub mod config {
    use std::collections::BTreeMap;

    #[derive(Deserialize, Debug)]
    /// Logging configuration
    pub struct Config {
        /// Outputs
        pub output : BTreeMap<String, Output>,
    }

    //#[derive(Deserialize, Debug)]
    /// Output configuration
    pub type Output = BTreeMap<String, String>;
}

include!("_file.rs");
include!("_term.rs");

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
pub fn all_factories() -> Vec<Box<DrainFactory>> {
    vec![Box::new(FileDrainFactory), Box::new(TermDrainFactory)]
}

/// Produce a `Drain` described by the the given `config_str`
///
/// `all_factories` will be used.
pub fn from_config(config_str : &str) -> Result<Drain, String> {
    from_config_with(config_str,  &all_factories())
}

/// `Drain` that logs to multiple sub-`Drain`s
struct DuplicateMultiple {
    drains : Vec<Drain>,
}

impl slog::Drain for DuplicateMultiple {

    type Ok = ();
    type Err = Box<std::error::Error>;

    fn log(&self, info: &slog::Record, kv : &slog::OwnedKVList) -> std::result::Result<(), Self::Err> {
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
where D : slog::Drain<Ok=(), Err=E>,
      E : std::error::Error+'static {
          type Ok =();
    type Err = Box<std::error::Error>;

    fn log(&self, info: &slog::Record, kv : &slog::OwnedKVList) -> std::result::Result<(), Self::Err> {
        self.0.log(info, kv).map_err(|e| Box::new(e) as  Box<std::error::Error>)
    }
}


/// Produce a `Drain` in the given `io`
///
/// Unlike `from_config` this allows manually specifing the list
/// of `DrainFactory`-ies to querry.
pub fn from_config_with(config_str : &str, factories : &[Box<DrainFactory>]) -> Result<Drain, String> {

    let config : config::Config = toml::from_str(config_str)
        .map_err(|e| format!("couldn't decode configuration: {}", e))?;

    let mut sub_drains = vec![];

    for output in &config.output {
        let mut drain_from_factory = None;
        for factory in factories {
            match factory.from_config(output.1)? {
                Some(drain) => {
                    drain_from_factory = Some(drain);
                    break;
                },
                None => {},
            }
        }
        let drain_from_factory = drain_from_factory.ok_or(format!("no backend implementing output {} found", output.0))?;
        sub_drains.push(drain_from_factory);
    }

    Ok(Box::new(DuplicateMultiple{drains: sub_drains}))
}

#[cfg(test)]
mod tests {
}
