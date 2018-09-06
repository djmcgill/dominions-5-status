FROM rustlang/rust:nightly as dev
WORKDIR /usr/src/myapp
COPY . .
# RUN cargo build --release
RUN cargo build

FROM alpine:latest
RUN apk add --no-cache libressl-dev
WORKDIR /usr/src/myapp
COPY --from=dev target/release/dom5status .
RUN apt-get update && apt-get install -y libgoogle-perftools-dev
CMD ["sh", "-c", "HEAPPROFILE=/tmp/profile ./dom5status"]
