# Stage 1: Build the application
FROM rust:1.87 as builder

# Set the working directory
WORKDIR /app

# 1. Copy workspace-level lock & manifest for dependency caching
COPY Cargo.toml Cargo.lock ./
# Copy crate manifest to ensure dependency resolution succeeds
COPY crates/source-uniswap/Cargo.toml ./crates/source-uniswap/
COPY config ./config

# 2. Create a minimal skeleton so `cargo build` can cache deps
RUN mkdir -p crates/source-uniswap/src \
 && echo "fn main() {}" > crates/source-uniswap/src/main.rs \
 && cargo build --release --bin source-uniswap

# 3. Copy only this crate’s real source
COPY crates/source-uniswap/src ./crates/source-uniswap/src

# 4. Re-build the actual binary
RUN cargo build --release --bin source-uniswap

# Stage 2: Create the final, minimal image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update \
 && apt-get install -y --no-install-recommends libssl3 ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Set the working directory in the final image
WORKDIR /app

# Copy the binary to a standard location
COPY --from=builder /app/target/release/source-uniswap /usr/local/bin/source-uniswap

# Set the command to run when container starts
ENTRYPOINT ["source-uniswap"]