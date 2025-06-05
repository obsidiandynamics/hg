_help:
    @just --list

# run Criterion bechmarks
bench:
    bash -c 'type cargo-criterion >/dev/null 2>&1 || cargo install cargo-criterion'
    cargo criterion
    
# run the tests
test:
    cargo test -- --include-ignored
    cargo test --examples
    cargo doc --no-deps
    cargo bench --no-run --profile dev

# run clippy with pedantic checks
clippy:
    cargo clippy -- -D clippy::pedantic -A clippy::must-use-candidate -A clippy::struct-excessive-bools -A clippy::single-match-else -A clippy::inline-always -A clippy::cast-possible-truncation -A clippy::cast-precision-loss -A clippy::items-after-statements

# install Rust
install-rust:
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
