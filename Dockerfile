FROM rust:buster as builder
WORKDIR /usr/src/rustyroad
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/rustyroad /usr/local/bin/rustyroad
EXPOSE 3000
EXPOSE 8080
CMD ["rustyroad"]