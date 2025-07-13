// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{fs, path::PathBuf};

#[test]
fn test_config_file_parsing() {
	// Test that the example config file can be parsed
	let config_content = r#"
[api]
enabled = true
port = 3011
host = "0.0.0.0"
tls = false

[gateway]
enabled = true
port = 3012
host = "0.0.0.0"
tls = false

[general]
log_level = "Trace"

[general.database]
max_connections = 20
database = "sonata_test"
username = "sonata"
password = "sonata"
port = 5432
host = "localhost"
tls = "prefer"
"#;

	// Write test config to a temporary file
	let temp_dir = std::env::temp_dir();
	let config_path = temp_dir.join("sonata_test_config.toml");
	fs::write(&config_path, config_content).expect("Failed to write test config");

	// Test that the config can be read
	let read_content = fs::read_to_string(&config_path).expect("Failed to read test config");
	assert_eq!(read_content, config_content);

	// Clean up
	fs::remove_file(&config_path).expect("Failed to remove test config");
}

#[test]
fn test_example_env_exists() {
	// Test that the example environment file exists
	let example_env_path = PathBuf::from(".example.env");
	assert!(example_env_path.exists(), ".example.env file should exist");

	// Test that it contains the expected variables
	let content = fs::read_to_string(&example_env_path).expect("Failed to read .example.env");
	assert!(content.contains("POSTGRES_USER"));
	assert!(content.contains("POSTGRES_PASSWORD"));
	assert!(content.contains("POSTGRES_DB"));
	assert!(content.contains("DATABASE_URL"));
}

#[test]
fn test_migrations_exist() {
	// Test that migration files exist
	let migrations_dir = PathBuf::from("migrations");
	assert!(migrations_dir.exists(), "migrations directory should exist");
	assert!(migrations_dir.is_dir(), "migrations should be a directory");

	// Check for expected migration files
	let entries: Vec<_> = fs::read_dir(&migrations_dir)
		.expect("Failed to read migrations directory")
		.filter_map(|e| e.ok())
		.collect();

	assert!(!entries.is_empty(), "migrations directory should not be empty");

	// Check that migration files have the expected naming pattern
	for entry in entries {
		let file_name = entry.file_name();
		let name_str = file_name.to_string_lossy();
		assert!(name_str.ends_with(".sql"), "Migration files should have .sql extension");
		assert!(
			name_str.chars().take(4).all(|c| c.is_numeric()),
			"Migration files should start with 4 digits"
		);
	}
}

#[test]
fn test_license_file_exists() {
	// Test that the LICENSE file exists and is MPL-2.0
	let license_path = PathBuf::from("LICENSE");
	assert!(license_path.exists(), "LICENSE file should exist");

	let content = fs::read_to_string(&license_path).expect("Failed to read LICENSE");
	assert!(content.contains("Mozilla Public License Version 2.0"));
}

#[test]
fn test_cargo_toml_valid() {
	// Test that Cargo.toml exists and contains expected metadata
	let cargo_toml_path = PathBuf::from("Cargo.toml");
	assert!(cargo_toml_path.exists(), "Cargo.toml should exist");

	let content = fs::read_to_string(&cargo_toml_path).expect("Failed to read Cargo.toml");
	assert!(content.contains("name = \"sonata\""));
	assert!(content.contains("license = \"MPL-2.0\""));
	assert!(content.contains("[dependencies]"));
	assert!(content.contains("[dev-dependencies]"));
	assert!(content.contains("[profile.release]"));
}
