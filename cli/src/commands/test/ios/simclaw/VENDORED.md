# Vendored simClaw

This directory is a vendored copy of the simClaw bash backend — **do not
edit files here directly.** Wukong's `test --platform ios` commands
delegate to `bin/sim`, which is bundled into the Wukong binary via
`include_str!` (see `../mod.rs`).

## Upstream

- **Repo:** https://github.com/justinchampappilly-mindvalley/homebrew-simClaw
- **Commit:** `2381b2da5139a49f063a6e0a4d907c113771ecab`
- **Ref:** `[bump] - Bump formula to v1.0.6`
- **Vendored on:** 2026-04-21

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
