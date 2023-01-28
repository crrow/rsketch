project_name := "HELLO"
home_dir := env_var('HOME')
current_dir := invocation_directory()

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
book:
    @echo serve book
    @mdbook serve book
init_chglog:
    @go install github.com/git-chglog/git-chglog/cmd/git-chglog@latest
    @git-chglog --init
    @echo "Install Finished"
chglog:
	@git-chglog -o CHANGELOG.md
