FROM rustlang/rust:nightly as dev
WORKDIR /usr/src/myapp
COPY . .
RUN cargo build --release
CMD ["./target/release/dom5status"]
