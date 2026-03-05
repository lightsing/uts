FROM ghcr.io/lightsing/uts-builder:latest AS builder

ENV SQLX_OFFLINE=true

WORKDIR /app
COPY . .

RUN cargo zigbuild --release --bin uts-calendar --target x86_64-unknown-linux-gnu.2.17 --features performance

FROM debian:trixie-slim AS calendar-runtime

WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/uts-calendar /app/app
ENTRYPOINT ["/app/uts-calendar"]
