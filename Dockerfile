FROM rust:1.82

WORKDIR /usr/doclytics
COPY . .
ARG VERSION
RUN cargo install cargo edit
RUN cargo set-version ${VERSION}

RUN cargo install --path .


CMD ["doclytics"]