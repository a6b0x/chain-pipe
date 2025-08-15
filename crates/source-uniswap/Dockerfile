# Stage 1: Compile the application in a full Rust environment
FROM rust:1.87 as builder
 
# Set up the application directory.
WORKDIR /app
 
# Copy the entire project context.
# The .dockerignore file will prevent unnecessary files from being copied.
COPY . .
 
# Compile the application.
# This will create the binary at /app/target/release/source-uniswap
RUN cargo build --release --bin source-uniswap
 
 
# Stage 2: Create the final, minimal production image
FROM debian:bookworm-slim
 
# Install only essential runtime dependencies.
# 'strace' is included for debugging; it can be removed for a smaller production image.
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl3 ca-certificates strace && \
    rm -rf /var/lib/apt/lists/*
 
WORKDIR /app
 
# Copy the compiled binary and the configuration from the builder stage.
COPY --from=builder /app/target/release/source-uniswap /app/source-uniswap
COPY --from=builder /app/config /app/config
 
# Ensure the binary is executable and define the entrypoint.
RUN chmod +x /app/source-uniswap
ENTRYPOINT ["/app/source-uniswap"]