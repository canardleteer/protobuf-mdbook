# syntax=docker/dockerfile:1.7

# Pinned Buf CLI for builder sanity checks (`buf --version`), matching `buf-toolchain` 1.69.0.
# Runtime image remains `scratch` + static `protoc-gen-mdbook` only.

ARG RUST_VERSION=1.95.0

FROM rust:${RUST_VERSION}-bookworm AS buf-anchor
ENV CARGO_HOME=/usr/local/cargo
RUN cargo install buf-toolchain --locked --version 1.69.0 \
  && install -m0755 "${CARGO_HOME}/bin/buf" /tmp/buf

FROM rust:${RUST_VERSION}-bookworm AS builder
WORKDIR /src
ENV CARGO_HOME=/usr/local/cargo
RUN apt-get update \
  && apt-get install -y --no-install-recommends musl-tools \
  && rm -rf /var/lib/apt/lists/* \
  && rustup target add x86_64-unknown-linux-musl
COPY --from=buf-anchor /tmp/buf /usr/local/bin/buf
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY assets ./assets
COPY src ./src
COPY xtask ./xtask
RUN buf --version \
  && cargo build --locked --release --target x86_64-unknown-linux-musl -p protoc-gen-mdbook

FROM debian:bookworm-slim AS ids
RUN echo 'nobody:x:65534:65534:nobody:/nonexistent:/usr/sbin/nologin' > /passwd \
  && echo 'nogroup:x:65534:' > /group

FROM scratch
COPY --from=ids /passwd /etc/passwd
COPY --from=ids /group /etc/group
COPY --from=builder /src/target/x86_64-unknown-linux-musl/release/protoc-gen-mdbook /protoc-gen-mdbook
USER nobody
ENTRYPOINT ["/protoc-gen-mdbook"]
