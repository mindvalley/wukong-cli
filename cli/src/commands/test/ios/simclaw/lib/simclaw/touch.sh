# lib/simclaw/touch.sh — tap, swipe, scroll commands
[[ -n "${_SIM_TOUCH_LOADED:-}" ]] && return 0; _SIM_TOUCH_LOADED=1

source "$SIM_LIB/bootstrap.sh"
source "$SIM_LIB/wda.sh"
source "$SIM_LIB/coords.sh"
source "$SIM_LIB/inspect.sh"

cmd_tap() {
  local x="${1:?Usage: sim tap <x> <y>}"
  local y="${2:?Usage: sim tap <x> <y>}"
  _is_integer "$x" || die "tap: x must be an integer, got: $x"
  _is_integer "$y" || die "tap: y must be an integer, got: $y"

  # Try WDA pointer action first — works reliably on all devices
  if _tap_wda "$x" "$y" 2>/dev/null; then
    return 0
  fi

  # Fallback: CGEvent injection (requires simulator window position)
  echo "WDA tap unavailable, falling back to CGEvent injection..." >&2
  _bootstrap
  local screen_coords sx sy
  screen_coords=$(_ios_to_screen "$x" "$y")
  sx=$(echo "$screen_coords" | awk '{print $1}')
  sy=$(echo "$screen_coords" | awk '{print $2}')
  _tap_cgevent "$sx" "$sy"
}

cmd_swipe() {
  [[ $# -ge 4 ]] || die "Usage: sim swipe <x1> <y1> <x2> <y2> [steps] [step_ms]"
  _is_integer "$1" || die "swipe: x1 must be an integer, got: $1"
  _is_integer "$2" || die "swipe: y1 must be an integer, got: $2"
  _is_integer "$3" || die "swipe: x2 must be an integer, got: $3"
  _is_integer "$4" || die "swipe: y2 must be an integer, got: $4"
  local steps="${5:-25}"
  local step_ms="${6:-20}"
  local duration_ms=$(( steps * step_ms ))

  # Try WDA pointer action first — works reliably on all devices
  if _swipe_wda "$1" "$2" "$3" "$4" "$duration_ms" 2>/dev/null; then
    return 0
  fi

  # Fallback: legacy CGEvent-style injection (macOS coord round-trip)
  echo "WDA swipe unavailable, falling back to CGEvent injection..." >&2
  _bootstrap
  local sc1 sc2 sx1 sy1 sx2 sy2
  sc1=$(_ios_to_screen "$1" "$2")
  sx1=$(echo "$sc1" | awk '{print $1}')
  sy1=$(echo "$sc1" | awk '{print $2}')
  sc2=$(_ios_to_screen "$3" "$4")
  sx2=$(echo "$sc2" | awk '{print $1}')
  sy2=$(echo "$sc2" | awk '{print $2}')
  _swipe_cgevent "$sx1" "$sy1" "$sx2" "$sy2" "$steps" "$step_ms"
}

cmd_scroll_up() {
  # finger moves DOWN (from_y=0.27H → to_y=0.75H) — reveals content ABOVE current view
  _bootstrap
  local x from_y to_y
  x="${1:-$(awk "BEGIN { printf \"%d\", $SIM_LOGICAL_W / 2 }")}"
  from_y="${2:-$(awk "BEGIN { printf \"%d\", $SIM_LOGICAL_H * 0.27 }")}"
  to_y="${3:-$(awk "BEGIN { printf \"%d\", $SIM_LOGICAL_H * 0.75 }")}"
  _wda_swipe "$x" "$from_y" "$x" "$to_y"
}

cmd_scroll_down() {
  # finger moves UP (from_y=0.75H → to_y=0.27H) — reveals content BELOW current view
  _bootstrap
  local x from_y to_y
  x="${1:-$(awk "BEGIN { printf \"%d\", $SIM_LOGICAL_W / 2 }")}"
  from_y="${2:-$(awk "BEGIN { printf \"%d\", $SIM_LOGICAL_H * 0.75 }")}"
  to_y="${3:-$(awk "BEGIN { printf \"%d\", $SIM_LOGICAL_H * 0.27 }")}"
  _wda_swipe "$x" "$from_y" "$x" "$to_y"
}

cmd_scroll_to_visible() {
  [[ $# -ge 1 ]] || die "Usage: sim scroll-to-visible <label> [max_swipes]"
  local label="$1"
  local max_swipes="${2:-10}"
  _bootstrap

  local found=""
  local swipe_count=0

  while [[ $swipe_count -le $max_swipes ]]; do
    if found=$( cmd_find_element "$label" 2>/dev/null ); then
      # Validate the element is actually within the visible viewport.
      # WDA /element/{id}/rect returns content coordinates — y > SIM_LOGICAL_H
      # means the element is below the fold and needs more scrolling.
      local ey eh
      ey=$(echo "$found" | python3 -c "import sys,json; print(json.load(sys.stdin)['y'])" 2>/dev/null || echo "0")
      eh=$(echo "$found" | python3 -c "import sys,json; print(json.load(sys.stdin)['h'])" 2>/dev/null || echo "0")
      if [[ $ey -ge 0 && $(( ey + eh )) -le $SIM_LOGICAL_H ]]; then
        echo "$found"
        return 0
      fi
      # Element found but outside viewport — keep scrolling
    fi
    if [[ $swipe_count -lt $max_swipes ]]; then
      cmd_scroll_down
    fi
    (( swipe_count++ )) || true
  done

  die "scroll-to-visible: '$label' not found after $max_swipes scrolls"
}
