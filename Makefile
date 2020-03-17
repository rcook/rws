.PHONY: default
default: target/debug/rws

Cargo.toml .rbbt_version:
	curl -sS https://gitlab.com/rcook/rbbt/-/raw/stable/rbbt | bash

target/debug/rws: Cargo.toml .rbbt_version
	cargo build
