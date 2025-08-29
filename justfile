RUST_TOOLCHAIN := `grep 'channel = ' rust-toolchain.toml | cut -d '"' -f 2`
TARGET_PLATFORM := "linux/arm64"
DISTRI_PLATFORM := "ubuntu"

@env:
    echo "RUST_TOOLCHAIN: {{RUST_TOOLCHAIN}}"
    echo "TARGET_PLATFORM: {{TARGET_PLATFORM}}"

# List available just recipes
@help:
    just -l

@fmt:
    cargo +nightly fmt --all
    taplo format
    taplo format --check
    hawkeye format

# Calculate code
@cloc:
    cloc . --exclude-dir=vendor,docs,tests,examples,build,scripts,tools,target

@clean:
    cargo clean

@lint:
    cargo clippy --all --tests --all-features --no-deps

# Protobuf/gRPC operations with Buf
@proto-lint:
    cd api && buf lint

@proto-format:
    cd api && buf format -w

@proto-breaking:
    cd api && buf breaking --against .git#branch=main

@proto:
    cd api && buf generate

@proto-generate-go:
    cd api && buf generate --template buf.gen.go.yaml

@proto-generate-java:
    cd api && buf generate --template buf.gen.java.yaml

@proto-generate-cpp:
    cd api && buf generate --template buf.gen.cpp.yaml

@proto-generate-c:
    cd api && buf generate --template buf.gen.c.yaml

@proto-deps-update:
    cd api && buf dep update

@proto-push:
    cd api && buf push

# Documentation
@docs-serve:
    mdbook serve docs

@docs-build:
    mdbook build docs

@build:
    cargo build -p rsketch-cmd

# Example
@example-hello:
    cargo run --example hello-world

# Binary
@run:
    cargo run --package binary hello

alias c := check
@check:
    cargo check --all --all-features --all-targets

alias t := test
@test:
    cargo nextest run --verbose

# Docker
@build-docker:
    docker buildx build \
        --build-arg RUST_TOOLCHAIN={{RUST_TOOLCHAIN}} \
        --tag rsketch \
        --file docker/Dockerfile \
        --output type=docker \
        .

# GitHub Actions (local execution with act)
# Install act: https://github.com/nektos/act#installation

# Run the full CI workflow locally
@ci-local:
    act

# Run specific jobs from the CI workflow
@ci-validate:
    act -j validate

@ci-clippy:
    act -j clippy

@ci-docs:
    act -j docs

@ci-test:
    act -j test

@ci-coverage:
    act -j coverage

# List available workflows and jobs
@ci-list:
    act -l

# Run CI with specific event (push/pull_request)
@ci-push:
    act push

@ci-pr:
    act pull_request

# Debug CI workflow (with verbose output)
@ci-debug:
    act --verbose --dry-run

# Setup local environment for act
@ci-setup:
    echo "Setting up local CI environment..."
    @if [ ! -f .env.local ]; then \
        echo "Creating .env.local from example..."; \
        cp env.local.example .env.local; \
        echo "Please edit .env.local and add your GITHUB_TOKEN if needed"; \
    else \
        echo ".env.local already exists"; \
    fi
    @echo "Install act if not already installed:"
    @echo "  macOS: brew install act"
    @echo "  Linux: curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash"
    @echo "  Windows: choco install act-cli"