# lib/simclaw/setup.sh — one-shot simulator setup
[[ -n "${_SIM_SETUP_LOADED:-}" ]] && return 0; _SIM_SETUP_LOADED=1

source "$SIM_LIB/bootstrap.sh"
source "$SIM_LIB/wda.sh"

cmd_setup() {
  # Parse arguments: <app_path_or_bundle_id> [--port <wda_port>]
  local app_arg=""
  local wda_port="8100"

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --port)
        [[ $# -ge 2 ]] || die "setup: --port requires a value"
        wda_port="$2"
        shift 2
        ;;
      *)
        [[ -z "$app_arg" ]] || die "setup: unexpected argument: $1"
        app_arg="$1"
        shift
        ;;
    esac
  done

  [[ -n "$app_arg" ]] || die "Usage: sim [--device <UDID>] setup <app_path_or_bundle_id> [--port <wda_port>]"

  # ── Step 1: Detect / boot simulator ─────────────────────────────────────────

  echo "==> [1/5] Detecting simulator..."

  # If a specific UDID was given, verify it exists and boot it if needed.
  if [[ -n "$SIM_TARGET_UDID" ]]; then
    local sim_state
    sim_state=$(xcrun simctl list devices --json 2>/dev/null \
      | jq -r --arg udid "$SIM_TARGET_UDID" '
          .devices | to_entries[] | .value[] | select(.udid == $udid) | .state
        ' 2>/dev/null | head -1) || true

    [[ -n "$sim_state" ]] \
      || die "setup: no simulator found with UDID: $SIM_TARGET_UDID"

    if [[ "$sim_state" != "Booted" ]]; then
      echo "    Booting simulator $SIM_TARGET_UDID..."
      xcrun simctl boot "$SIM_TARGET_UDID" 2>/dev/null || true
    else
      echo "    Simulator $SIM_TARGET_UDID already booted."
    fi

    echo "    Waiting for simulator to fully boot..."
    xcrun simctl bootstatus "$SIM_TARGET_UDID" -b \
      || die "setup: simulator failed to reach booted state"
  fi

  # Open Simulator.app if not already visible.
  if ! pgrep -x Simulator > /dev/null 2>&1; then
    echo "    Opening Simulator.app..."
    open -a Simulator
    # Give Simulator.app a moment to render its window so AppleScript can see it.
    sleep 2
  else
    echo "    Simulator.app is already running."
  fi

  # Run _bootstrap to populate all SIM_* globals (uses detect_device if no --device was given).
  _bootstrap
  echo "    Target: $SIM_NAME ($SIM_OS) — UDID: $SIM_UDID"

  # ── Step 2: Build WDA if not already built ──────────────────────────────────

  echo "==> [2/5] Checking WDA build..."

  local wda_source="/tmp/WebDriverAgent"
  local wda_derived
  wda_derived=$(_wda_derived_path)
  local wda_runner="$wda_derived/Build/Products/Debug-iphonesimulator/WebDriverAgentRunner-Runner.app"
  local patch_stamp="$wda_derived/.patch_version"

  if [[ ! -d "$wda_source" ]]; then
    echo ""
    echo "Error: WebDriverAgent not found at $wda_source"
    echo "Clone it first: git clone https://github.com/appium/WebDriverAgent $wda_source"
    exit 1
  fi

  # Cache reuse is gated on TWO conditions: the runner app exists AND the cached
  # patch version matches WDA_PATCH_VERSION. If either fails we rebuild + re-patch
  # so a stale cache (e.g. built before _wda_patch_ios_compat existed, or against
  # the wrong iOS SDK) can't silently break WDA launch on the current runtime.
  local cached_stamp=""
  [[ -f "$patch_stamp" ]] && cached_stamp=$(cat "$patch_stamp" 2>/dev/null || true)

  if [[ -d "$wda_runner" && "$cached_stamp" == "$WDA_PATCH_VERSION" ]]; then
    echo "    WDA already built and patched (v$WDA_PATCH_VERSION) for $SIM_OS — skipping build."
  else
    if [[ -d "$wda_runner" && "$cached_stamp" != "$WDA_PATCH_VERSION" ]]; then
      echo "    WDA cache patch version mismatch (cached='$cached_stamp', current='$WDA_PATCH_VERSION') for $SIM_OS — rebuilding."
      rm -rf "$wda_derived"
    fi
    echo "    Building WDA for $SIM_OS (this may take a few minutes)..."
    xcodebuild build-for-testing \
      -project "$wda_source/WebDriverAgent.xcodeproj" \
      -scheme WebDriverAgentRunner \
      -destination "platform=iOS Simulator,id=${SIM_UDID}" \
      -derivedDataPath "$wda_derived" \
      CODE_SIGNING_ALLOWED=NO 2>&1 | tail -5
    [[ -d "$wda_runner" ]] \
      || die "setup: WDA build completed but runner app not found at $wda_runner"
    echo "    WDA build complete."

    # Patch WebDriverAgentLib to weak-link _LocationEssentials so a binary built
    # against the iOS 26+ SDK still launches on older iOS simulator runtimes
    # (< 26) where that framework does not exist. Also rewrites LC_BUILD_VERSION
    # minos to 13.0 so xcodebuild won't reject the binary for older sims.
    _wda_patch_ios_compat "$wda_derived/Build/Products"

    # Stamp the build with the current patch version so subsequent setup runs
    # can detect whether a re-patch (or full rebuild) is required.
    echo "$WDA_PATCH_VERSION" > "$patch_stamp"
  fi

  # ── Step 3: Install app (if a .app path was given) ──────────────────────────

  echo "==> [3/5] Installing app..."

  local bundle_id=""
  if [[ "$app_arg" == *.app ]]; then
    echo "    Installing $app_arg on $SIM_UDID..."
    xcrun simctl install "$SIM_UDID" "$app_arg" \
      || die "setup: xcrun simctl install failed for $app_arg"
    echo "    App installed."

    # Derive bundle ID from the installed .app's Info.plist.
    local info_plist="$app_arg/Info.plist"
    [[ -f "$info_plist" ]] \
      || die "setup: Info.plist not found in $app_arg"
    bundle_id=$(plutil -extract CFBundleIdentifier raw "$info_plist" 2>/dev/null) \
      || die "setup: could not read CFBundleIdentifier from $info_plist"
    echo "    Bundle ID: $bundle_id"
  else
    bundle_id="$app_arg"
    echo "    Bundle ID provided directly ($bundle_id) — skipping install."
  fi

  # ── Step 4: Start WDA (graceful — setup continues if WDA fails) ─────────────

  echo "==> [4/5] Starting WDA on port $wda_port..."
  local wda_available="true"
  # Run cmd_wda_start in a subshell so its internal `die` (exit 1) does not
  # kill the parent setup process. WDA is expected to fail on older iOS
  # versions where the WDA xctest binary's MinimumOSVersion exceeds the
  # simulator's OS version (e.g. WDA built with iOS 26 SDK on an iOS 18 sim).
  if ( cmd_wda_start "$wda_port" ); then
    # cmd_wda_start wrote port+session to the bootstrap cache — read them back
    # into the parent shell (subshell variables don't propagate).
    local cache_file="/tmp/sim_bootstrap_cache_${SIM_UDID}.json"
    if [[ -f "$cache_file" ]]; then
      WDA_PORT=$(python3 -c "import json; print(json.load(open('$cache_file')).get('wda_port',''))" 2>/dev/null) || true
      WDA_SESSION=$(python3 -c "import json; print(json.load(open('$cache_file')).get('wda_session',''))" 2>/dev/null) || true
    fi
  else
    wda_available="false"
    WDA_PORT=""
    WDA_SESSION=""
    echo ""
    echo "    WDA failed to start — continuing without WDA."
    echo "    (Expected on iOS versions with WDA SDK mismatch.)"
    echo "    Screenshot and AX-based commands (describe, wait-for) still work."
  fi

  # ── Step 5: Launch app ───────────────────────────────────────────────────────

  echo "==> [5/5] Launching $bundle_id..."
  xcrun simctl launch "$SIM_UDID" "$bundle_id" \
    || die "setup: xcrun simctl launch failed for bundle ID: $bundle_id"
  echo "    App launched."

  # ── Final status ─────────────────────────────────────────────────────────────

  echo ""
  echo "── Setup complete ───────────────────────────────────────────────────────────"
  echo "  Simulator:  $SIM_NAME ($SIM_OS)"
  echo "  UDID:       $SIM_UDID"
  if [[ "$wda_available" == "true" ]]; then
    echo "  WDA port:   $WDA_PORT"
    echo "  WDA session: $WDA_SESSION"
    echo "  WDA:        ready"
  else
    echo "  WDA:        unavailable (screenshot + AX commands only)"
  fi
  echo "  App:        $bundle_id"
  echo ""
  if [[ "$wda_available" == "true" ]]; then
    echo "WDA is ready. You can now use tap/swipe/scroll commands."
  else
    echo "Setup complete (no WDA). Use screenshot/describe/wait-for commands."
  fi
}
