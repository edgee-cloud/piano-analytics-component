.PHONY: all
MAKEFLAGS += --silent

all: help

help:
	@grep -E '^[a-zA-Z1-9\._-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| sort \
		| sed -e "s/^Makefile://" -e "s///" \
		| awk 'BEGIN { FS = ":.*?## " }; { printf "\033[36m%-30s\033[0m %s\n", $$1, $$2 }'

wit-deps: ## Install edgee wit
	wit-deps

build: ## Build the wasi component
	cargo build --target wasm32-wasip2 --release
	cp ./target/wasm32-wasip2/release/piano_analytics_component.wasm piano_analytics.wasm

test: ## Test the component on host platform
	cargo test --lib

test.coverage:
	cargo llvm-cov --all-features

test.coverage.html:
	cargo llvm-cov --all-features --open