// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

/*!
 * # sonata
 *
 * A robust, performant polyproto home server.
 */

use std::{path::PathBuf, process::exit, str::FromStr};

use clap::Parser;
use log::{LevelFilter, debug, error, info, trace};
use polyproto::signature::Signature;
use sqlx::query_scalar;

/// The maximum password length this server allows. Passwords longer than this
/// will not be hashed or processed at all, and will result in a `400` status
/// code for the user.
pub(crate) const MAX_PERMITTED_PASSWORD_LEN: usize = 128;

/// Module housing the HTTP API routes and functionality
mod api;
/// Module hosting logic for the sonata CLI
pub(crate) mod cli;
/// Module for parsing and interpreting the sonata config file.
pub(crate) mod config;
/// Implementations of the required cryptographic traits of polyproto-rs. WE ARE
/// **NOT** DOING OUR OWN CRYPTO.
pub(crate) mod crypto;
/// Module defining PostgreSQL database entities as Rust structs and providing
/// CRUD functionality
pub(crate) mod database;

/// Finer-grained error types for sonata.
pub(crate) mod errors;
/// Module housing the WebSocket Gateway logic
mod gateway;

pub(crate) use crate::errors::{StdError, StdResult};
use crate::{
	crypto::ed25519::DigitalSignature,
	database::{
		Issuer,
		algorithm_identifier::AlgorithmIdentifier,
		api_keys::{self, ApiKey},
		tokens::TokenStore,
	},
};

#[tokio::main]
#[cfg_attr(coverage_nightly, coverage(off))]
async fn main() -> StdResult<()> {
	use crate::{cli::Args, config::SonataConfig, database::Database};
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

	info!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
	info!("Build Timestamp: {}", env!("VERGEN_BUILD_TIMESTAMP"));

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

	debug!("Connecting to the database...");
	let database =
		match Database::connect_with_config(&SonataConfig::get_or_panic().general.database).await {
			Ok(db) => db,
			Err(e) => exit_with_log(3, &format!("Couldn't connect to the database: {e}")),
		};
	debug!("Connected to database!");
	debug!("Applying migrations...");
	match database.run_migrations().await {
		Ok(_) => debug!("Migrations applied!"),
		Err(e) => exit_with_log(4, &format!("Couldn't apply migrations: {e}")),
	};
	let keys_in_table =
		query_scalar!("SELECT COUNT(*) FROM api_keys").fetch_one(&database.pool).await?;
	match keys_in_table {
		Some(0) | None => {
			let api_key =
				api_keys::add_api_key_to_database(&ApiKey::new_random(&mut rand::rng()), &database)
					.await
					.map_err(|_| String::from("Error adding API key to database}"))?;
			info!("Added an API key to the database, since none were available: {api_key}");
			info!("Save this API key, as it will not be shown again on future starts.");
		}
		_ => (),
	};
	debug!("Inserting known algorithm identifiers into algorithm_identifiers table...");
	match AlgorithmIdentifier::try_insert(
		&database,
		&DigitalSignature::algorithm_identifier().oid,
		Some("Edwards-curve Digital Signature Algorithm (EdDSA) Ed25519"),
		Default::default(),
	)
	.await
	{
		Ok(a_id) => debug!(
			"Inserted algorithm_identifier {} {}",
			a_id.algorithm_identifier,
			a_id.common_name.unwrap_or_default()
		),
		Err(e) => match e.code {
			errors::Errcode::Duplicate => {
				debug!("Algorithm identifier already present, nothing changed")
			}
			_ => error!("Could not manipulate database: {e:?}"),
		},
	};
	debug!("Inserting own issuer domain name into the database...");
	match Issuer::create_own(&database).await {
		Ok(i) => match i {
			Some(issuer) => {
				debug!(r#"Inserted own issuer "{}" into the database!"#, issuer.domain_components)
			}
			None => debug!("Issuer entry already present, nothing changed"),
		},
		Err(e) => {
			error!("Could not manipulate database: {e:?}");
			exit(5)
		}
	}

	let token_store = TokenStore::new(database.clone());

	let tasks = vec![api::start_api(
		SonataConfig::get_or_panic().api.clone(),
		database.clone(),
		token_store.clone(),
	)];

	for task in tasks.into_iter() {
		task.await.unwrap()
	}

	Ok(())
}

/// Exits the program with a given status code, printing a log message
/// beforehand.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn exit_with_log(code: i32, message: &str) -> ! {
	error!("{message}");
	error!("Exiting due to previous error.");
	std::process::exit(code)
}
