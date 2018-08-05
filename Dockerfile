FROM rustlang/rust:nightly as dev
WORKDIR /usr/src/myapp
RUN rustup toolchain install nightly-2018-05-25 && rustup default nightly-2018-05-25
COPY . .
RUN cargo build
RUN apt-get update && apt-get install libgoogle-perftools-dev
CMD ["sh", "-c", "LD_PRELOAD=/usr/lib/libtcmalloc.so HEAPPROFILE=/tmp/profile ./target/debug/dom5status"]
