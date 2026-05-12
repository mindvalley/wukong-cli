# Wukong CLI

## Pre-commit Checks

Run these before committing (CI uses Rust 1.81.0):

```sh
cargo fmt --all
cargo clippy -- -D warnings
cargo test --verbose
```

## Testing

- Snapshot tests use `insta`. If shell completion snapshots fail after adding/changing CLI commands, accept new snapshots by renaming `.snap.new` files to `.snap` in `cli/tests/snapshots/`.
