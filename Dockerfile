FROM rust:1-trixie AS rust-2026-01-01-zig-0-15-2-zigbuild-0-22-1

RUN cd /tmp && \
    curl -L -o zig.tar.xz https://ziglang.org/download/0.15.2/zig-x86_64-linux-0.15.2.tar.xz && \
    tar -xJf zig.tar.xz -C /usr/local --strip-components=1 && \
    rm zig.tar.xz

ENV PATH="/usr/local:${PATH}"

RUN rustup install nightly-2026-01-01 && \
    rustup default nightly-2026-01-01 && \
    cargo install cargo-zigbuild --version 0.22.1 --locked

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    ca-certificates \
    curl \
    xz-utils \
    libclang-dev \
    libssl-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*


FROM rust-2026-01-01-zig-0-15-2-zigbuild-0-22-1 AS rust
ENV SQLX_OFFLINE=true

FROM rust AS builder

WORKDIR /app
COPY . .

RUN cargo zigbuild --release --bin uts-calendar --target x86_64-unknown-linux-gnu.2.17 --features performance

FROM debian:trixie-slim AS calendar-runtime

WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/uts-calendar /app/app
ENTRYPOINT ["/app/uts-calendar"]
