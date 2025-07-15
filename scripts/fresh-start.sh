#!/bin/bash
cargo sqlx database reset -y --source ./migrations -f && cargo run
