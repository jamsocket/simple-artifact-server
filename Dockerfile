FROM rust:1.84-slim-bullseye AS builder

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

WORKDIR /work

COPY --from=builder /app/target/release/simple-fragment-server /app/simple-fragment-server

ENTRYPOINT [ "/app/simple-fragment-server" ]
