FROM rust:1.77

WORKDIR /usr/doclytics
COPY . .

RUN cargo install --path .

CMD ["doclytics"]