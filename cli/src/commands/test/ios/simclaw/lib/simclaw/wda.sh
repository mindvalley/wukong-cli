# lib/simclaw/wda.sh — WDA session management and touch primitives
[[ -n "${_SIM_WDA_LOADED:-}" ]] && return 0; _SIM_WDA_LOADED=1

source "$SIM_LIB/bootstrap.sh"
source "$SIM_LIB/coords.sh"

# WDA cache version. Bump this whenever `_wda_patch_ios_compat` changes so any
# existing cached WDA build gets wiped and re-built + re-patched on next setup.
# Cached builds carry a `.patch_version` stamp inside their derived data dir;
# a mismatch triggers a clean rebuild in cmd_setup.
WDA_PATCH_VERSION="2"

# _wda_derived_path
# Returns the per-iOS-version WDA DerivedData path so each iOS runtime gets its
# own cache. Without this, a WDA build patched for one runtime (say iOS 26) gets
# silently reused on a different runtime (say iOS 18), where it either fails to
# launch or behaves incorrectly. Per-iOS caches also let qa-branch test across
# multiple iOS versions in parallel without the caches colliding.
# Requires SIM_OS to be populated (call after `_bootstrap`).
_wda_derived_path() {
  local slug="${SIM_OS// /-}"   # "iOS 26.2" → "iOS-26.2"
  echo "/tmp/WDA_DerivedData_${slug}"
}

# ── WDA Touch Backend ─────────────────────────────────────────────────────────
#
# All touch injection goes through WebDriverAgent (WDA), which runs as an
# XCUITest runner inside each simulator. The flow is:
#   curl W3C Actions JSON → WDA HTTP server → XCTest framework →
#   testmanagerd socket (UDID-scoped) → synthetic IOHIDEvent
#
# Because the channel is scoped to a specific simulator UDID via testmanagerd,
# there is no window-focus dependency and no need for a global serialization
# queue. Multiple simulators can receive taps concurrently.
#
# Coordinate space: WDA accepts iOS UIKit logical points (not pixels, not macOS
# screen coordinates). The functions below receive macOS screen coordinates
# (from the internal to_screen_x/to_screen_y mapping) and reverse-map them
# back to logical points before sending to WDA.

# _tap_cgevent <screen_x> <screen_y>
# Despite the historical name, this uses WDA W3C pointer actions (not CGEvent).
# Accepts macOS screen coordinates and reverse-maps to iOS logical UIKit points.
# Sends a W3C Actions sequence: pointerMove → pointerDown → pause(100ms) → pointerUp.
# The request is UDID-scoped via the cached WDA session on WDA_PORT.
_tap_cgevent() {
  local sx="$1" sy="$2"
  _wda_ensure
  # Reverse map macOS screen coords → iOS logical points: logical = (screen - origin) / zoom
  local lx ly
  lx=$(awk "BEGIN { printf \"%d\", ($sx - $SIM_SCREEN_X) / $SIM_ZOOM + 0.5 }")
  ly=$(awk "BEGIN { printf \"%d\", ($sy - $SIM_SCREEN_Y) / $SIM_ZOOM + 0.5 }")

  local body
  body=$(python3 -c "import json; print(json.dumps({'actions':[{'type':'pointer','id':'f1','parameters':{'pointerType':'touch'},'actions':[{'type':'pointerMove','duration':0,'x':${lx},'y':${ly}},{'type':'pointerDown','button':0},{'type':'pause','duration':100},{'type':'pointerUp','button':0}]}]}))")

  local resp http_code
  resp=$(curl -s -w "\n%{http_code}" -X POST \
    -H "Content-Type: application/json" \
    -d "$body" \
    --max-time 10 \
    "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/actions" 2>/dev/null) || \
    die "WDA tap: curl failed at logical (${lx},${ly})"
  http_code=$(echo "$resp" | tail -1)
  [[ "$http_code" == "200" ]] \
    || die "WDA tap: HTTP ${http_code} at logical (${lx},${ly}). Response: $(echo "$resp" | head -1)"
}

