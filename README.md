# ts-gate

A small Rust service that puts a [Cloudflare Turnstile](https://developers.cloudflare.com/turnstile/) check in front of any path served by nginx. Visitors solve the challenge once, get a signed cookie, and nginx lets them through until the cookie expires.

ts-gate does not proxy your content. It only answers three questions for nginx:

- "Does this request carry a valid cookie?" (`GET /_ts/verify`)
- "Show the challenge page." (`GET /_ts/challenge`)
- "Here is a Turnstile token; is it valid?" (`POST /_ts/challenge`)

## How it works

```
 visitor                     nginx                         ts-gate                  Cloudflare
    │  GET /files/a.zip         │                             │                          │
    │ ─────────────────────────▶│ auth_request /_ts/verify    │                          │
    │                           │ ───────────────────────────▶│ cookie ok? 204 / 401     │
    │                           │◀─────────────────────────── │                          │
    │                           │ 401 → error_page @ts_challenge                         │
    │  challenge page (HTML)    │ ───────────────────────────▶│ GET /_ts/challenge       │
    │ ◀──────────────────────────────────────────────────────── (X-Original-URI = next) │
    │  solves Turnstile widget  │                             │                          │
    │  fetch POST /_ts/challenge (token, next)                │                          │
    │ ─────────────────────────▶│ ───────────────────────────▶│ siteverify(token, IP) ──▶│
    │                           │                             │◀──────── success: true   │
    │                           │                             │ Set-Cookie: TS_STATUS=…  │
    │  302 → next               │◀─────────────────────────── │                          │
    │ ◀─────────────────────────│                             │                          │
    │  GET /files/a.zip (cookie)│ auth_request → 204 → proxied to the protected backend  │
```

1. **Gate.** nginx sends an internal subrequest to `/_ts/verify` before serving a protected location. ts-gate checks the signed cookie and answers `204` (allowed) or `401` (not allowed).
2. **Challenge.** On `401`, nginx serves `/_ts/challenge`, passing the original URI in `X-Original-URI`. ts-gate returns the challenge page with that URI embedded as the hidden `next` field.
3. **Verify.** The page loads the Turnstile widget. When the visitor solves it, the page `fetch()`es `POST /_ts/challenge` with the token and `next`, then reads the response:
   - **302 to `next`**: the token was accepted. The browser stores the cookie and the page navigates to `next`.
   - **400**: the token was rejected. The page shows a message and a "Tentar novamente" button, and gets a fresh Turnstile token.
   - **5xx**: a server-side problem. The page asks the visitor to try again later.
   - **Network failure or unexpected status**: the page says so and offers a retry.
4. **Cookie.** The cookie is `HttpOnly`, `SameSite=Lax`, and HMAC-SHA256 signed over its expiry timestamp and, optionally, the client IP. It is valid for `CHECK_COOKIE__TTL` seconds.

The server validates each token against Cloudflare's `siteverify` endpoint, retrying with exponential backoff (2 s, 4 s, 8 s…) on network errors and non-2xx answers, up to `TURNSTILE__MAX_RETRIES` attempts. Each attempt uses a fresh idempotency key.

## Endpoints

| Method | Path               | Purpose                                                                 | Responses                                                                 |
| ------ | ------------------ | ----------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| GET    | `/_ts/verify`      | Checks the `CHECK_COOKIE__NAME` cookie (HMAC, expiry, IP if bound).     | `204` valid · `401` missing, malformed, expired or invalid                |
| GET    | `/_ts/challenge`   | Serves the challenge page. `X-Original-URI` becomes `next`.             | `200` HTML                                                                |
| POST   | `/_ts/challenge`   | Form body: `cf-turnstile-response` (token) and `next`. Verifies, sets the cookie. | `302` to `next` on success · `400` rejected or failed · `400` on missing IP header · `500` signing error |

`next` is only honored when it is a local path: it must start with `/`, and not with `//` or `/\`. Anything else falls back to `/`.

## Configuration

Settings come from environment variables. A `.env` file in the working directory is loaded automatically.

### Required

| Variable                  | Description                                                                                                 |
| ------------------------- | ----------------------------------------------------------------------------------------------------------- |
| `TURNSTILE__SITEKEY`      | Turnstile site key. Injected into the page at startup.                                                     |
| `TURNSTILE__SECRET`       | Turnstile secret key, used for server-side verification.                                                    |
| `CHECK_COOKIE__HMAC_KEY`  | Key for signing the cookie. Must be **longer than 43 characters**. Generate one with `openssl rand -base64 32`. |

ts-gate refuses to start if a required variable is missing.

### Optional

| Variable                    | Default          | Description                                                                                                                              |
| --------------------------- | ---------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `TURNSTILE__MAX_RETRIES`    | `3`              | Verification attempts against Cloudflare, 1–5. Values above 5 abort startup.                                                             |
| `CHECK_COOKIE__NAME`        | `TS_STATUS`      | Name of the cookie.                                                                                                                      |
| `CHECK_COOKIE__TTL`         | `3600`           | Cookie lifetime, in seconds.                                                                                                             |
| `CHECK_COOKIE__BIND_IP`     | `true`           | Bind the cookie to the client IP. Requires the IP header below on every request.                                                         |
| `CHECK_COOKIE__SECURE`      | `true`           | Sets the `Secure` flag. Turn off only for plain-HTTP local testing.                                                                      |
| `IP_HEADER`                 | `X-Forwarded-From` | Header nginx uses to pass the client IP. The bundled nginx snippets send `X-Real-IP`, so set `IP_HEADER=X-Real-IP` to match them.       |
| `ORIGINAL_URI_HEADER`       | `X-Original-uri` | Header carrying the URI the visitor asked for. Used as `next`.                                                                           |
| `LISTEN_ADDRESS`            | `127.0.0.1:9000` | Address and port to bind.                                                                                                                |
| `TEMPLATE__CUSTOM_PATH`     | *(built-in)*     | Path to your own challenge HTML. Read once at startup.                                                                                   |
| `TEMPLATE__TITLE`           | *(empty)*        | Replaces `{{TITLE}}` in the template.                                                                                                    |
| `TEMPLATE__URL`             | *(empty)*        | Replaces `{{URL}}` in the template.                                                                                                      |
| `TEMPLATE__VERSION`         | *(empty)*        | Replaces `{{VERSION}}` in the template.                                                                                                  |

Example `.env`:

```dotenv
TURNSTILE__SITEKEY=0x4AAAAAAA...
TURNSTILE__SECRET=0x4AAAAAAA...
CHECK_COOKIE__HMAC_KEY=<output of: openssl rand -base64 32>

CHECK_COOKIE__BIND_IP=true
CHECK_COOKIE__TTL=3600
IP_HEADER=X-Real-IP
LISTEN_ADDRESS=127.0.0.1:9000
```

Keep the secret and HMAC key out of version control. `.env` is already in `.gitignore`.

## Building and running

Requires a recent stable Rust toolchain (edition 2024).

```sh
cargo build --release
./target/release/ts-gate      # reads ./.env if present
```

On startup it prints `Listenting on <address>`. Then nginx needs to be configured, as below.

## Testing with Docker

The image runs nginx and ts-gate together, with a hello-world page behind the gate. Use it to test the whole flow before configuring your production nginx.

```sh
docker build -t ts-gate-test .
docker run --rm -p 8080:80 --env-file .env ts-gate-test
```

Then open http://localhost:8080. You should see the challenge; after solving it you land on "Hello, world!".

- `Dockerfile`: multi-stage build. Rust compiles `ts-gate`, and the runtime is the official `nginx:stable-bookworm` image.
- `docker/nginx.conf`: the test gate. It protects `/` and leaves `/_ts/challenge` open.
- `docker/site/simple.html`: the protected page.
- `docker/entrypoint.sh`: starts `ts-gate` in the background, then nginx in the foreground.

The image defaults `CHECK_COOKIE__SECURE=false` and `IP_HEADER=X-Real-IP` so it works over plain `http://localhost`. Keep `CHECK_COOKIE__SECURE=true` in production behind TLS.

`.env` is not copied into the image (see `.dockerignore`). Pass it at run time with `--env-file`.

If you use Cloudflare's test keys (secret `1x0000000000000000000000000000000AA`), every token passes. To check the rejection path, pass a wrong secret with `-e TURNSTILE__SECRET=...`. The POST then returns `400`.

## nginx integration

The `temp/` directory has example snippets:

- `temp/ts-gate-server.conf`: include **once** per `server { }` block. Defines the internal `/_ts_check` subrequest, the `@ts_challenge` fallback, and the `/_ts/challenge` location.
- `temp/ts-gate.conf`: include in any `location` (or at server level) you want protected.
- `temp/example-site.conf`: a full example, both per-location and whole-domain.

Minimal setup:

```nginx
server {
    server_name example.com;

    include snippets/ts-gate-server.conf;

    location /files/ {
        include snippets/ts-gate.conf;
        proxy_pass http://fileserver;
    }
}
```

The fallback must pass the original URI and the client IP, and must not itself be gated:

```nginx
proxy_set_header X-Real-IP $remote_addr;
proxy_set_header X-Original-URI $request_uri;
```

If you run nginx behind another proxy, make sure the header ts-gate reads (`IP_HEADER`) carries the real client address, not the address of the proxy.

## Customizing the challenge page

The built-in page is `src/public/challenge.html`, compiled into the binary. To use your own, set `TEMPLATE__CUSTOM_PATH`. ts-gate replaces these placeholders at startup:

| Placeholder       | Replaced with                       |
| ----------------- | ----------------------------------- |
| `{{SITEKEY}}`     | `TURNSTILE__SITEKEY`                |
| `{{TITLE}}`       | `TEMPLATE__TITLE`                   |
| `{{URL}}`         | `TEMPLATE__URL`                     |
| `{{VERSION}}`     | `TEMPLATE__VERSION`                 |
| `{{NEXT_VALUE}}`  | The request's original URI (`X-Original-URI`), per request |

The page must:

- include the Turnstile script from `challenges.cloudflare.com`;
- contain a `<form method="POST" action="/_ts/challenge">` with the Turnstile widget (`cf-turnstile`) inside it, so the widget's `cf-turnstile-response` field is posted;
- keep a hidden `next` input.

The built-in page's script sends the POST with `fetch()` and reads the status code, instead of doing a native form submit. Its error messages are in Portuguese.

## Development

```sh
cargo check
cargo clippy
```

Project layout:

```
src/
├── main.rs            startup: env, HTTP client, router, listener
├── handler.rs         routes and the verify/challenge logic
├── extract.rs         ClientCtx extractor: client IP and `next`
├── app_error.rs       error type that turns into HTTP responses
├── utils.rs           now(), is_safe_path()
├── config.rs          env loading and the Turnstile settings
├── config/cookie.rs   cookie signing and HMAC verification
├── config/template.rs challenge page loading and placeholder replacement
└── public/challenge.html   built-in challenge page
temp/                  nginx and systemd examples
```

## Security notes

- The cookie is an HMAC over the expiry time and, when `CHECK_COOKIE__BIND_IP=true`, the client IP. A cookie copied to another IP is rejected, but only if your nginx sets the IP header correctly. Anything that can set `IP_HEADER` from outside can spoof the IP.
- Turnstile tokens are checked server-side with the client IP, and each attempt uses its own idempotency key.
- `next` is limited to local paths to prevent open redirects.
- The service binds to `127.0.0.1` by default. Keep it off the public network; only nginx should reach it.
- Run it under a restricted account. `temp/ts-gate.service` uses `DynamicUser`, `ProtectSystem=strict`, and `NoNewPrivileges`.

## Known gaps

- **`TURNSTILE__MAX_RETRIES=0`** does not make any attempt, so every token is rejected. Use 1–5.
- **Verify route name.** The code serves `GET /_ts/verify`, but the nginx snippets in `temp/` (and the commented-out code in `handler.rs`) still refer to `/_ts/check`. Update the snippets to `/_ts/verify`, or the gate never passes.
- **`CHECK_COOKIE__DOMAIN`** is read but not used yet.
- **Error details are not sent to the client.** A rejected token gets a bare `400`. The Cloudflare error codes are only written to stderr.
