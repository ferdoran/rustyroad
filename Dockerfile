FROM rust:buster as builder
WORKDIR /usr/src/rustyroad
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/rustyroad /usr/local/bin/rustyroad
EXPOSE 3000
EXPOSE 8080
CMD ["rustyroad"]