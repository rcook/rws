.PHONY: default
default: target/debug/rws

Cargo.toml .rbbt_version:
	curl -sS https://gitlab.com/rcook/rbbt/-/raw/v0.4.4/rbbt | bash

target/debug/rws: Cargo.toml .rbbt_version
	cargo build
