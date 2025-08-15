# This is a template Dockerfile for building any binary crate in the workspace.
# It expects a build argument `CRATE_NAME` to be passed.

# Define the build argument for the crate name.
ARG CRATE_NAME

# Stage 1: Build the application in a full Rust environment
FROM rust:1.87 as builder
ARG CRATE_NAME # Make CRATE_NAME available in this stage

# Set up the application directory.
WORKDIR /app
 
# Copy the entire project context.
# The .dockerignore file will prevent unnecessary files from being copied.
COPY . .
 
# Compile the specific application using the build argument.
RUN cargo build --release --bin ${CRATE_NAME}
 
# Stage 2: Create the final, minimal production image
FROM debian:bookworm-slim
ARG CRATE_NAME # Make CRATE_NAME available in this stage
 
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*
 
WORKDIR /app
 
# Copy the compiled binary and the configuration from the builder stage.
COPY --from=builder /app/target/release/${CRATE_NAME} /app/${CRATE_NAME}
COPY --from=builder /app/config /app/config
 
# Ensure the binary is executable and define the entrypoint.
RUN chmod +x /app/${CRATE_NAME}
ENTRYPOINT ["/app/${CRATE_NAME}"]
