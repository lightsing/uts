FROM ghcr.io/lightsing/uts-builder:latest AS planner

WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM ghcr.io/lightsing/uts-builder:latest AS builder

ENV SQLX_OFFLINE=true

WORKDIR /app

RUN mkdir -p target/x86_64-unknown-linux-gnu/release && \
    mkdir -p target/x86_64-unknown-linux-gnu.2.17/release && \
    ln -s target/x86_64-unknown-linux-gnu/release target/x86_64-unknown-linux-gnu.2.17/release

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --zigbuild --target x86_64-unknown-linux-gnu.2.17

COPY . .
RUN cargo zigbuild --release --bin uts-calendar --target x86_64-unknown-linux-gnu.2.17 --features performance
RUN cargo zigbuild --release --bin uts

FROM debian:trixie-slim AS cli-runtime

WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/uts /app/uts
ENTRYPOINT ["/app/uts"]

FROM debian:trixie-slim AS calendar-runtime

WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/uts-calendar /app/uts-calendar
ENTRYPOINT ["/app/uts-calendar"]
