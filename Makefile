run:
	cargo run --release

debug:
	cargo run

test:
	cargo test

clean:
	cargo clean

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

all-checks: clippy fmt-check test
