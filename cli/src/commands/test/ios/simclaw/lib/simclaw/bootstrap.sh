# lib/simclaw/bootstrap.sh — bootstrap, cache read/write
[[ -n "${_SIM_BOOTSTRAP_LOADED:-}" ]] && return 0; _SIM_BOOTSTRAP_LOADED=1

source "$SIM_LIB/device.sh"

# ── Bootstrap Cache ──────────────────────────────────────────────────────────

_write_bootstrap_cache() {
  local cache_file="/tmp/sim_bootstrap_cache_${SIM_TARGET_UDID:-default}.json"
  local win_pos
  win_pos=$(osascript << APPLESCRIPT 2>/dev/null
tell application "System Events"
  tell process "Simulator"
    set targetWindow to missing value
    set simFullTitle to "${SIM_NAME} – ${SIM_OS}"
    repeat with w in windows
      set wName to name of w
      if wName starts with simFullTitle then
        set targetWindow to w
        exit repeat
      end if
    end repeat
    if targetWindow is missing value then set targetWindow to window 1
    get position of targetWindow
  end tell
end tell
APPLESCRIPT
  )
  local wx wy
  wx=$(echo "$win_pos" | awk -F',' '{print $1}' | tr -d ' ')
  wy=$(echo "$win_pos" | awk -F',' '{print $2}' | tr -d ' ')
  # Read existing wda_port/wda_session from cache if present (preserve across re-bootstraps)
  local existing_wda_port existing_wda_session
  existing_wda_port=$(jq -r '.wda_port // ""' "$cache_file" 2>/dev/null || true)
  existing_wda_session=$(jq -r '.wda_session // ""' "$cache_file" 2>/dev/null || true)
  local wda_port_val="${WDA_PORT:-${existing_wda_port:-}}"
  local wda_session_val="${WDA_SESSION:-${existing_wda_session:-}}"
  cat > "$cache_file" << CACHEEOF
{
  "udid": "$SIM_UDID",
  "name": "$SIM_NAME",
  "device_type": "$SIM_DEVICE_TYPE",
  "os": "$SIM_OS",
  "logical_w": $SIM_LOGICAL_W,
  "logical_h": $SIM_LOGICAL_H,
  "scale": "$SIM_SCALE",
  "screen_x": $SIM_SCREEN_X,
  "screen_y": $SIM_SCREEN_Y,
  "screen_w": $SIM_SCREEN_W,
  "screen_h": $SIM_SCREEN_H,
  "zoom": "$SIM_ZOOM",
  "window_x": $wx,
  "window_y": $wy,
  "wda_port": "$wda_port_val",
  "wda_session": "$wda_session_val"
}
CACHEEOF
}

_load_bootstrap_cache() {
  local cache_file="/tmp/sim_bootstrap_cache_${SIM_TARGET_UDID:-default}.json"
  [[ -f "$cache_file" ]] || return 1
  # Validate UDID still booted
  local cached_udid
  cached_udid=$(jq -r '.udid' "$cache_file" 2>/dev/null)
  [[ -n "$cached_udid" ]] || return 1
  # If a specific device was requested, ensure the cache is for that device
  if [[ -n "$SIM_TARGET_UDID" && "$cached_udid" != "$SIM_TARGET_UDID" ]]; then
    return 1
  fi
  local state
  state=$(plutil -extract state raw "$HOME/Library/Developer/CoreSimulator/Devices/$cached_udid/device.plist" 2>/dev/null)
  [[ "$state" == "3" ]] || return 1
  # Validate window hasn't moved
  local cached_wx cached_wy
  cached_wx=$(jq -r '.window_x' "$cache_file")
  cached_wy=$(jq -r '.window_y' "$cache_file")
  local cached_name cached_os
  cached_name=$(jq -r '.name' "$cache_file")
  cached_os=$(jq -r '.os' "$cache_file")
  local current_pos
  current_pos=$(osascript << APPLESCRIPT 2>/dev/null
tell application "System Events"
  tell process "Simulator"
    set targetWindow to missing value
    set simFullTitle to "${cached_name} – ${cached_os}"
    repeat with w in windows
      set wName to name of w
      if wName starts with simFullTitle then
        set targetWindow to w
        exit repeat
      end if
    end repeat
    if targetWindow is missing value then set targetWindow to window 1
    get position of targetWindow
  end tell
end tell
APPLESCRIPT
  )
  local cur_wx cur_wy
  cur_wx=$(echo "$current_pos" | awk -F',' '{print $1}' | tr -d ' ')
  cur_wy=$(echo "$current_pos" | awk -F',' '{print $2}' | tr -d ' ')
  [[ "$cur_wx" == "$cached_wx" && "$cur_wy" == "$cached_wy" ]] || return 1
  # Load all globals from cache
  SIM_UDID=$(jq -r '.udid' "$cache_file")
  SIM_NAME=$(jq -r '.name' "$cache_file")
  SIM_DEVICE_TYPE=$(jq -r '.device_type' "$cache_file")
  SIM_OS=$(jq -r '.os' "$cache_file")
  SIM_LOGICAL_W=$(jq -r '.logical_w' "$cache_file")
  SIM_LOGICAL_H=$(jq -r '.logical_h' "$cache_file")
  SIM_SCALE=$(jq -r '.scale' "$cache_file")
  SIM_SCREEN_X=$(jq -r '.screen_x' "$cache_file")
  SIM_SCREEN_Y=$(jq -r '.screen_y' "$cache_file")
  SIM_SCREEN_W=$(jq -r '.screen_w' "$cache_file")
  SIM_SCREEN_H=$(jq -r '.screen_h' "$cache_file")
  SIM_ZOOM=$(jq -r '.zoom' "$cache_file")
  WDA_PORT=$(jq -r '.wda_port // ""' "$cache_file")
  WDA_SESSION=$(jq -r '.wda_session // ""' "$cache_file")
  return 0
}

_bootstrap() {
  [[ -n "$SIM_UDID" ]] && return 0  # already bootstrapped this invocation
  if _load_bootstrap_cache; then
    return 0  # cache hit — skip all detection
  fi
  # Cache miss — run full detection and write cache
  detect_device
  lookup_logical_resolution
  detect_screen_bounds
  _write_bootstrap_cache
}

# Write only wda_port and wda_session into the existing cache file (patch in-place via jq).
_wda_write_cache() {
  local cache_file="/tmp/sim_bootstrap_cache_${SIM_TARGET_UDID:-default}.json"
  [[ -f "$cache_file" ]] || die "_wda_write_cache: bootstrap cache missing — run bootstrap first"
  local tmp
  tmp=$(mktemp /tmp/sim_wda_cache_XXXXXX.json)
  jq --arg port "$WDA_PORT" --arg session "$WDA_SESSION" \
    '.wda_port = $port | .wda_session = $session' "$cache_file" > "$tmp" \
    && mv "$tmp" "$cache_file" \
    || { rm -f "$tmp"; die "_wda_write_cache: failed to update cache"; }
}
