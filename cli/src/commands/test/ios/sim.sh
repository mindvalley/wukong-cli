#!/usr/bin/env bash
# Placeholder for the simClaw bash backend.
#
# The real script (~3000 lines) is owned and maintained by a separate team and
# will be dropped in here before the `wukong test` feature ships. The Rust CLI
# layer (cli/src/commands/test/ios/mod.rs) extracts this file to
# ~/.config/wukong/scripts/sim.sh on first invocation and delegates every
# subcommand to it.
#
# Contract (see RFC: "iOS backend: bash delegate"):
#   sim [--device <UDID>] <subcommand> [args...]
# Exit codes are mapped to WKCliError variants by the Rust layer.

set -euo pipefail

echo "wukong test ios backend is not yet installed." >&2
echo "This is a placeholder. Replace cli/src/commands/test/ios/sim.sh with" >&2
echo "the real simClaw script before release." >&2
exit 127
