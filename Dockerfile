FROM rustlang/rust:nightly as dev
WORKDIR /usr/src/myapp
COPY . .
RUN rustup target add x86_64-unknown-linux-musl

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest
WORKDIR /usr/src/myapp
COPY --from=dev /usr/src/myapp/target/release/dom5status .
RUN apk add --no-cache bash
CMD ["./dom5status"]
