FROM rustlang/rust:nightly as dev
WORKDIR /usr/src/myapp
COPY . .
RUN apt-get update && apt-get install -y libgoogle-perftools-dev
RUN cargo build
CMD ["sh", "-c", "HEAPPROFILE=/tmp/profile ./target/debug/dom5status"]
