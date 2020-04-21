FROM rust:slim-stretch AS noria-server

WORKDIR /tmp/noria

COPY noria/ ./

RUN apt-get update && \
    apt-get install -y \
        build-essential \
        libssl-dev \
        linux-libc-dev \
        pkgconf \
        llvm \
        clang \
        default-mysql-client && \
    apt-get clean 

RUN cargo build --release --bin noria-server

FROM rust:slim-stretch AS noria-mysql

WORKDIR /tmp/noria

COPY noria-mysql/ ./

RUN cargo build --release 

FROM rust:slim-stretch AS noria-ui

WORKDIR /tmp/noria

COPY noria-ui/ ./

RUN apt-get update && \
    apt-get install -y \
        build-essential \
        python3 \
        python3-pystache && \
    apt-get clean 

RUN make

FROM rust:slim-stretch

RUN apt-get update && \
    apt-get install -y \
        python3 && \
    apt-get clean 

RUN mkdir -p /var/lib/noria

COPY --from=noria-server /tmp/noria/target/release/noria-server /usr/local/bin/noria-server
COPY --from=noria-mysql /tmp/noria/target/release/noria-mysql /usr/local/bin/noria-mysql
COPY --from=noria-server /usr/bin/mysqladmin /usr/local/bin/mysqladmin

COPY noria-ui/*.css /srv/noria-ui/
COPY noria-ui/*.html /srv/noria-ui/
COPY noria-ui/js /srv/noria-ui/js
COPY --from=noria-ui /tmp/noria/*.css /srv/noria-ui/
COPY --from=noria-ui /tmp/noria/*.html /srv/noria-ui/
