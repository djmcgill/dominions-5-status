FROM rustlang/rust:nightly

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install

CMD ["dom5status"]
