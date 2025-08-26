# ---- Build stage ----
FROM rust:1.89-slim AS builder

WORKDIR /app

# Cache deps
COPY Cargo.toml Cargo.lock ./
COPY node/Cargo.toml node/
COPY core/Cargo.toml core/
COPY vm/Cargo.toml vm/
COPY compiler/Cargo.toml compiler/
COPY cli/Cargo.toml cli/
RUN cargo fetch

# Copy full source and build
COPY . .
RUN cargo build --release -p trove-node

# ---- Runtime stage ----
FROM rust:1.89-slim

WORKDIR /app
COPY --from=builder /app/target/release/trove-node /usr/local/bin/trove-node

EXPOSE 4321

ENTRYPOINT ["trove-node"]

