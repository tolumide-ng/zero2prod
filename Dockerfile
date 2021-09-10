FROM rust:1.55.0

WORKDIR /app
COPY . .
RUN cargo build --release
ENTRYPOINT ["./target/release/zero2prod"]

