FROM rust:1.81

WORKDIR /usr/doclytics
COPY . .

RUN cargo install --path .

CMD ["doclytics"]