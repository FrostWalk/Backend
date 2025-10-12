# Buil stage
FROM rust:slim AS builder
RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y curl musl-tools build-essential pkg-config git
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app
COPY ./ ./

# compile a static musl binary
ARG PROFILE=release
# Accept Woodpecker CI variables (when build_args_from_env is used)
ARG CI_COMMIT_SHA=""
ARG CI_COMMIT_TAG=""
ARG CI_COMMIT_BRANCH=""
# Pass them to build environment so build.rs can access them
# Use BUILD_PROFILE to avoid conflicts with Cargo's internal PROFILE variable
ENV BUILD_PROFILE=${PROFILE}
ENV CI_COMMIT_SHA=${CI_COMMIT_SHA}
ENV CI_COMMIT_TAG=${CI_COMMIT_TAG}
ENV CI_COMMIT_BRANCH=${CI_COMMIT_BRANCH}
# Build with the specified profile
# Note: BUILD_PROFILE env var will be available to build.rs during compilation
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
ENV WORKERS=4
ENV DB_URL=""
ENV JWT_SECRET=""
ENV JWT_VALIDITY_DAYS=7
ENV LOGS_MONGO_URI=""
ENV LOGS_DB_NAME=""
ENV DEFAULT_ADMIN_PASSWORD=""
ENV DEFAULT_ADMIN_EMAIL="federico"
ENV ALLOWED_SIGNUP_DOMAINS="[\"studenti.unitn.it\"]"

ENV SMTP_HOST=""
ENV SMTP_PORT=587
ENV SMTP_USERNAME=""
ENV SMTP_PASSWORD=""
ENV APP_BASE_URL=""
ENV EMAIL_FROM="Advanced Programming"
ENV EMAIL_TOKEN_SECRET=""
ENV SKIP_EMAIL_CONFIRMATION=false

ENTRYPOINT ["/app/backend"]