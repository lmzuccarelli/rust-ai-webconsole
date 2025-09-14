.PHONY: all clean-all build build-debug build-cross create-configs deploy start-service restart-service stop-service

all: clean-all build

build-debug: 
	cargo build

build: 
	cargo build --release

build-cross: 
	cross build --target aarch64-unknown-linux-gnu --release

clean-all:
	rm -rf cargo-test*
	cargo clean
	rm -rf ./target/debug

create-configs:
	./scripts/infrastructure.sh create_configs

deploy: build
	./scripts/infrastructure.sh deploy_service

start-service:
	./scripts/infrastructure.sh start_service

restart-service:
	./scripts/infrastructure.sh restart_service

stop-service:
	./scripts/infrastructure.sh stop_service

