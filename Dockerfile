FROM rust:1.78

WORKDIR /usr/doclytics
COPY . .

RUN cargo install --path .

CMD ["doclytics"]