FROM ekidd/rust-musl-builder:stable AS builder

WORKDIR /tmp/workspace

COPY . ./

RUN sudo chown -R rust:rust /tmp/workspace \
  && sudo groupadd --gid 999 docker \
  && sudo usermod -a -G docker rust

RUN cargo build --release

FROM alpine

COPY --from=builder /tmp/workspace/target/x86_64-unknown-linux-musl/release/noria-operator /usr/local/bin

ENTRYPOINT [ "/usr/local/bin/noria-operator" ]
