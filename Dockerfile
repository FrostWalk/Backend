# Buil stage
FROM rust:slim AS builder
RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y curl musl-tools build-essential pkg-config
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app
COPY ./ ./

# compile a static musl binary
ARG PROFILE=release
RUN cargo install --path . --root /out --profile ${PROFILE} --target x86_64-unknown-linux-musl

# Runtime Stage
FROM alpine:latest

RUN adduser -D app
WORKDIR /app

COPY --from=builder --chown=app:app /out/bin/backend /app/backend
RUN chmod +x /app/backend

USER app

ENV RUST_BACKTRACE=0

# app environmnet variables
ENV ADDRESS="0.0.0.0"
ENV PORT=8080
ENV WORKERS=0
ENV DB_URL=""
ENV JWT_SECRET=""
ENV JWT_VALIDITY_DAYS=7
ENV LOGS_MONGO_URI=""
ENV LOGS_DB_NAME=""
ENV DEFAULT_ADMIN_PASSWORD=""
ENV DEFAULT_ADMIN_EMAIL="federico"

ENV SMTP_HOST=""
ENV SMTP_PORT=587
ENV SMTP_USERNAME=""
ENV SMTP_PASSWORD=""
ENV APP_BASE_URL=""
ENV EMAIL_FROM="Advanced Programming"
ENV EMAIL_TOKEN_SECRET=""

ENTRYPOINT ["/app/backend"]