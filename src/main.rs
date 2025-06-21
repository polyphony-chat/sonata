// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/*!
 * # sonata
 *
 * A robust, performant polyproto home server.
 */

use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use log::{LevelFilter, debug, error, trace};

mod api;
pub(crate) mod cli;
pub(crate) mod config;
pub(crate) mod database;
mod gateway;

pub(crate) type StdError = Box<dyn std::error::Error + 'static>;
pub(crate) type StdResult<T> = Result<T, StdError>;

#[tokio::main]
#[cfg(not(tarpaulin))]
async fn main() -> StdResult<()> {
    use crate::cli::Args;
    use crate::config::SonataConfig;
    _ = Args::parse(); // Has to be done, else clap doesn't work correctly.
    Args::init_global()?;
    let verbose_level = match Args::get_or_panic().verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        2 => LevelFilter::Trace,
        _ => {
            println!(
                r#"Woah there! You don't need to supply a bajillion "-v"'s. 2 is the limit! Interpreting input as "verbose"."#
            );
            LevelFilter::Trace
        }
    };
    let log_level = match Args::get_or_panic().quiet {
        0 => verbose_level,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Error,
        3 => LevelFilter::Off,
        _ => {
            println!(
                r#"Woah there! You don't need to supply a bajillion "-q"'s. 3 is the limit! Interpreting input as "off""#
            );
            LevelFilter::Trace
        }
    };
    env_logger::Builder::new()
        .filter(None, LevelFilter::Off)
        .filter(Some("sonata"), log_level)
        .try_init()?;
    debug!("Hello, world!");
    let config_location = match &Args::get_or_panic().config {
        Some(path) => path,
        None => &PathBuf::from_str("sonata.toml")?,
    };

    debug!("Parsing config at {config_location:?}...");
    SonataConfig::init(&match std::fs::read_to_string(config_location) {
        Ok(string) => string,
        Err(_) => {
            exit_with_log(
                1,
                &format!(
                    r#"Couldn't find a file at "{}". Are you sure that the path is correct and that the file is accessible?"#,
                    config_location.to_string_lossy()
                ),
            );
        }
    })?;
    debug!("Parsed config!");
    trace!("Read config {:#?}", SonataConfig::get_or_panic());

    Ok(())
}

/// Exits the program with a given status code, printing a log message beforehand.
pub fn exit_with_log(code: i32, message: &str) -> ! {
    error!("{message}");
    error!("Exiting due to previous error.");
    std::process::exit(code)
}
