FROM rustlang/rust:nightly as dev
WORKDIR /usr/src/myapp
RUN rustup toolchain install nightly-2018-05-25 && rustup default nightly-2018-05-25
COPY . .
RUN cargo build
RUN apt-get update && apt-get install libtcmalloc-minimal4 google-perftools
CMD ["sh", "-c", "LD_PRELOAD=/usr/lib/libtcmalloc_minimal.so.4 HEAPPROFILE=/tmp/profile ./target/debug/dom5status"]
