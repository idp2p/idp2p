FROM rust:1.64 as build

# create project
RUN USER=root cargo new --bin idp2p-gossip
WORKDIR /idp2p-gossip

# copy toml files
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# cache dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy source
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/idp2p-gossip*
RUN cargo build --release

# final base
FROM rust:1.64-slim-buster

COPY --from=build /idp2p-gossip/target/release/idp2p-gossip .

CMD ["./idp2p-gossip"]