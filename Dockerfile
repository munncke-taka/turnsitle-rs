# syntax=docker/dockerfile:1

# ---- Build ts-gate ----------------------------------------------------------
# Same Debian release as the runtime image, so the glibc versions match.
FROM rust:1-bookworm AS build
WORKDIR /src

COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

# ---- Runtime: nginx + ts-gate -----------------------------------------------
# Official nginx image includes ngx_http_auth_request_module.
FROM nginx:stable-bookworm

COPY --from=build /src/target/release/ts-gate /usr/local/bin/ts-gate
COPY docker/nginx.conf /etc/nginx/conf.d/default.conf
COPY docker/site/ /usr/share/nginx/html/site/
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Defaults for local testing. Override any of them with `docker run -e` or --env-file.
#   * Secure cookie off so the test works over plain http://localhost.
#     Set CHECK_COOKIE__SECURE=true behind TLS.
#   * nginx forwards the client IP in X-Real-IP (see nginx.conf).
ENV LISTEN_ADDRESS=127.0.0.1:9000 \
    IP_HEADER=X-Real-IP \
    ORIGINAL_URI_HEADER=X-Original-URI \
    CHECK_COOKIE__SECURE=false

# Required at runtime (no defaults, ts-gate refuses to start without them):
#   TURNSTILE__SITEKEY, TURNSTILE__SECRET, CHECK_COOKIE__HMAC_KEY

EXPOSE 80
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
