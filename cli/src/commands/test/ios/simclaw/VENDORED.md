# Vendored simClaw

This directory is a vendored copy of the simClaw bash backend — **do not
edit files here directly.** Wukong's `test --platform ios` commands
delegate to `bin/sim`, which is bundled into the Wukong binary via
`include_str!` (see `../mod.rs`).

## Upstream

- **Repo:** https://github.com/mindvalley/mv-simclaw-ios
- **Commit:** `572a587935b55acb234cafc969c37b1417b9b408`
- **Ref:** `[fix] - Stage bundled skills + harden installer against existing symlinks (#2)`
- **Vendored on:** 2026-05-11

## Updating

When the simClaw team cuts a new release:

1. Pull the upstream repo to a local checkout.
2. Run `./revendor.sh <path-to-homebrew-simClaw>` from this directory.
3. Update the **Commit** and **Ref** lines above to the new SHA.
4. Rebuild Wukong and run the command-group smoke (`wukong test --platform ios --help`) to confirm the manifest still matches the on-disk tree.
5. If the re-vendor script reports new or removed files, update
   `SIMCLAW_FILES` in `../mod.rs` to match — the build will fail via the
   `manifest_matches_tree` test if you forget.

## Contract boundaries

The Rust layer assumes three stable contracts from the upstream script.
Breaking any of these will silently misbehave or loudly fail, so flag
them in PR review:

| Contract | Where it's checked |
|---|---|
| Subcommand names exactly match the strings passed from `ios/mod.rs` | `PlatformBackend` impl call sites |
| JSON shapes from `layout-map`, `find-element`, `describe`, `describe-point` match the structs in `../platform.rs` | `run_json` deserialization |
| Non-zero exit codes mean the operation failed | `TestError::ScriptFailed` |

If upstream changes any of those, fix the Rust side in the same PR that
re-vendors.