# _swipe_cgevent <sx1> <sy1> <sx2> <sy2> [steps=25] [step_ms=20]
# Despite the historical name, this uses WDA W3C pointer actions (not CGEvent).
# Accepts macOS screen coordinates; reverse-maps to iOS logical UIKit points.
# Sends a W3C Actions sequence: pointerMove(start) → pointerDown → pointerMove(end,
# duration=steps*step_ms ms) → pointerUp. The duration parameter is how WDA
# interpolates the finger movement — longer duration = slower, smoother swipe.
# Default: 25 steps × 20ms = 500ms total swipe duration.
# All coordinates are UDID-scoped via testmanagerd — no global queue needed.
# _wda_swipe <lx1> <ly1> <lx2> <ly2> [duration_ms=500]
# Sends a WDA W3C pointer swipe directly in iOS logical coordinates.
# No macOS ↔ iOS coordinate conversion — avoids all orientation issues.
_wda_swipe() {
  local lx1="$1" ly1="$2" lx2="$3" ly2="$4"
  local duration_ms="${5:-500}"
  _wda_ensure

  local body
  body=$(python3 -c "import json; print(json.dumps({'actions':[{'type':'pointer','id':'f1','parameters':{'pointerType':'touch'},'actions':[{'type':'pointerMove','duration':0,'x':${lx1},'y':${ly1}},{'type':'pointerDown','button':0},{'type':'pointerMove','duration':${duration_ms},'x':${lx2},'y':${ly2}},{'type':'pointerUp','button':0}]}]}))")

  local resp http_code
  resp=$(curl -s -w "\n%{http_code}" -X POST \
    -H "Content-Type: application/json" \
    -d "$body" \
    --max-time 15 \
    "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/actions" 2>/dev/null) || \
    die "WDA swipe: curl failed from logical (${lx1},${ly1}) to (${lx2},${ly2})"
  http_code=$(echo "$resp" | tail -1)
  [[ "$http_code" == "200" ]] \
    || die "WDA swipe: HTTP ${http_code}. Response: $(echo "$resp" | head -1)"
}

# _wda_tap <lx> <ly>
# Sends a WDA W3C pointer tap directly in iOS logical coordinates.
_wda_tap() {
  local lx="$1" ly="$2"
  _wda_ensure

  local body
  body=$(python3 -c "import json; print(json.dumps({'actions':[{'type':'pointer','id':'f1','parameters':{'pointerType':'touch'},'actions':[{'type':'pointerMove','duration':0,'x':${lx},'y':${ly}},{'type':'pointerDown','button':0},{'type':'pause','duration':100},{'type':'pointerUp','button':0}]}]}))")

  local resp http_code
  resp=$(curl -s -w "\n%{http_code}" -X POST \
    -H "Content-Type: application/json" \
    -d "$body" \
    --max-time 10 \
    "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/actions" 2>/dev/null) || \
    die "WDA tap: curl failed at logical (${lx},${ly})"
  http_code=$(echo "$resp" | tail -1)
  [[ "$http_code" == "200" ]] \
    || die "WDA tap: HTTP ${http_code} at logical (${lx},${ly}). Response: $(echo "$resp" | head -1)"
}

