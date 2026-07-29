# syntax=docker/dockerfile:1.6
# TODO:
#   - pin Rust and base image digests
#   - build musl for a smaller final image
#   - multi-arch build via buildx
#   - non-root user
#   - OCI labels (title, source, licenses, revision)

FROM rust:1.75-slim AS builder
WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release -p ailint-cli \
 && cp /src/target/release/ailint /usr/local/bin/ailint

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /usr/local/bin/ailint /usr/local/bin/ailint
WORKDIR /workspace
ENTRYPOINT ["/usr/local/bin/ailint"]
CMD ["check", "."]
