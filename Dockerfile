FROM rust:1.82

WORKDIR /usr/doclytics
COPY . .
ENV VERSION
RUN cargo install cargo edit
RUN cargo set-version ${{ env.VERSION }}

RUN cargo install --path .


CMD ["doclytics"]