_swipe_cgevent() {
  local sx1="$1" sy1="$2" sx2="$3" sy2="$4"
  local steps="${5:-25}"
  local step_ms="${6:-20}"
  local duration_ms=$(( steps * step_ms ))
  _wda_ensure
  # Reverse map macOS screen coords → iOS logical points (portrait-only, kept for compat)
  local lx1 ly1 lx2 ly2
  lx1=$(awk "BEGIN { printf \"%d\", ($sx1 - $SIM_SCREEN_X) / $SIM_ZOOM + 0.5 }")
  ly1=$(awk "BEGIN { printf \"%d\", ($sy1 - $SIM_SCREEN_Y) / $SIM_ZOOM + 0.5 }")
  lx2=$(awk "BEGIN { printf \"%d\", ($sx2 - $SIM_SCREEN_X) / $SIM_ZOOM + 0.5 }")
  ly2=$(awk "BEGIN { printf \"%d\", ($sy2 - $SIM_SCREEN_Y) / $SIM_ZOOM + 0.5 }")
  _wda_swipe "$lx1" "$ly1" "$lx2" "$ly2" "$duration_ms"
}

# _tap_wda <x> <y>
# Non-fatal WDA tap — returns 1 on failure instead of die-ing.
# Used by cmd_tap for graceful fallback to _tap_cgevent.
_tap_wda() {
  local x="$1" y="$2"
  # Ensure we have a WDA session
  _bootstrap || return 1

  local resp rc=0
  resp=$(curl -sf -X POST \
    -H "Content-Type: application/json" \
    -d "{\"actions\":[{\"type\":\"pointer\",\"id\":\"finger\",\"parameters\":{\"pointerType\":\"touch\"},\"actions\":[{\"type\":\"pointerMove\",\"duration\":0,\"x\":${x},\"y\":${y}},{\"type\":\"pointerDown\",\"button\":0},{\"type\":\"pause\",\"duration\":100},{\"type\":\"pointerUp\",\"button\":0}]}]}" \
    --max-time 10 \
    "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/actions" 2>/dev/null) || rc=$?

  if [[ $rc -ne 0 ]]; then
    return 1
  fi

  # Check for WDA error response
  local status
  status=$(echo "$resp" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('status', d.get('value',{}).get('error','')))" 2>/dev/null) || true
  if [[ -n "$status" ]] && [[ "$status" != "0" ]] && [[ "$status" != "" ]] && [[ "$status" != "None" ]]; then
    return 1
  fi

  return 0
}

# _swipe_wda <x1> <y1> <x2> <y2> [duration_ms=800]
# Non-fatal WDA swipe — returns 1 on failure instead of die-ing.
# Used by cmd_swipe for graceful fallback to _swipe_cgevent.
_swipe_wda() {
  local x1="$1" y1="$2" x2="$3" y2="$4" duration="${5:-800}"
  _bootstrap || return 1

  curl -sf -X POST \
    -H "Content-Type: application/json" \
    -d "{\"actions\":[{\"type\":\"pointer\",\"id\":\"finger\",\"parameters\":{\"pointerType\":\"touch\"},\"actions\":[{\"type\":\"pointerMove\",\"duration\":0,\"x\":${x1},\"y\":${y1}},{\"type\":\"pointerDown\",\"button\":0},{\"type\":\"pointerMove\",\"duration\":${duration},\"x\":${x2},\"y\":${y2}},{\"type\":\"pointerUp\",\"button\":0}]}]}" \
    --max-time 15 \
    "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/actions" \
    > /dev/null 2>&1 || return 1
}

