FROM rust:slim AS build
RUN apt-get update && apt-get install curl -y

WORKDIR /app
COPY ./ ./

# Build profile switches via ARG (`dev` or `release`)
ARG PROFILE=dev
ENV RUST_BACKTRACE=1
RUN cargo install --path . --root /out --profile ${PROFILE}

FROM alpine:latest

RUN adduser -D -H -s /sbin/nologin app
WORKDIR /app

COPY --from=build /out/bin/ferris-store /app/ferris-store

USER app

ENV RUST_BACKTRACE=1

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

ENTRYPOINT ["/app/ferris-store"]
