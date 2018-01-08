FROM rust:1.19.0

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install

CMD ["dom5status"]
