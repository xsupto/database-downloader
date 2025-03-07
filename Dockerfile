ARG RUST_VERSION=1.83.0
ARG APP_NAME=db_writter

FROM  rust:${RUST_VERSION}-alpine AS build
ARG APP_NAME

WORKDIR /app
RUN apk add --no-cache clang lld musl-dev git
ENV RUST_LOG=info
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && \
    cp ./target/release/$APP_NAME /bin/server

FROM  alpine AS final
ARG UID=10001
RUN apk add --no-cache bash postgresql16-client

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/app" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser

WORKDIR /app
RUN apk add --no-cache bash

COPY --from=build /bin/server /bin/
COPY backup.sh /app/backup.sh
COPY .env.example /app/.env

RUN chmod +x /app/backup.sh
RUN chown -R appuser:appuser /app

ENV RUST_LOG=info

USER appuser

CMD ["/bin/server"]




