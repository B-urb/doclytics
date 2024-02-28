FROM rust:1.76

WORKDIR /usr/doclytics
COPY . .

RUN cargo install --path .

CMD ["doclytics"]