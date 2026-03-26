.PHONY: all build build-frontend build-server build-updater dev dev-frontend dev-server clean typegen lint

all: build

build: build-frontend build-server build-updater

build-frontend:
	cd frontend && npm install && npm run build

build-server:
	cargo build --release -p kabu-server

build-updater:
	cargo build --release -p kabu-updater

dev-frontend:
	cd frontend && npm run dev

dev-server:
	cargo run -p kabu-server

dev: build-frontend dev-server

clean:
	cargo clean
	rm -rf frontend/dist frontend/node_modules

typegen:
	cargo run -p kabu-typegen

lint:
	cd frontend && npm run lint
