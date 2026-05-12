# Wukong CLI

## Pre-commit Checks

Run these before committing (CI uses Rust 1.81.0):

```sh
cargo fmt --all
cargo clippy -- -D warnings
cargo test --verbose
```
