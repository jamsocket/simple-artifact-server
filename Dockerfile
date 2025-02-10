FROM rust:1.84-slim-bullseye AS builder

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

WORKDIR /work

COPY --from=builder /app/target/release/simple-artifact-server /app/simple-artifact-server

ENTRYPOINT [ "/app/simple-artifact-server" ]
