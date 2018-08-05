FROM rustlang/rust:nightly as dev
WORKDIR /usr/src/myapp
COPY . .
RUN cargo build
RUN apt-get update && apt-get install libtcmalloc-minimal4
CMD ["sh", "-c", "LD_PRELOAD=\"/usr/lib/libtcmalloc.so.4\" HEAPPROFILE=/tmp/profile ./target/debug/dom5status"]
