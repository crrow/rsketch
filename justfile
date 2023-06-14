project_name := "HELLO"
home_dir := env_var('HOME')
current_dir := invocation_directory()

# helper
cloc:
    @echo cloc
    @cloc --exclude-dir target .
docs:
    @echo serve docs
    @mdbook serve docs
init_chglog:
    @go install github.com/git-chglog/git-chglog/cmd/git-chglog@latest
    @git-chglog --init
    @echo "Install Finished"
chglog:
	@git-chglog -o CHANGELOG.md
oc:
    @opencommit

#cargo
build:
    @cargo build
release:
    @cargo build --release
check:
    @cargo check --workspace
fmt:
    @cargo +nightly fmt --all
lint:
	@echo lint
	@cargo clippy
test:
    @cargo test --all
clean:
    @echo clean
    @cargo clean

