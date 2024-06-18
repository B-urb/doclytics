FROM rust:1.79

WORKDIR /usr/doclytics
COPY . .

RUN cargo install --path .

CMD ["doclytics"]