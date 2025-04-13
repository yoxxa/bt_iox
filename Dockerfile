FROM rust:alpine AS builder
WORKDIR /app
USER root
RUN apk add --no-cache pkgconfig openssl-dev libc-dev
# MAKE MORE SPECIFIC LATER
COPY . ./
RUN cargo build --release

FROM arm64v8/alpine:3.21
COPY build/qemu-aarch64-static /usr/bin
WORKDIR /app
RUN apk update && apk add openssl ca-certificates
COPY --from=builder /app/target/release/bt_iox /app/bt_iox
ENTRYPOINT ["/app/bt_iox"]