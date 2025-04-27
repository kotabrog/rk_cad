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

rkstep:
	@cd rk_step_parser && cargo run --features cli --bin rkstep -- $(ARGS)

rc:
	$(MAKE) rkstep ARGS="write tests/fixtures/cube.step output/cube.step"

all-checks: clippy fmt-check test rkstep rc
