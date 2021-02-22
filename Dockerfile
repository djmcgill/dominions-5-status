FROM messense/rust-musl-cross:x86_64-musl as builder

WORKDIR /home/rust/src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM scratch
COPY --from=builder /root/.cargo/bin/dom5status .
USER 1000
CMD ["./dom5status"]
