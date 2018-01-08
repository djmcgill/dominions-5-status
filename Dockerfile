FROM rust:1.23.0

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install --release

CMD ["dom5status > resources/logs"]
