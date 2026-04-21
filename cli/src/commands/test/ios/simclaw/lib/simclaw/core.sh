# lib/simclaw/core.sh — leaf helpers (no dependencies)
[[ -n "${_SIM_CORE_LOADED:-}" ]] && return 0; _SIM_CORE_LOADED=1

die() { echo "ERROR: $*" >&2; exit 1; }

_is_integer() { [[ "$1" =~ ^-?[0-9]+$ ]]; }
