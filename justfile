project_name := "HELLO"

build:
    @cargo build
check:
    @cargo check --workspace
lint:
	@echo lint
	@rustup component add clippy 2> /dev/null
	@cargo clippy
clean:
    @echo clean
    @cargo clean
cloc:
    @echo cloc
    @cloc --exclude-dir target .
