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