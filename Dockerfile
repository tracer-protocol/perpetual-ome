# TODO
FROM rust
WORKDIR /user/src/tracer-ome
COPY . .
RUN cargo build --release

# TODO replace hardcoding with env vars
ENV RUST_LOG=info
CMD ["sh", "-c", "target/release/tracer-ome --executioner_address <EXECUTIONER_IP>"]
