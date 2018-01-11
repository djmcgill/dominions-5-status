FROM rustlang/rust:nightly

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install

CMD ["dom5status"]

# run with docker run -it -d --restart unless-stopped --volume /home/ec2-user/dominions-5-status/resources:/usr/src/myapp/resources dom-5-bot
