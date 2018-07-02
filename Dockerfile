FROM rustlang/rust:nightly as dev
WORKDIR /usr/src/myapp
COPY . .
RUN rustup target add x86_64-unknown-linux-musl

RUN PKG_CONFIG_ALLOW_CROSS=1 cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest
WORKDIR /usr/src/myapp
COPY --from=dev /usr/src/myapp/target/release/dom5status .
CMD ["./dom5status"]
