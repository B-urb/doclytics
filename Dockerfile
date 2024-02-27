FROM rust:1.67

WORKDIR /usr/doclytics
COPY . .

RUN cargo install --path .

CMD ["doclytics"]