# _wda_type <text>
# Sets text in the currently focused/active element via WDA's setValue endpoint.
# This goes through the proper iOS input path (unlike the AX direct-value approach).
# Requires WDA to be running (_wda_ensure must succeed).
# Returns 0 on success, 1 on failure (caller falls back to _type_cgevent).
_wda_type() {
  local text="$1"
  # Load WDA session — if not running, return 1 so caller can fall back
  if [[ -z "$WDA_PORT" || -z "$WDA_SESSION" ]]; then
    return 1
  fi
  # Quick liveness check
  local http_code
  http_code=$(curl -s -o /dev/null -w "%{http_code}" \
    --max-time 3 \
    "http://localhost:${WDA_PORT}/session/${WDA_SESSION}" 2>/dev/null) || true
  if [[ "$http_code" != "200" ]]; then
    return 1
  fi

  # Find the active/focused element
  local active_resp
  active_resp=$(curl -s --max-time 5 \
    "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/element/active" \
    2>/dev/null) || return 1

  local element_id
  element_id=$(echo "$active_resp" | python3 -c \
    "import sys,json; d=json.load(sys.stdin); v=d.get('value',{}); print(v.get('ELEMENT') or v.get('element-6066-11e4-a52e-4f735466cecf',''))" \
    2>/dev/null) || return 1
  [[ -n "$element_id" ]] || return 1

  # Set value on the active element — goes through proper iOS input path
  local body set_resp set_code
  body=$(python3 -c "import json,sys; print(json.dumps({'value': [sys.argv[1]], 'text': sys.argv[1]}))" "$text" 2>/dev/null) || return 1
  set_resp=$(curl -s -w "\n%{http_code}" -X POST \
    -H "Content-Type: application/json" \
    -d "$body" \
    --max-time 10 \
    "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/element/${element_id}/value" \
    2>/dev/null) || return 1
  set_code=$(echo "$set_resp" | tail -1)
  [[ "$set_code" == "200" ]] || return 1
  return 0
}

# _wda_ensure — called at the start of every tap and swipe to validate that
# WDA is running and the cached session is still alive. It checks two things:
#   1. WDA_PORT and WDA_SESSION are non-empty (loaded from cache or wda-start).
#   2. The session is live: GET /session/:id returns HTTP 200.
# If either check fails, it dies with an actionable error telling the agent to
# run `sim --device <UDID> wda-start <port>` to restart WDA.
# This guard ensures every tap/swipe is UDID-scoped and routed correctly —
# there is no fallback to CGEvent or any other global input injection.
_wda_ensure() {
  if [[ -z "$WDA_PORT" || -z "$WDA_SESSION" ]]; then
    local udid_hint="${SIM_TARGET_UDID:-<UDID>}"
    die "WDA not running. Start with: sim --device ${udid_hint} wda-start <port>"
  fi
  # Quick liveness check — GET /session/:id (WDA returns 200 if alive)
  local http_code
  http_code=$(curl -s -o /dev/null -w "%{http_code}" \
    --max-time 3 \
    "http://localhost:${WDA_PORT}/session/${WDA_SESSION}" 2>/dev/null) || true
  if [[ "$http_code" != "200" ]]; then
    # Session dead — clear cached values so the error message is clean
    WDA_PORT=""
    WDA_SESSION=""
    local udid_hint="${SIM_TARGET_UDID:-<UDID>}"
    die "WDA session is no longer alive (HTTP ${http_code:-timeout}). Restart with: sim --device ${udid_hint} wda-start <port>"
  fi
}

# ── WDA Commands ─────────────────────────────────────────────────────────────

