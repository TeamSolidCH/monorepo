FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.lock ./Cargo.toml crates/calendarbot ./
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
ARG TARGETARCH
ARG TARGETOS
RUN if [ "$TARGETOS" = "linux" ]; then \
  if [ "$TARGETARCH" = "arm64" ]; then \
      export RUST_TARGET="aarch64-unknown-linux-gnu"; \
  elif [ "$TARGETARCH" = "amd64" ]; then \
      export RUST_TARGET="x86_64-unknown-linux-gnu"; \
  else \
      echo "Unsupported architecture: $TARGETARCH"; \
      exit 1; \
  fi; \
elif [ "$TARGETOS" = "windows" ]; then \
  if [ "$TARGETARCH" = "amd64" ]; then \
      export RUST_TARGET="x86_64-pc-windows-msvc"; \
  else \
      echo "Unsupported architecture: $TARGETARCH"; \
      exit 1; \
  fi; \
else \
  echo "Unsupported OS: $TARGETOS"; \
  exit 1; \
fi

COPY --from=planner /app/recipe.json recipe.json
RUN rustup target add $RUST_TARGET
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json --target $RUST_TARGET
# Build application
COPY ./Cargo.lock ./Cargo.toml crates/calendarbot ./
RUN cargo build --release --bin calendarbot --target $TARGETARCH

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
RUN apt-get update -y && apt-get install -y --no-install-recommends libpq-dev=15.8-0+deb12u1 \
	&& apt-get clean \
	&& rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/calendarbot /usr/local/bin
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
ENTRYPOINT ["/usr/local/bin/calendarbot"]
