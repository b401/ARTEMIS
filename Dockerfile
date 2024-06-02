FROM rust:1.78.0-bullseye AS builder
WORKDIR /app

COPY Cargo.toml .
COPY src src
COPY templates templates
RUN cargo build --release && \
	strip target/release/http && \
	apt update && apt install ca-certificates

FROM debian:bullseye-slim as release
WORKDIR /app
COPY --from=builder /app/target/release/http .
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY templates templates
COPY static static
COPY css css
RUN groupadd -g 999 -r artemis && useradd -u 999 --no-log-init -r -g artemis artemis
USER artemis
CMD ["./http"]
