# Multi-stage build pro minimální velikost image
FROM rust:1.75-slim as builder

# Nainstalujeme system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Vytvoříme pracovní adresář
WORKDIR /app

# Zkopírujeme manifest soubory
COPY Cargo.toml Cargo.lock ./

# Vytvoříme dummy src soubor pro build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Buildneme dependencies (pro lepší cache layeru)
RUN cargo build --release
RUN rm src/main.rs

# Zkopírujeme skutečný zdrojový kód
COPY src ./src

# Buildneme aplikaci
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Nainstalujeme runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Vytvoříme non-root uživatele
RUN useradd -r -s /bin/false easyproject

# Vytvoříme pracovní adresář
WORKDIR /app

# Zkopírujeme binárku z builder stage
COPY --from=builder /app/target/release/easyproject-mcp-server .

# Zkopírujeme konfigurační soubor
COPY config.toml .

# Zkopírujeme swagger definici
COPY easy_swagger.yml .

# Nastavíme ownership
RUN chown -R easyproject:easyproject /app

# Přepneme na non-root uživatele
USER easyproject

# Exponujeme port pro WebSocket (pokud je použit)
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Spustíme aplikaci
CMD ["./easyproject-mcp-server"] 