FROM rust:bookworm

RUN rustup toolchain install stable
RUN rustup default stable
RUN rustup target add aarch64-unknown-linux-gnu

RUN dpkg --add-architecture arm64
RUN apt update && apt install -y libssl-dev openssl gcc-aarch64-linux-gnu libc6-dev:arm64

ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PKG_CONFIG_PATH=/tmp/aarch64/usr/share/pkgconfig
ENV PKG_CONFIG_LIBDIR=/tmp/aarch64/usr/lib/aarch64-linux-gnu/pkgconfig/
ENV PKG_CONFIG_SYSROOT_DIR=/tmp/aarch64/
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/aarch64-linux-gnu-gcc
ENV RUSTFLAGS="-C link-arg=--sysroot=/tmp/aarch64"

WORKDIR /project
CMD cargo build --target aarch64-unknown-linux-gnu --release
