# Release guide

This repository publishes binaries, checksums, a GitHub prerelease, and a multi-arch
GHCR image when a version tag is pushed. For `v0.1.0-alpha.1`, use the following flow.

## 1. Preflight

Run from the repository root:

```bash
cargo fmt --all -- --check
cargo build --locked --release
./target/release/mcp-sandboxscan --version
./examples/minimal.sh | jq '.summary'
```

The version must be `0.1.0-alpha.1`, and the example must report one flow. Review
`CHANGELOG.md`, `RELEASE_NOTES.md`, `THREAT_MODEL.md`, `BENCHMARK.md`, and the JSON
schema before tagging. The broader ecosystem benchmark is separate because it needs
Go, Python, Node.js, Javy, network access, and local socket permissions.

## 2. Commit the release candidate

Inspect the diff and ensure no scan report contains real credentials:

```bash
git status --short
git diff --check
git diff
git add .
git commit -m "release: v0.1.0-alpha.1"
git push origin main
```

Wait for the `CI` workflow on `main` to pass before creating the tag.

## 3. Tag and publish

The tag must exactly match `v` plus the Cargo package version; the workflow enforces
this relationship.

```bash
git tag -a v0.1.0-alpha.1 -m "MCP-SandboxScan v0.1.0-alpha.1"
git push origin v0.1.0-alpha.1
```

The `Release` workflow then:

1. builds Linux x86_64, macOS x86_64/arm64, and Windows x86_64 archives;
2. publishes `linux/amd64` and `linux/arm64` images to GHCR;
3. generates `SHA256SUMS`; and
4. creates a GitHub prerelease using `RELEASE_NOTES.md`.

Monitor it with:

```bash
gh run list --workflow Release --limit 3
gh run watch
```

If GHCR is being used for the first time, confirm the package visibility allows the
intended audience to pull it.

## 4. Verify published assets

```bash
gh release view v0.1.0-alpha.1
gh release download v0.1.0-alpha.1 --dir /tmp/mcp-sandboxscan-release
cd /tmp/mcp-sandboxscan-release
sha256sum --check SHA256SUMS

docker pull ghcr.io/wapiti08/mcp-sandboxscan:v0.1.0-alpha.1
docker run --rm ghcr.io/wapiti08/mcp-sandboxscan:v0.1.0-alpha.1 --version
```

On macOS, use `shasum -a 256 -c SHA256SUMS` if GNU `sha256sum` is unavailable.

Do not move or reuse a published tag. If the workflow fails before a usable release is
published, diagnose the failed job; delete and recreate the tag only when no consumer
could reasonably have fetched it. Otherwise publish the next prerelease version.
