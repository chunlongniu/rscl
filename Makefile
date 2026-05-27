CARGO_BIN := $(shell echo "$${CARGO_HOME:-$$HOME/.cargo}/bin")

.PHONY: build install uninstall preview

build:
	cargo build --release

preview:
	cargo clippy -- -D warnings

install: build
	@cp target/release/rscl $(CARGO_BIN)/rscl
	@echo "rscl installed to $(CARGO_BIN)/rscl"

uninstall:
	@rm -f $(CARGO_BIN)/rscl
	@echo "rscl removed"
