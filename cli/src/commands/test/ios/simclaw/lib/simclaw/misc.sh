# lib/simclaw/misc.sh — status, screenshot, cleanup, install-skills, health-check, etc.
[[ -n "${_SIM_MISC_LOADED:-}" ]] && return 0; _SIM_MISC_LOADED=1

source "$SIM_LIB/device.sh"
source "$SIM_LIB/bootstrap.sh"

# Usage: sim --device <UDID> ensure-foreground-app [bundle_id]
cmd_ensure_foreground_app() {
  local bundle_id="${1:-${WDA_APP_BUNDLE_ID:-com.mindvalley.mvacademy}}"
  _bootstrap

  # Re-pin defaultActiveApplication
  curl -s -X POST \
    -H "Content-Type: application/json" \
    -d "{\"settings\":{\"defaultActiveApplication\":\"${bundle_id}\"}}" \
    --max-time 5 \
    "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/appium/settings" \
    > /dev/null 2>&1 || true

  # Activate the app to ensure it is in the foreground
  curl -s -X POST \
    -H "Content-Type: application/json" \
    -d "{\"bundleId\":\"${bundle_id}\"}" \
    --max-time 10 \
    "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/wda/apps/activate" \
    > /dev/null 2>&1 || true

  echo "Ensured foreground app: ${bundle_id}"
}

cmd_status() {
  detect_device
  lookup_logical_resolution

  echo "Booted Device:"
  echo "  UDID:        $SIM_UDID"
  echo "  Name:        $SIM_NAME"
  echo "  OS:          $SIM_OS"
  echo "  Device Type: $SIM_DEVICE_TYPE"
  echo "  Scale:       ${SIM_SCALE}x"
  echo "  Logical:     ${SIM_LOGICAL_W}x${SIM_LOGICAL_H} pts"

  local bounds_ok=false
  if pgrep -x Simulator > /dev/null 2>&1; then
    if detect_screen_bounds 2>/dev/null; then
      bounds_ok=true
    fi
  fi

  if $bounds_ok; then
    echo ""
    echo "Screen Bounds (macOS screen coordinates):"
    echo "  Origin:      (${SIM_SCREEN_X}, ${SIM_SCREEN_Y})"
    echo "  Size:        ${SIM_SCREEN_W}x${SIM_SCREEN_H} pts"
    echo "  Zoom factor: ${SIM_ZOOM}x"
  else
    echo ""
    echo "Screen Bounds: [unavailable — Simulator.app not running or Accessibility permission missing]"
  fi
}

cmd_screenshot() {
  [[ $# -ge 1 ]] || die "Usage: sim screenshot <output.png>"
  local outdir
  outdir=$(dirname "$1")
  [[ -d "$outdir" ]] || die "Output directory does not exist: $outdir"
  detect_device
  xcrun simctl io "$SIM_UDID" screenshot "$1" \
    || die "simctl screenshot failed for UDID $SIM_UDID — path: $1"
  echo "Screenshot saved: $1"
}

cmd_install_skills() {
  # Locate pkgshare: resolve via brew if available, else derive from script location
  local skills_src=""
  if command -v brew &>/dev/null; then
    local prefix
    prefix=$(brew --prefix simclaw 2>/dev/null) || true
    [[ -n "$prefix" ]] && skills_src="$prefix/share/simclaw/skills"
  fi
  # Fallback: derive from SIM_LIB (Homebrew: lib/../skills, repo: lib/simclaw/../../skills)
  if [[ -z "$skills_src" || ! -d "$skills_src" ]]; then
    skills_src="$(cd "$SIM_LIB/.." && pwd)/skills"
  fi
  if [[ -z "$skills_src" || ! -d "$skills_src" ]]; then
    skills_src="$(cd "$SIM_LIB/../.." && pwd)/skills"
  fi
  [[ -d "$skills_src" ]] \
    || die "install-skills: skills directory not found at $skills_src"

  local skills_dst="$HOME/.claude/skills"
  mkdir -p "$skills_dst"

  local installed=0
  for skill_dir in "$skills_src"/*/; do
    [[ -d "$skill_dir" ]] || continue
    local skill_name
    skill_name=$(basename "$skill_dir")
    mkdir -p "$skills_dst/$skill_name"
    cp -f "$skill_dir"/* "$skills_dst/$skill_name/"
    echo "Installed: $skill_name → $skills_dst/$skill_name"
    (( installed++ )) || true
  done

  [[ $installed -gt 0 ]] \
    || die "install-skills: no skills found in $skills_src"
  echo ""
  echo "$installed skill(s) installed. Available in any Claude Code session immediately."
}

cmd_cleanup() {
  echo "Cleaning up stale sim state..."
  # Kill any stale WDA processes
  pkill -f WebDriverAgentRunner 2>/dev/null || true
  sleep 1
  # Remove bootstrap caches
  rm -f /tmp/sim_bootstrap_cache_*.json
  # Remove queue artifacts
  rm -f /tmp/sim_queue.fifo /tmp/sim_queue.pid
  echo "Cleanup complete"
}

cmd_wda_session() {
  _bootstrap
  echo "$WDA_SESSION"
}

cmd_health_check() {
  _bootstrap
  local resp rc=0
  resp=$(curl -sf --max-time 5 "http://localhost:${WDA_PORT}/status" 2>/dev/null) || rc=$?
  if [[ $rc -eq 0 ]] && [[ -n "$resp" ]]; then
    echo "OK — WDA responding on port ${WDA_PORT}"
    echo "$resp" | python3 -c "import sys,json; d=json.load(sys.stdin).get('value',{}); print(f'  Session: {d.get(\"sessionId\",\"unknown\")}')" 2>/dev/null || true
  else
    echo "FAILED — WDA not responding on port ${WDA_PORT}" >&2
    return 1
  fi
}
