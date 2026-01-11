# Load environment variables from .env.local if it exists
set dotenv-load
set dotenv-filename := ".env.local"

# Environment variables with defaults
RUST_TOOLCHAIN := `grep 'channel = ' rust-toolchain.toml | cut -d '"' -f 2`
TARGET_PLATFORM := env("TARGET_PLATFORM", "linux/arm64")
DISTRI_PLATFORM := env("DISTRI_PLATFORM", "ubuntu")
DOCKER_TAG := env("DOCKER_TAG", "rsketch:latest")

# ========================================================================================
# Default Recipe & Help
# ========================================================================================

[group("📒 Help")]
[private]
default:
    @just --list --list-heading '🦀 rsketch justfile manual page:\n'

[doc("show help")]
[group("📒 Help")]
help: default

[doc("show environment variables")]
[group("📒 Help")]
env:
    @echo "🔧 Environment Configuration:"
    @echo "  RUST_TOOLCHAIN: {{RUST_TOOLCHAIN}}"
    @echo "  TARGET_PLATFORM: {{TARGET_PLATFORM}}"
    @echo "  DISTRI_PLATFORM: {{DISTRI_PLATFORM}}"
    @echo "  DOCKER_TAG: {{DOCKER_TAG}}"

# ========================================================================================
# Code Quality
# ========================================================================================

[doc("run `cargo fmt` to format Rust code")]
[group("👆 Code Quality")]
fmt: fmt-proto
    @echo "🔧 Formatting Rust code..."
    cargo +nightly fmt --all
    @echo "🔧 Formatting TOML files..."
    taplo format
    @echo "🔧 Formatting with hawkeye..."
    hawkeye format
    @echo "✅ All formatting complete!"

[doc("format protobuf files")]
[group("👆 Code Quality")]
[working-directory: 'api']
fmt-proto:
    @echo "🔧 Formatting protobuf files..."
    buf format -w

[doc("run `cargo fmt` in check mode")]
[group("👆 Code Quality")]
fmt-check:
    @echo "📝 Checking Rust code formatting..."
    cargo +nightly fmt --all --check
    @echo "📝 Checking TOML formatting..."
    taplo format --check

[doc("run `cargo clippy`")]
[group("👆 Code Quality")]
clippy:
    @echo "🔍 Running clippy checks..."
    cargo clippy --workspace --all-targets --all-features --no-deps -- -D warnings

[doc("run `cargo check`")]
[group("👆 Code Quality")]
check:
    @echo "🔨 Running compilation check..."
    cargo check --all --all-features --all-targets

alias c := check

[doc("run `cargo test`")]
[group("👆 Code Quality")]
test:
    @echo "🧪 Running tests with nextest..."
    cargo nextest run --workspace --all-features

alias t := test

[doc("run linting checks (clippy, docs, buf, golangci)")]
[group("👆 Code Quality")]
lint:
    @echo "🔍 Running clippy..."
    cargo clippy --workspace --all-targets --all-features --no-deps -- -D warnings
    @echo "📚 Building documentation..."
    cargo doc --workspace --all-features --no-deps --document-private-items
    @echo "🔍 Linting protobuf..."
    cd api && buf lint
    @echo "🔍 Linting Go code..."
    cd examples/goclient && golangci-lint run
    @echo "✅ All linting checks passed!"

[doc("run `fmt` `clippy` `check` `test` at once")]
[group("👆 Code Quality")]
pre-commit: fmt clippy check test
    @echo "✅ All pre-commit checks passed!"

[doc("clean build artifacts")]
[group("👆 Code Quality")]
clean:
    @echo "🧹 Cleaning build artifacts..."
    cargo clean

[doc("count lines of code")]
[group("👆 Code Quality")]
cloc:
    @echo "📊 Counting lines of code..."
    cloc . --exclude-dir=vendor,docs,tests,examples,build,scripts,tools,target

# ========================================================================================
# Build
# ========================================================================================

[doc("build rsketch binary")]
[group("🔨 Build")]
build:
    @echo "🔨 Building rsketch..."
    cargo build -p rsketch-cmd

[doc("build in release mode")]
[group("🔨 Build")]
build-release:
    @echo "🔨 Building rsketch (release mode)..."
    cargo build -p rsketch-cmd --release

# ========================================================================================
# Protobuf/gRPC
# ========================================================================================

[doc("generate code from protobuf definitions")]
[group("🔌 Protobuf")]
[working-directory: 'api']
proto:
    @echo "🔌 Generating code from protobuf..."
    buf generate

# ========================================================================================
# Documentation
# ========================================================================================

[doc("serve documentation with mdbook")]
[group("📚 Documentation")]
book:
    @echo "📚 Serving documentation..."
    mdbook serve docs --port 13000

[doc("build documentation with mdbook")]
[group("📚 Documentation")]
docs-build:
    @echo "📚 Building documentation..."
    mdbook build docs

[doc("open cargo docs in browser")]
[group("📚 Documentation")]
docs-open:
    @echo "📚 Opening cargo documentation..."
    cargo doc --workspace --all-features --no-deps --document-private-items --open

# ========================================================================================
# Running & Examples
# ========================================================================================

[doc("run the binary")]
[group("🏃 Running")]
run:
    @echo "🏃 Running rsketch binary..."
    cargo run --package binary hello

[doc("run hello-world example")]
[group("🏃 Running")]
example-hello:
    @echo "🏃 Running hello-world example..."
    cargo run --example hello-world

# ========================================================================================
# Docker
# ========================================================================================

[doc("build Docker image")]
[group("🐳 Docker")]
build-docker:
    @echo "🐳 Building Docker image..."
    docker buildx build \
        --build-arg RUST_TOOLCHAIN={{RUST_TOOLCHAIN}} \
        --tag {{DOCKER_TAG}} \
        --file docker/Dockerfile \
        --output type=docker \
        .

[doc("build Docker image for multiple platforms")]
[group("🐳 Docker")]
build-docker-multiarch:
    @echo "🐳 Building multi-arch Docker image..."
    docker buildx build \
        --platform linux/amd64,linux/arm64 \
        --build-arg RUST_TOOLCHAIN={{RUST_TOOLCHAIN}} \
        --tag {{DOCKER_TAG}} \
        --file docker/Dockerfile \
        .

# ========================================================================================
# Development Tools
# ========================================================================================

[doc("update dependencies interactively")]
[group("🔧 Development")]
deps-update:
    @echo "📦 Updating dependencies..."
    ./scripts/update-deps.sh

[doc("run GitHub Actions locally with act")]
[group("🔧 Development")]
act:
    @echo "🎬 Running GitHub Actions locally..."
    ./scripts/ci-act.sh check-all
