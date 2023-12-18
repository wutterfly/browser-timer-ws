
ARG APP_NAME=browser-timer-ws

FROM rust:1.70-slim-buster as build

# create a new empty shell project
WORKDIR /server
RUN USER=root cargo init --bin .

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# copy your source tree
COPY ./src ./src

# build for release
RUN cargo build --release

# our final base
FROM debian:buster-slim

# copy the build artifact from the build stage
COPY --from=build /server/target/release/browser-timer-ws .

EXPOSE 8021

ENV RUST_LOG=info

# set the startup command to run your binary
CMD ["./browser-timer-ws"]