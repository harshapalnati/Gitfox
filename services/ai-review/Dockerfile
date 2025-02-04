# Stage 1: Build Stage
FROM rust:1.75-slim-bullseye as builder

WORKDIR /build
# Create the generated directory
RUN mkdir -p src/generated

COPY proto proto/
COPY services/ai-review .

# Install build dependencies
RUN apt-get update && \
    apt-get install -y \
    protobuf-compiler \
    pkg-config \
    libssl-dev \
    && rustup update stable \
    && rustup default stable

RUN cargo build --release

# Stage 2: Runtime Stage
FROM debian:bullseye-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y libssl1.1 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/ai-review /app/ai-review

EXPOSE 50051

CMD ["./ai-review"]
