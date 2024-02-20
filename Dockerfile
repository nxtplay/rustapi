# Use the official Rust image for the build stage
FROM rust:latest as builder

# Create a new empty shell project
RUN USER=root cargo new --bin rustapi
WORKDIR /rustapi

# Copy your project's Cargo.toml and Cargo.lock and source code into the Docker image
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src

# Build your project in release mode
RUN cargo build --release

# Use the latest version of Ubuntu as the base image for runtime
FROM ubuntu:latest

ENV RUST_BACKTRACE=1
# Install necessary packages, including OpenSSL
RUN apt-get update && \
    apt-get install -y openssl libssl-dev curl && \
    rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /rustapi/target/release/rustapi /usr/local/bin/rustapi

# Set the default command of the container to run your binary
CMD ["/usr/local/bin/rustapi"]

