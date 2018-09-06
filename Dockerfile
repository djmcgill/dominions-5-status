FROM ekidd/rust-musl-builder:nightly as dev
ADD . ./
RUN sudo chown -R rust:rust .
RUN cargo build --release

FROM alpine:latest
WORKDIR /usr/src/myapp
COPY --from=dev /home/rust/src/target/x86_64-unknown-linux-musl/release/dom5status .
ENV SSL_CERT_FILE=/etc/ssl/cert.pem
ENV SSL_CERT_DIR=/etc/ssl/certs
CMD ["./dom5status"]
