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

# Example
@hello:
    cargo run --example hello-world

alias c := check
@check:
    cargo check --all --all-features --all-targets