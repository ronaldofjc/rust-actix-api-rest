FROM rust:latest as build
ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/my-api
COPY . .

RUN cargo install --path .

FROM gcr.io/distroless/cc-debian10

COPY --from=build /usr/local/cargo/bin/my-api /usr/local/bin/my-api

CMD ["my-api"]

## build stage
#FROM rust:latest as cargo-build
#
#RUN apt-get update && apt-get install libpq-dev musl-tools -y
#RUN rustup target add x86_64-unknown-linux-musl
#
#WORKDIR /usr/src/app
#
#COPY . .
#
#RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl
#
## final stage
#FROM alpine:latest
#
#RUN addgroup -g 1000 app
#RUN adduser -D -s /bin/sh -u 1000 -G app app
#
#WORKDIR /home/app/bin/
#
#COPY --from=cargo-build /usr/src/app/target/x86_64-unknown-linux-musl/release/my-api .
#
#RUN chown app:app my-api
#USER app
#
#EXPOSE 8080
#
#CMD ["./my-api"]