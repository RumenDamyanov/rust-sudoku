SHELL := /bin/bash
BINARY := sudoku-server
CLIBIN := sudoku-cli
OUT := bin

.PHONY: all fmt check test cover build run cli clean docker-build docker-run docker-push

all: fmt check test

fmt:
	cargo fmt --all

check:
	cargo clippy --all-targets --all-features -- -D warnings

# Runs tests with all features
TEST_FLAGS ?= --all-features

test:
	cargo test $(TEST_FLAGS)

cover: test
	@echo "Run: cargo install cargo-tarpaulin && cargo tarpaulin --all-features"

VERSION ?= $(shell git describe --tags --always --dirty 2>/dev/null || echo dev)
COMMIT ?= $(shell git rev-parse --short HEAD 2>/dev/null || echo none)
DATE ?= $(shell date -u +%Y-%m-%dT%H:%M:%SZ)

build:
	mkdir -p $(OUT)
	cargo build --release --features server --bin sudoku-server
	cargo build --release --features cli --bin sudoku-cli
	cp target/release/sudoku-server $(OUT)/$(BINARY)
	cp target/release/sudoku-cli $(OUT)/$(CLIBIN)

run:
	cargo run --features server --bin sudoku-server

cli:
	cargo run --features cli --bin sudoku-cli

clean:
	rm -rf $(OUT)
	cargo clean

bench:
	cargo bench --all-features

IMAGE ?= ghcr.io/rumendamyanov/rust-sudoku:latest

docker-build:
	docker build -t $(IMAGE) .

docker-run:
	docker run --rm -p 8080:8080 -e PORT=8080 $(IMAGE)

docker-push:
	docker push $(IMAGE)
