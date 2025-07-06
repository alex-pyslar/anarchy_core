# --- BUILD STAGE ---
FROM rust:1.77-slim-buster AS builder

# Установка зависимостей, необходимых для компиляции PostgreSQL-драйвера (sqlx)
# и других системных библиотек.
RUN apt-get update && apt-get install -y \
    pkg-config \
    libpq-dev \
    gcc \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Копируем Cargo.toml и Cargo.lock для кэширования зависимостей
COPY Cargo.toml Cargo.lock ./

# Копируем исходный код
COPY src ./src

# Кэшируем зависимости: попытка скомпилировать фиктивную бинарную папку
RUN mkdir -p src/bin/dummy_app && \
    echo "fn main() {println!(\"dummy\");}" > src/bin/dummy_app/main.rs && \
    cargo build --bin dummy_app --release && \
    rm -rf src/bin/dummy_app

# Собираем ваш основной проект
RUN cargo build --release --bin anarchy_core

# --- RUNTIME STAGE ---
FROM debian:buster-slim

# Установка зависимостей, необходимых для запуска PostgreSQL-драйвера
RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Копируем скомпилированный бинарник из стадии сборки
COPY --from=builder /app/target/release/anarchy_core ./anarchy_core

# Устанавливаем ENTRYPOINT для запуска приложения
ENTRYPOINT ["./anarchy_core"]

# Порт, который будет слушать ваш сервер
EXPOSE 3000