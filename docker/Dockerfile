# Build stage
FROM rust:latest as builder

WORKDIR /app

# Install Vosk library for building
RUN apt-get update && apt-get install -y \
    wget \
    unzip \
    && wget https://github.com/alphacep/vosk-api/releases/download/v0.3.45/vosk-linux-aarch64-0.3.45.zip -O /tmp/vosk.zip \
    && unzip /tmp/vosk.zip -d /usr/local \
    && cp /usr/local/vosk-linux-aarch64-0.3.45/libvosk.so /usr/local/lib/ \
    && ldconfig \
    && rm -rf /tmp/vosk.zip \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests and migrations
COPY Cargo.toml ./
COPY src ./src
COPY tests ./tests
COPY migrations ./migrations

# Skip tests for now - uncomment to enable
# RUN cargo test --release

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    wget \
    unzip \
    libatomic1 \
    && rm -rf /var/lib/apt/lists/*

# Install Vosk library for runtime
RUN wget https://github.com/alphacep/vosk-api/releases/download/v0.3.45/vosk-linux-aarch64-0.3.45.zip -O /tmp/vosk.zip \
    && unzip /tmp/vosk.zip -d /usr/local \
    && cp /usr/local/vosk-linux-aarch64-0.3.45/libvosk.so /usr/local/lib/ \
    && ldconfig \
    && rm -rf /tmp/vosk.zip

WORKDIR /app

# Download Vosk model
RUN mkdir -p /models && \
    cd /models && \
    curl -L https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip -o model.zip && \
    unzip model.zip && \
    rm model.zip && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/rusty-tea /app/rusty-tea

# Copy migrations folder to runtime
COPY migrations ./migrations

EXPOSE 3000

ENV API_KEY=prod_api_key_change_me_12345
ENV VOSK_MODEL_PATH=/models/vosk-model-small-en-us-0.15
ENV RUST_LOG=info

CMD ["/app/rusty-tea"]
