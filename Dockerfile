FROM rust:1.82

WORKDIR /usr/doclytics
COPY . .

RUN cargo install --path .

CMD ["doclytics"]