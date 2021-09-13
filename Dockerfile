# Builder stage
FROM rust:1.53.0-slim AS builder

WORKDIR /app
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

# Runtime stage
FROM rust:1.53.0 AS runtime
WORKDIR /app
# copy the compuled binary from the builder environment
# to our runtime environment
COPY --from=builder /app/target/release/zero2prod zero2prod
# We need the configuration file at runtime!
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./target/release/zero2prod"]

