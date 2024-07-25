FROM rust:1.80

WORKDIR /usr/doclytics
COPY . .

RUN cargo install --path .

CMD ["doclytics"]