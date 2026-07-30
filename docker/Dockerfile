# syntax=docker/dockerfile:1.6

# Build stage: static musl binary for the requested platform.
# Newer than MSRV: cargo 1.75 cannot parse current crates.io manifests.
FROM rust:1.88-alpine AS builder

ARG TARGETARCH
ARG GIT_SHA=unknown

RUN apk add --no-cache musl-dev

WORKDIR /src
COPY . .

RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/src/target \
    mkdir -p /out && \
    case "${TARGETARCH}" in \
      amd64) RUST_TARGET=x86_64-unknown-linux-musl ;; \
      arm64) RUST_TARGET=aarch64-unknown-linux-musl ;; \
      *) echo "unsupported arch: ${TARGETARCH}" && exit 1 ;; \
    esac && \
    rustup target add "$RUST_TARGET" && \
    cargo build --release --target "$RUST_TARGET" -p ailint-cli && \
    cp "target/$RUST_TARGET/release/ailint" /out/ailint

# Runtime stage: distroless static, non-root uid 65532.
FROM gcr.io/distroless/static-debian12:nonroot

ARG GIT_SHA=unknown

LABEL org.opencontainers.image.title="ailint"
LABEL org.opencontainers.image.source="https://github.com/jamesmhall/ailint"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"
LABEL org.opencontainers.image.description="Lint AI agent guidance files."
LABEL org.opencontainers.image.revision="${GIT_SHA}"

COPY --from=builder /out/ailint /ailint
WORKDIR /workspace
ENTRYPOINT ["/ailint"]
CMD ["check", "."]
