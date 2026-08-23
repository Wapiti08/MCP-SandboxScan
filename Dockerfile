FROM rust:1.93-bookworm AS builder

WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY mcp-sandboxscan/Cargo.toml mcp-sandboxscan/Cargo.toml
COPY mcp-sandboxscan/src mcp-sandboxscan/src
RUN cargo build --locked --release --bin mcp-sandboxscan

FROM debian:bookworm-slim

ARG VERSION=dev
ARG REVISION=unknown
LABEL org.opencontainers.image.title="MCP-SandboxScan" \
      org.opencontainers.image.description="Dynamic security analysis for MCP tools and servers" \
      org.opencontainers.image.source="https://github.com/Wapiti08/MCP-SandboxScan" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${REVISION}" \
      org.opencontainers.image.licenses="MIT"

RUN useradd --create-home --uid 10001 scanner
COPY --from=builder /src/target/release/mcp-sandboxscan /usr/local/bin/mcp-sandboxscan

USER scanner
WORKDIR /work
ENTRYPOINT ["/usr/local/bin/mcp-sandboxscan"]
CMD ["--help"]
