FROM rust as build

# create a new empty shell project
RUN USER=root cargo new --bin tracer-ome
RUN rustup override set nightly
WORKDIR /tracer-ome

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# cache dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy source tree
COPY ./src ./src

# build for release
RUN rm target/release/deps/tracer*
RUN cargo build --release

# build on final base
FROM rust:slim

# copy the build artifact from the build stage
COPY --from=build /tracer-ome/target/release/tracer-ome .

# run
ENV RUST_LOG=info
CMD ["./tracer-ome", "--force-no-tls"]
