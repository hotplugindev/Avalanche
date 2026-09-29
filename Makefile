.PHONY: build test lint fmt fmt-check clippy check frontend-install frontend-dev frontend-build frontend-typecheck

build:
	cargo build --workspace

test:
	cargo test --workspace

clippy:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

check: fmt-check clippy test

frontend-install:
	cd apps/desktop/frontend && npm install

frontend-dev:
	cd apps/desktop/frontend && npm run dev

frontend-build:
	cd apps/desktop/frontend && npm run build

frontend-typecheck:
	cd apps/desktop/frontend && npm run typecheck
