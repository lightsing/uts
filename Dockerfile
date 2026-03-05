FROM rust:1-trixie AS rust-2026-01-01-zig-0-15-2-chef-0-1-77-zigbuild-0-22-1

RUN cd /tmp && \
    curl -L -o zig.tar.xz https://ziglang.org/download/0.15.2/zig-x86_64-linux-0.15.2.tar.xz && \
    tar -xJf zig.tar.xz -C /usr/local --strip-components=1 && \
    rm zig.tar.xz

ENV PATH="/usr/local:${PATH}"

RUN rustup install nightly-2026-01-01 && \
    rustup default nightly-2026-01-01 && \
    rustup target add x86_64-unknown-linux-gnu && \
    cargo install cargo-chef --version 0.1.77 --locked && \
    cargo install cargo-zigbuild --version 0.22.1 --locked

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    xz-utils \
    libclang-dev \
    libssl-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*


FROM rust-2026-01-01-zig-0-15-2-chef-0-1-77-zigbuild-0-22-1 AS rust

FROM rust AS planner

WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust AS builder

ENV RUSTFLAGS="-C target-cpu=x86-64"

WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --workspace --release --recipe-path recipe.json --zigbuild --target x86_64-unknown-linux-gnu.2.17
COPY . .

ARG BINARY_NAME
RUN cargo zigbuild --release --bin ${BINARY_NAME} --target x86_64-unknown-linux-gnu.2.17

FROM debian:trixie-slim AS runtime

WORKDIR /app
ARG BINARY_NAME

COPY --from=builder /app/target/x86_64-unknown-linux-gnu/${BINARY_NAME} /app/app

ENTRYPOINT ["/app/app"]
# docker build -t uts-calendar:latest --build-arg BINARY_NAME=uts-calendar -f Dockerfile .