# cmd_wda_start [port]
#
# _wda_patch_ios_compat <products_dir>
#
# Makes a WDA build (compiled with iOS 26+ SDK) backward-compatible with older
# iOS simulator runtimes (e.g. iOS 18). Two problems are fixed:
#
# 1. WebDriverAgentLib links _LocationEssentials.framework, which only exists in
#    iOS 26+ runtimes. Fix: redirect the load to CoreLocation.framework (which
#    exports the same symbols on all iOS versions) via install_name_tool -change.
#
# 2. WDA binaries have LC_BUILD_VERSION minos=26.x, causing xcodebuild to reject
#    them for older simulators. Fix: patch minos to 13.0 via xcrun vtool.
#
# Both patches are idempotent and preserve iOS 26 compatibility.
# Runs automatically after WDA is built in cmd_setup.
_wda_patch_ios_compat() {
  local products_dir="$1"
  local runner_app="$products_dir/Debug-iphonesimulator/WebDriverAgentRunner-Runner.app"
  [[ -d "$runner_app" ]] || return 0

  # Collect all WDA binaries that need patching
  local -a bins=()
  while IFS= read -r f; do
    bins+=("$f")
  done < <(find "$products_dir/Debug-iphonesimulator" \
    \( -name "WebDriverAgentLib" -o -name "WebDriverAgentRunner" -o -name "WebDriverAgentRunner-Runner" \) \
    -not -path "*/_CodeSignature/*" -type f 2>/dev/null)

  [[ ${#bins[@]} -gt 0 ]] || return 0

  local did_patch=false

  for bin in "${bins[@]}"; do
    local short
    short=$(echo "$bin" | sed "s|.*/Debug-iphonesimulator/||")

    # --- Redirect _LocationEssentials → CoreLocation ---
    if otool -L "$bin" 2>/dev/null | grep -q "_LocationEssentials"; then
      install_name_tool -change \
        "/System/Library/Frameworks/_LocationEssentials.framework/_LocationEssentials" \
        "/System/Library/Frameworks/CoreLocation.framework/CoreLocation" \
        "$bin" 2>/dev/null
      echo "    $short: redirected _LocationEssentials -> CoreLocation"
      did_patch=true
    fi

    # --- Patch minos to 13.0 (keep sdk unchanged) ---
    local current_minos
    current_minos=$(xcrun vtool -show-build "$bin" 2>/dev/null \
      | awk '/minos/{print $2}' | head -1) || true
    if [[ -n "$current_minos" && "$current_minos" != "13.0" ]]; then
      local current_sdk
      current_sdk=$(xcrun vtool -show-build "$bin" 2>/dev/null \
        | awk '/sdk/{print $2}' | head -1) || true
      xcrun vtool -set-build-version 7 13.0 "${current_sdk:-26.2}" \
        -replace -output "${bin}.patched" "$bin" 2>/dev/null
      mv "${bin}.patched" "$bin"
      echo "    $short: minos $current_minos -> 13.0"
      did_patch=true
    fi
  done

  if [[ "$did_patch" == "true" ]]; then
    # Re-sign all modified bundles (inner to outer)
    local top_wdalib="$products_dir/Debug-iphonesimulator/WebDriverAgentLib.framework"
    local xctest_dir="$runner_app/PlugIns/WebDriverAgentRunner.xctest"
    local inner_wdalib="$xctest_dir/Frameworks/WebDriverAgentLib.framework"
    [[ -d "$top_wdalib" ]]   && codesign --force --sign - "$top_wdalib"   2>/dev/null || true
    [[ -d "$inner_wdalib" ]] && codesign --force --sign - "$inner_wdalib" 2>/dev/null || true
    [[ -d "$xctest_dir" ]]   && codesign --force --sign - "$xctest_dir"   2>/dev/null || true
    [[ -d "$runner_app" ]]   && codesign --force --sign - "$runner_app"   2>/dev/null || true
    echo "    Re-signed all WDA bundles."
  else
    echo "    WDA binaries already compatible — no patch needed."
  fi
}

# cmd_wda_start [port]
#
# WebDriverAgent (WDA) is an Apple XCUITest-based HTTP server that runs inside
# the iOS Simulator. It exposes a W3C WebDriver-compatible REST API for touch
# injection, app control, and other automation primitives. All tap and swipe
# commands in this tool use WDA as their touch backend.
#
# This command uses `xcodebuild test-without-building` against a pre-built WDA
# binary located at /tmp/WDA_DerivedData_<iOS-X.Y> (per-iOS-version cache). The DerivedData directory must already
# exist (built once via `xcodebuild build` against the WebDriverAgent project).
# This avoids re-compiling WDA on every invocation.
#
# Port-to-UDID mapping: each running simulator needs its own port number so
# that multiple simulators can have WDA running simultaneously without conflict.
# Example: simulator A → port 8100, simulator B → port 8101.
# Always pass --device <UDID> together with wda-start so the correct simulator
# is targeted and the port/session are cached under that UDID's cache file.
#
# After WDA starts and its HTTP server is ready, this command POSTs to /session
# to create a W3C session. The resulting session ID and port are cached in
# /tmp/sim_bootstrap_cache_<UDID>.json so subsequent tap/swipe calls can reuse
# them without re-running wda-start.
cmd_wda_start() {
  local port="${1:-8100}"
  _bootstrap

  local wda_derived
  wda_derived=$(_wda_derived_path)
  # WDA must be pre-built — this path is created by building WebDriverAgent once.
  [[ -d "$wda_derived" ]] || die "WDA DerivedData not found at $wda_derived — run 'sim setup' first to build WDA for $SIM_OS"

  # Find the WDA test runner app bundle inside the pre-built DerivedData
  local wda_bundle
  wda_bundle=$(find "$wda_derived" -name "WebDriverAgentRunner-Runner.app" 2>/dev/null | head -1)
  [[ -n "$wda_bundle" ]] || die "WebDriverAgentRunner-Runner.app not found in $wda_derived"

  # Find xctestrun — check WDA_DerivedData first, then fall back to Appium's pre-built xctestrun
  local wda_xctestrun
  wda_xctestrun=$(find "$wda_derived" -name "*.xctestrun" 2>/dev/null | head -1)
  if [[ -z "$wda_xctestrun" ]]; then
    # Appium ships pre-built WDA xctestrun files for each iOS version — use the matching one
    local appium_wda_dir="$HOME/.appium/node_modules/appium-xcuitest-driver/node_modules/Build/Products"
    local os_ver
    os_ver=$(echo "$SIM_OS" | grep -oE '[0-9]+\.[0-9]+' | head -1)
    wda_xctestrun=$(find "$appium_wda_dir" -name "*iphonesimulator${os_ver}*.xctestrun" 2>/dev/null | head -1)
    if [[ -n "$wda_xctestrun" ]]; then
      echo "Using Appium pre-built WDA xctestrun: $wda_xctestrun"
    fi
  fi
  [[ -n "$wda_xctestrun" ]] || die "No .xctestrun file found in $wda_derived or Appium — rebuild WDA with build-for-testing"

  echo "Starting WDA for UDID=${SIM_UDID} on port ${port}..."

  # The xctestrun EnvironmentVariables block contains USE_PORT="" baked in at build time.
  # That empty-string entry overrides any host-level `env USE_PORT=N` prefix, causing WDA
  # to always fall back to its hardcoded default port (8100). Fix: patch a temp copy of the
  # xctestrun with the correct USE_PORT value before launching, so xcodebuild reads it.
  local patched_xctestrun
  local wda_products_dir
  wda_products_dir=$(dirname "$wda_xctestrun")
  patched_xctestrun="${wda_products_dir}/wda_patched_${port}.xctestrun"
  cp "$wda_xctestrun" "$patched_xctestrun"
  /usr/libexec/PlistBuddy -c \
    "Set :WebDriverAgentRunner:EnvironmentVariables:USE_PORT $port" \
    "$patched_xctestrun" 2>/dev/null || true

  # Launch WDA in the background using xcodebuild test-without-building.
  local wda_log="/tmp/wda_${SIM_UDID}.log"
  xcodebuild test-without-building \
    -xctestrun "$patched_xctestrun" \
    -destination "id=${SIM_UDID}" \
    > "$wda_log" 2>&1 &
  local wda_pid=$!
  echo "WDA process PID: $wda_pid (log: $wda_log)"

  # Wait up to 60s for WDA /status to become ready
  echo -n "Waiting for WDA HTTP server on port ${port}..."
  local attempts=0
  local max_attempts=60
  while [[ $attempts -lt $max_attempts ]]; do
    local status_code
    status_code=$(curl -s -o /dev/null -w "%{http_code}" \
      --max-time 2 "http://localhost:${port}/status" 2>/dev/null) || true
    if [[ "$status_code" == "200" ]]; then
      echo " ready."
      break
    fi
    echo -n "."
    sleep 1
    (( attempts++ )) || true
  done

  if [[ $attempts -ge $max_attempts ]]; then
    die "WDA did not become ready on port ${port} within ${max_attempts}s. Check log: $wda_log"
  fi

  # Create a W3C WebDriver session. WDA requires an active session before
  # accepting any /actions (tap/swipe) requests. The session ID is returned
  # in the response body and cached in the bootstrap cache for reuse.
  #
  # If WDA_APP_BUNDLE_ID is set, include it in the session capabilities so
  # WDA pins its active-application resolution to that specific app. This
  # prevents the background-window mismatch bug on iPad simulators where
  # WDA's hit-test heuristic (FBActiveAppDetectionPoint) selects a persistent
  # background window (e.g. the onboarding/auth flow) instead of the visible
  # foreground content.
  echo "Creating WDA session..."
  local session_caps session_resp
  if [[ -n "${WDA_APP_BUNDLE_ID:-}" ]]; then
    session_caps="{\"capabilities\":{\"alwaysMatch\":{\"bundleId\":\"${WDA_APP_BUNDLE_ID}\"}}}"
  else
    session_caps='{"capabilities":{"firstMatch":[{}]}}'
  fi
  session_resp=$(curl -s -X POST \
    -H "Content-Type: application/json" \
    -d "$session_caps" \
    --max-time 15 \
    "http://localhost:${port}/session" 2>/dev/null) \
    || die "Failed to POST /session to WDA on port ${port}"

  local session_id
  session_id=$(echo "$session_resp" | python3 -c \
    "import sys,json; d=json.load(sys.stdin); print(d.get('sessionId') or d.get('value',{}).get('sessionId',''))" \
    2>/dev/null) || true

  [[ -n "$session_id" ]] \
    || die "WDA session creation failed. Response: $session_resp"

  WDA_PORT="$port"
  WDA_SESSION="$session_id"

  # Belt-and-suspenders: pin defaultActiveApplication via Appium settings so
  # that even if the session capability above is ignored, WDA's element queries
  # resolve against the correct foreground app rather than any background window.
  if [[ -n "${WDA_APP_BUNDLE_ID:-}" ]]; then
    curl -s -X POST \
      -H "Content-Type: application/json" \
      -d "{\"settings\":{\"defaultActiveApplication\":\"${WDA_APP_BUNDLE_ID}\"}}" \
      --max-time 5 \
      "http://localhost:${port}/session/${WDA_SESSION}/appium/settings" \
      > /dev/null 2>&1 || true
    echo "WDA: pinned defaultActiveApplication to ${WDA_APP_BUNDLE_ID}"
  fi

  # Tune WDA serialization performance:
  # - snapshotMaxDepth 15: prune deep subtrees (default 50), biggest speed win
  # - snapshotMaxChildren 25: cap sibling count in list views
  # - pageSourceExcludedAttributes: skip computed-per-element attrs (visible, accessible)
  #   that WDA must derive rather than just read — confirmed significant speedup
  curl -s -X POST \
    -H "Content-Type: application/json" \
    -d '{"settings":{"snapshotMaxDepth":15,"snapshotMaxChildren":25,"pageSourceExcludedAttributes":"visible,accessible"}}' \
    --max-time 5 \
    "http://localhost:${port}/session/${WDA_SESSION}/appium/settings" \
    > /dev/null 2>&1 || true
  echo "WDA: applied performance settings (snapshotMaxDepth=15, snapshotMaxChildren=25)"

  # Persist port and session ID to the bootstrap cache so tap/swipe commands
  # in subsequent invocations can reuse the running WDA without re-starting it.
  _wda_write_cache

  echo "WDA ready: port=${WDA_PORT} session=${WDA_SESSION}"
}
