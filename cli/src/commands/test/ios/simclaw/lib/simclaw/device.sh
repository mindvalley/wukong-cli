# lib/simclaw/device.sh — simulator device detection and screen bounds
[[ -n "${_SIM_DEVICE_LOADED:-}" ]] && return 0; _SIM_DEVICE_LOADED=1

# ── Device Detection ──────────────────────────────────────────────────────────

detect_device() {
  local json
  json=$(xcrun simctl list devices --json 2>/dev/null) \
    || die "xcrun simctl failed. Ensure Xcode command-line tools are installed."

  # Find the target device: either by UDID (if --device was given) or the most recently booted.
  local chosen_json
  if [[ -n "$SIM_TARGET_UDID" ]]; then
    chosen_json=$(echo "$json" | jq --arg udid "$SIM_TARGET_UDID" '
      [
        .devices
        | to_entries[]
        | .key as $runtime
        | .value[]
        | select(.udid == $udid)
        | . + { runtime: $runtime }
      ]
      | first // empty
    ' 2>/dev/null) || die "jq failed parsing simctl output."
    [[ -n "$chosen_json" ]] && [[ "$chosen_json" != "null" ]] \
      || die "No simulator found with UDID: $SIM_TARGET_UDID"
  else
    chosen_json=$(echo "$json" | jq '
      [
        .devices
        | to_entries[]
        | .key as $runtime
        | .value[]
        | select(.state == "Booted")
        | . + { runtime: $runtime }
      ]
      | if length == 0 then empty
        else (sort_by(.lastBootedAt // "") | last)
        end
    ' 2>/dev/null) || die "jq failed parsing simctl output. Ensure jq is installed (brew install jq)."
    [[ -z "$chosen_json" ]] && die "No booted simulator found. Boot a simulator in Xcode first."
  fi

  IFS=$'\t' read -r SIM_UDID SIM_NAME SIM_DEVICE_TYPE < <(
    echo "$chosen_json" | jq -r '[.udid, .name, .deviceTypeIdentifier] | @tsv'
  )

  # Convert runtime key to readable OS string: e.g.
  # "com.apple.CoreSimulator.SimRuntime.iOS-18-2" → "iOS 18.2"
  local raw_runtime
  raw_runtime=$(echo "$chosen_json" | jq -r '.runtime')
  SIM_OS=$(echo "$raw_runtime" \
    | sed 's/com\.apple\.CoreSimulator\.SimRuntime\.//' \
    | sed 's/-/ /g' \
    | awk '{
        os = $1
        ver = ""
        for (i=2; i<=NF; i++) {
          if (i==2) ver = $i
          else ver = ver "." $i
        }
        print os " " ver
      }')
}

# ── Logical Resolution Lookup ─────────────────────────────────────────────────

lookup_logical_resolution() {
  local profiles_dir="/Library/Developer/CoreSimulator/Profiles/DeviceTypes"
  [[ -d "$profiles_dir" ]] \
    || die "CoreSimulator device profiles not found at: $profiles_dir"

  local bundle_path=""
  local candidate
  for candidate in "$profiles_dir"/*.simdevicetype; do
    [[ -d "$candidate" ]] || continue
    local plist="$candidate/Contents/Info.plist"
    [[ -f "$plist" ]] || continue
    local identifier
    identifier=$(plutil -extract CFBundleIdentifier raw "$plist" 2>/dev/null) || continue
    if [[ "$identifier" == "$SIM_DEVICE_TYPE" ]]; then
      bundle_path="$candidate"
      break
    fi
  done

  [[ -n "$bundle_path" ]] \
    || die "Could not find device type profile for: $SIM_DEVICE_TYPE"

  local profile_plist="$bundle_path/Contents/Resources/profile.plist"
  [[ -f "$profile_plist" ]] \
    || die "profile.plist not found in: $bundle_path/Contents/Resources/"

  local px_w px_h scale
  px_w=$(plutil -extract mainScreenWidth  raw "$profile_plist" 2>/dev/null | tr -d '[:space:]') \
    || die "Could not read mainScreenWidth from profile.plist"
  px_h=$(plutil -extract mainScreenHeight raw "$profile_plist" 2>/dev/null | tr -d '[:space:]') \
    || die "Could not read mainScreenHeight from profile.plist"
  scale=$(plutil -extract mainScreenScale raw "$profile_plist" 2>/dev/null | tr -d '[:space:]') \
    || die "Could not read mainScreenScale from profile.plist"

  [[ "$px_w" =~ ^[0-9]+$ ]] || die "mainScreenWidth is not a positive integer: '$px_w'"
  [[ "$px_h" =~ ^[0-9]+$ ]] || die "mainScreenHeight is not a positive integer: '$px_h'"

  SIM_SCALE="$scale"
  SIM_LOGICAL_W=$(awk "BEGIN { printf \"%d\", $px_w / $scale }")
  SIM_LOGICAL_H=$(awk "BEGIN { printf \"%d\", $px_h / $scale }")
  [[ "$SIM_LOGICAL_W" -gt 0 && "$SIM_LOGICAL_H" -gt 0 ]] \
    || die "Invalid logical resolution computed: ${SIM_LOGICAL_W}x${SIM_LOGICAL_H}"
}

# ── Screen Bounds Detection ───────────────────────────────────────────────────

detect_screen_bounds() {
  pgrep -x Simulator > /dev/null 2>&1 \
    || die "Simulator.app is not running. Launch the Simulator from Xcode first."

  local applescript_result
  applescript_result=$(osascript << APPLESCRIPT 2>/dev/null
tell application "System Events"
  tell process "Simulator"
    -- Find the target window by matching the full title "Name – OS" (e.g. "iPhone 16 – iOS 18.1")
    -- This distinguishes two simulators of the same model running different OS versions.
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

    -- Find screen element: ratio-match direct children, then try to go one level deeper
    -- to get inside the bezel frame to the actual screen surface
    -- Support both portrait (W/H) and landscape (H/W) orientations
    set portraitRatio to ${SIM_LOGICAL_W} / ${SIM_LOGICAL_H}
    set landscapeRatio to ${SIM_LOGICAL_H} / ${SIM_LOGICAL_W}
    set bestEl to missing value
    set bestScore to 999

    repeat with el in UI elements of targetWindow
      try
        set elSz to size of el
        set elW to item 1 of elSz
        set elH to item 2 of elSz
        if elW > 100 and elH > 100 then
          set ratio to elW / elH
          -- Check against portrait ratio
          set diff to ratio - portraitRatio
          if diff < 0 then set diff to -diff
          -- Also check against landscape ratio
          set ldiff to ratio - landscapeRatio
          if ldiff < 0 then set ldiff to -ldiff
          if ldiff < diff then set diff to ldiff
          if diff < bestScore then
            set bestScore to diff
            set bestEl to el
          end if
        end if
      end try
    end repeat

    if bestEl is missing value then
      return "ERROR: Could not find screen element in Simulator accessibility tree"
    end if
    if bestScore > 0.05 then
      return "ERROR: Best matching element ratio diff " & bestScore & " exceeds threshold"
    end if

    -- Try to descend one level: find a child whose aspect ratio is also within 5%
    -- This moves us from the bezel frame into the actual screen surface
    try
      repeat with child in UI elements of bestEl
        try
          set cSz to size of child
          set cW to item 1 of cSz
          set cH to item 2 of cSz
          if cW > 50 and cH > 50 then
            set cRatio to cW / cH
            set cDiff to cRatio - portraitRatio
            if cDiff < 0 then set cDiff to -cDiff
            set cLDiff to cRatio - landscapeRatio
            if cLDiff < 0 then set cLDiff to -cLDiff
            if cLDiff < cDiff then set cDiff to cLDiff
            if cDiff < 0.05 then
              set bestEl to child
              exit repeat
            end if
          end if
        end try
      end repeat
    end try

    set screenPos to position of bestEl
    set screenSz to size of bestEl
    set px to (item 1 of screenPos) as integer
    set py to (item 2 of screenPos) as integer
    set pw to (item 1 of screenSz) as integer
    set ph to (item 2 of screenSz) as integer
    return (px as text) & "," & (py as text) & "," & (pw as text) & "," & (ph as text)
  end tell
end tell
APPLESCRIPT
  )

  if [[ -z "$applescript_result" ]] || [[ "$applescript_result" == ERROR:* ]]; then
    die "AppleScript failed to detect screen bounds. Ensure Terminal has Accessibility permission in System Settings > Privacy & Security > Accessibility. Detail: ${applescript_result:-empty response}"
  fi

  SIM_SCREEN_X=$(echo "$applescript_result" | cut -d',' -f1 | tr -d ' ')
  SIM_SCREEN_Y=$(echo "$applescript_result" | cut -d',' -f2 | tr -d ' ')
  SIM_SCREEN_W=$(echo "$applescript_result" | cut -d',' -f3 | tr -d ' ')
  SIM_SCREEN_H=$(echo "$applescript_result" | cut -d',' -f4 | tr -d ' ')

  # In landscape mode (screen_w > screen_h), the device is rotated so the macOS
  # screen width corresponds to iOS logical height (and vice versa). Compute zoom
  # from the axis that is larger in both spaces to get the correct scale factor.
  local orient_for_zoom
  orient_for_zoom=$(plutil -extract "DevicePreferences.${SIM_UDID}.SimulatorWindowOrientation" raw \
    ~/Library/Preferences/com.apple.iphonesimulator.plist 2>/dev/null || echo "Portrait")
  case "$orient_for_zoom" in
    LandscapeLeft|LandscapeRight)
      SIM_ZOOM=$(awk "BEGIN { printf \"%.4f\", $SIM_SCREEN_W / $SIM_LOGICAL_H }")
      ;;
    *)
      SIM_ZOOM=$(awk "BEGIN { printf \"%.4f\", $SIM_SCREEN_W / $SIM_LOGICAL_W }")
      ;;
  esac

  # Sanity check: screen dimensions must be non-zero and zoom must be plausible
  [[ "$SIM_SCREEN_W" -gt 0 && "$SIM_SCREEN_H" -gt 0 ]] \
    || die "AppleScript returned invalid screen size: ${SIM_SCREEN_W}x${SIM_SCREEN_H}. The Simulator window accessibility tree may have changed — try restarting Simulator."
  local zoom_ok
  zoom_ok=$(awk "BEGIN { z=${SIM_ZOOM}; print (z >= 0.4 && z <= 6.0) ? \"ok\" : \"bad\" }")
  [[ "$zoom_ok" == "ok" ]] \
    || die "Computed zoom factor ${SIM_ZOOM} is outside plausible range (0.4–6.0). Screen bounds detection likely failed."
}
