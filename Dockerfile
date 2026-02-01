FROM node:20-bookworm AS build

# Install Rust toolchain + wasm-pack for WASM build
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
  && rm -rf /var/lib/apt/lists/*

RUN corepack enable

ENV RUSTUP_HOME=/usr/local/rustup \
  CARGO_HOME=/usr/local/cargo \
  PATH=/usr/local/cargo/bin:/usr/local/rustup/bin:$PATH

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal \
  && rustup target add wasm32-unknown-unknown \
  && cargo install wasm-pack

WORKDIR /app

# Install JS deps first for better layer caching
COPY web/package.json web/pnpm-lock.yaml ./web/
WORKDIR /app/web
RUN pnpm install --frozen-lockfile

# Copy only what the build needs (avoid host node_modules)
WORKDIR /app
COPY rs/Cargo.toml rs/Cargo.lock ./rs/
COPY rs/src ./rs/src
COPY web/index.html web/vite.config.js ./web/
COPY web/public ./web/public
COPY web/src ./web/src

WORKDIR /app/web
RUN pnpm run build

FROM nginx:1.27-alpine AS runtime

# Serve Vite build output
COPY --from=build /app/web/dist /usr/share/nginx/html

# Ensure .wasm is served with the correct MIME type
RUN printf '%s\n' \
  'server {' \
  '  listen 80;' \
  '  server_name _;' \
  '  root /usr/share/nginx/html;' \
  '  index index.html;' \
  '  include /etc/nginx/mime.types;' \
  '  location /assets/ {' \
  '    try_files $uri =404;' \
  '    add_header Cache-Control "public, max-age=31536000, immutable";' \
  '  }' \
  '  location /sprites/ {' \
  '    try_files $uri =404;' \
  '    add_header Cache-Control "public, max-age=31536000, immutable";' \
  '  }' \
  '  location = /index.html {' \
  '    add_header Cache-Control "no-cache";' \
  '  }' \
  '  location / {' \
  '    try_files $uri $uri/ /index.html;' \
  '  }' \
  '  types {' \
  '    application/wasm wasm;' \
  '  }' \
  '}' \
  > /etc/nginx/conf.d/default.conf

EXPOSE 80
