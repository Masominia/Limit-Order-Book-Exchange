.PHONY: install build test lint run all

install:
	pip install -e ".[dev]"

build:
	cargo build --release --manifest-path engine/Cargo.toml

lint:
	ruff check src tests scripts
	ruff format --check src tests scripts

test: build
	cargo test --release --manifest-path engine/Cargo.toml
	pytest

run: build
	python -m lobsim run

all: lint test run
