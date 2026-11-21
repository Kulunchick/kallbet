# syntax=docker/dockerfile:1
# Three stages: SPA bundle, Rust binary, then a small runtime image
# (Alpine + the binary + the sqlite3 CLI used by the backup job).

# --- 1. Frontend: web/ -> web/dist -------------------------------------------
FROM node:22-alpine AS web
WORKDIR /web
COPY web/package.json web/package-lock.json ./
RUN npm ci --no-audit --no-fund
COPY web/ ./
RUN npm run build

# --- 2. Backend: the kallbet binary (musl, so it runs on plain Alpine) -------
FROM rust:1-alpine AS build
RUN apk add --no-cache musl-dev
WORKDIR /src
ENV CARGO_PROFILE_RELEASE_STRIP=true
# Compile the dependencies first, against stub sources, so that a change to
# our own code does not rebuild them.
COPY Cargo.toml Cargo.lock build.rs ./
RUN mkdir src migrations \
 && echo 'fn main() {}' > src/main.rs \
 && touch src/lib.rs \
 && cargo build --release --locked \
 && rm -rf src
COPY migrations migrations
COPY src src
RUN touch src/main.rs src/lib.rs && cargo build --release --locked

# --- 3. Runtime ----------------------------------------------------------------
FROM alpine:3.21
RUN apk add --no-cache ca-certificates sqlite \
 && addgroup -S -g 10001 kallbet && adduser -S -u 10001 -G kallbet kallbet \
 && mkdir /data && chown kallbet:kallbet /data
COPY --from=build /src/target/release/kallbet /usr/local/bin/kallbet
COPY --from=web /web/dist /app/web
ENV KALLBET_BIND=0.0.0.0:8095 \
    KALLBET_WEB_DIR=/app/web \
    DATABASE_URL=sqlite:///data/kallbet.db
USER kallbet
VOLUME /data
EXPOSE 8095
CMD ["kallbet"]
