#!/bin/bash
cargo sqlx database reset -y --source ./migrations -f && cargo run $1 $2 $3 $4 $5 $6
