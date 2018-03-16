FROM rustlang/rust:nightly as dev
WORKDIR /usr/src/myapp
COPY . .
RUN cargo build --release

FROM alpine:latest
WORKDIR /usr/src/myapp
COPY --from=dev /usr/src/myapp/target/release/dom5status .
CMD ["./dom5status"]
