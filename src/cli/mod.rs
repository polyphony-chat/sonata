// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{path::PathBuf, sync::OnceLock};

use clap::Parser;

use crate::StdResult;

/// Module-local global for storing CLI arg values after they have been parsed.
static CLI_ARGUMENTS: OnceLock<Args> = OnceLock::new();

#[derive(Debug, clap::Parser)]
#[command(name = "sonata")]
#[command(version, long_about = None)]
/// `sonata` CLI args
pub struct Args {
    #[arg(short, long, value_name = "FILE")]
    /// Path to a sonata config.toml file. If not specified, will use default
    /// values.
    pub(crate) config: Option<PathBuf>,

    #[arg(short = 'v', long, action = clap::ArgAction::Count)]
    /// Turn on verbose logging. The default log level is "INFO".
    /// Each instance of "v" in "-v" will increase the logging level by one.
    /// Logging levels are DEBUG (-v) and TRACE (-vv).
    /// "Quiet" settings override "verbose" settings. If set, overrides config
    /// value.
    pub(crate) verbose: u8,
    #[arg(short = 'q', long, action = clap::ArgAction::Count)]
    /// Configure "quiet" mode. The default log level is "INFO".
    /// Each instance of "q" in "-q" will decrease the logging level by one.
    /// Logging levels are WARN (-q), ERROR (-qq) and None (completely silent,
    /// except for regular stdout) (-qqq). "Quiet" settings override "verbose"
    /// settings. If set, overrides config value.
    pub(crate) quiet: u8,
}

impl Args {
    #[cfg_attr(coverage_nightly, coverage(off))]
    /// Initialize the global Args storage by parsing the CLI arguments, then
    /// keeping them in memory. Will only yield `Err` after the first call.
    pub fn init_global() -> StdResult<&'static Self> {
        let parsed = Args::try_parse()?;
        CLI_ARGUMENTS.set(parsed).map_err(|_| String::from("cli arguments already parsed"))?;
        Ok(CLI_ARGUMENTS.get().ok_or("cli arguments not set? this should never happen")?)
    }

    /// Get a reference to the parsed CLI args. Will panic, if the CLI args have
    /// not been parsed using `Self::init()` prior to calling this function.
    #[allow(clippy::expect_used)]
    pub fn get_or_panic() -> &'static Self {
        CLI_ARGUMENTS.get().expect("cli arguments should have been set")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "cli arguments should have been set")]
    fn test_get_or_panic_without_init() {
        // This test should panic because CLI_ARGUMENTS hasn't been initialized
        // Note: This test might fail if run after other tests that initialize
        // CLI_ARGUMENTS
        Args::get_or_panic();
    }

    // Note: Testing init_global() and command line parsing would require
    // either mocking or integration tests, as they interact with global state
    // and command line arguments
}
