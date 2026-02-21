# syntax=docker/dockerfile:1

# Build stage
FROM rust:1.84-alpine AS build
WORKDIR /app

RUN apk add --no-cache musl-dev

# Cache deps
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    mkdir -p src/bin && echo 'fn main() {}' > src/bin/server.rs && \
    cargo build --release --features server --bin sudoku-server 2>/dev/null || true && \
    rm -rf src

# Copy source
COPY . .

# Build static binary
RUN cargo build --release --features server --bin sudoku-server && \
    cp target/release/sudoku-server /out-server

# Runtime stage (distroless)
FROM gcr.io/distroless/static-debian12:nonroot

ENV PORT=8080
EXPOSE 8080

COPY --from=build /out-server /server

USER nonroot:nonroot
ENTRYPOINT ["/server"]
