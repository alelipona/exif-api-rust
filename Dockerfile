FROM rust:alpine AS builder

RUN apk add --no-cache \
    musl-dev \
    gcc \
    make \
    pkgconfig \
    openssl-dev \
    libc-dev

WORKDIR /app
COPY . .

RUN cargo build --release

FROM alpine:latest

# Устанавливаем exiftool
RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    libstdc++ \
    perl \
    exiftool \
    wget

# Проверяем установку
RUN exiftool -ver || echo "exiftool installed"

WORKDIR /app

COPY --from=builder /app/target/release/exif-api /usr/local/bin/exif-api

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD wget -O- http://localhost:3000/health || exit 1

CMD ["/usr/local/bin/exif-api"]
