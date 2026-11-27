# ==============================================================================
# Stage 1: Chef Base (Rust + cargo-chef untuk optimalisasi layer caching)
# ==============================================================================
FROM lukemathwalker/cargo-chef:latest-rust-1-bookworm AS chef
WORKDIR /app

# ==============================================================================
# Stage 2: Planner (Menghasilkan recipe.json dari Cargo.toml & Cargo.lock)
# ==============================================================================
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ==============================================================================
# Stage 3: Builder (Cache dependency Cargo terlebih dahulu, lalu build binary)
# ==============================================================================
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Compile semua dependencies (layer ini di-cache selama Cargo.toml/Cargo.lock tidak berubah)
RUN cargo chef cook --release --recipe-path recipe.json

# Salin source code & migrasi, lalu build binary release
COPY . .
RUN cargo build --release --locked --bin agritech-api

# ==============================================================================
# Stage 4: Runtime (Minimal Debian Bookworm Slim + Non-Root User)
# ==============================================================================
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install CA certificates (untuk koneksi TLS/Supabase) dan tzdata (WIB/UTC+7)
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

# Buat non-root system user & group
RUN groupadd --system appgroup \
    && useradd --system --gid appgroup --no-create-home --shell /usr/sbin/nologin appuser

# Salin binary release & folder migrations dari stage builder
COPY --from=builder --chown=appuser:appgroup /app/target/release/agritech-api /app/agritech-api
COPY --from=builder --chown=appuser:appgroup /app/migrations /app/migrations

# Konfigurasi environment default (non-sensitive; inject secret saat runtime)
ENV PORT=8080 \
    ENV=production \
    RUST_LOG=info

EXPOSE 8080

# Jalankan container sebagai non-root user
USER appuser

ENTRYPOINT ["/app/agritech-api"]
