FROM messense/rust-musl-cross:x86_64-musl as builder

WORKDIR /home/rust/src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
# TODO: cache deps
RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM alpine:latest
WORKDIR /usr/src/myapp
COPY --from=builder /root/.cargo/bin/dom5status .
# FIXME: don't be root, but watch out for file read/write perms from the new user
# USER 1000
ENV SSL_CERT_FILE=/etc/ssl/cert.pem
ENV SSL_CERT_DIR=/etc/ssl/certs
CMD ["./dom5status"]
