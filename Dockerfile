FROM rustlang/rust:nightly

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install

CMD ["dom5status"]

# run with docker run -d -it --volume "/home/ec2-user/dominions-5-status/resources":"/usr/src/myapp/resources" dom-5-bot-2
