# lib/simclaw/nav.sh — tap-element and tap-and-wait compound navigation
[[ -n "${_SIM_NAV_LOADED:-}" ]] && return 0; _SIM_NAV_LOADED=1

source "$SIM_LIB/bootstrap.sh"
source "$SIM_LIB/wda.sh"
source "$SIM_LIB/coords.sh"
source "$SIM_LIB/inspect.sh"
source "$SIM_LIB/wait.sh"
source "$SIM_LIB/layout_map.sh"

cmd_tap_element() {
  # Parse args: <label> [--role <substring>]
  # --role disambiguates when the same label exists on multiple element types
  # on the same screen. Common case: a settings menu row labeled "Search"
  # plus the search field at the bottom of the screen also labeled "Search"
  # — without --role the tap target is ambiguous and may pick the wrong one.
  # The role value is matched as a case-insensitive substring against both
  # the WDA tag (XCUIElementTypeTextField, etc.) and the AX role
  # (AXTextField, etc.), so `--role textfield` works regardless of which
  # path resolves the element.
  local label=""
  local role_filter=""
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --role) [[ $# -ge 2 ]] || die "tap-element: --role requires a value"
              role_filter="$2"; shift 2 ;;
      *)      [[ -z "$label" ]] || die "tap-element: unexpected argument '$1'"
              label="$1"; shift ;;
    esac
  done
  [[ -n "$label" ]] || die "Usage: sim tap-element <label> [--role <substring>]"
  _bootstrap

  # Search WDA /source (viewport only — avoids cross-tab search pollution).
  # /source returns only elements currently rendered on screen; /elements searches
  # all loaded app contexts including background tabs, causing wrong-target taps.
  local wda_xml
  wda_xml=$(curl -sf --max-time 10 "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/source" 2>/dev/null)
  [[ -n "$wda_xml" ]] || die "tap-element: WDA /source unavailable"

  local tmp_xml tmp_py
  tmp_xml=$(mktemp -t sim_tapelement_xml)
  tmp_py=$(mktemp -t sim_tapelement_py)
  printf '%s' "$wda_xml" > "$tmp_xml"

  cat > "$tmp_py" << 'PYEOF'
import sys, json, xml.etree.ElementTree as ET

search_label = sys.argv[1]
screen_w     = int(sys.argv[2])
screen_h     = int(sys.argv[3])
xml_path     = sys.argv[4]
role_filter  = sys.argv[5].lower() if len(sys.argv) > 5 else ""

raw = open(xml_path).read()
try:
    parsed = json.loads(raw)
    raw_xml = parsed.get("value", raw) if isinstance(parsed, dict) else raw
except Exception:
    raw_xml = raw

root = ET.fromstring(raw_xml)

def elem_name(el):
    lbl = el.get("label", "")
    nm  = el.get("name", "")
    return lbl if lbl else nm

lower_search = search_label.lower()
best = None
best_score = -1

for el in root.iter():
    # Apply --role filter if set: tag is e.g. "XCUIElementTypeTextField",
    # so a substring check against role_filter (already lowercased) lets the
    # caller pass "textfield", "TextField", "AXTextField", etc. and have it
    # match the same set of elements regardless of casing or AX/XCUI prefix.
    if role_filter and role_filter not in el.tag.lower():
        continue
    n = elem_name(el)
    if not n:
        continue
    n_lower = n.lower()
    # Prefer exact match, then prefix, then contains
    if n_lower == lower_search:
        score = 3
    elif n_lower.startswith(lower_search):
        score = 2
    elif lower_search in n_lower:
        score = 1
    else:
        continue

    # Parse bounding box
    try:
        x = int(el.get("x", -1))
        y = int(el.get("y", -1))
        w = int(el.get("width", 0))
        h = int(el.get("height", 0))
    except (TypeError, ValueError):
        continue

    # Must be fully within viewport
    if x < 0 or y < 0 or w <= 0 or h <= 0:
        continue
    if (x + w) > screen_w or (y + h) > screen_h:
        continue

    if score > best_score:
        best_score = score
        best = {"label": n, "x": x, "y": y, "w": w, "h": h,
                "cx": x + w // 2, "cy": y + h // 2}

if best:
    print(json.dumps(best))
else:
    sys.exit(1)
PYEOF

  local found
  found=$(python3 "$tmp_py" "$label" "$SIM_LOGICAL_W" "$SIM_LOGICAL_H" "$tmp_xml" "$role_filter" 2>/dev/null) || true
  rm -f "$tmp_xml" "$tmp_py"

  local cx cy
  if [[ -n "$found" ]]; then
    cx=$(echo "$found" | python3 -c "import sys,json; print(json.load(sys.stdin)['cx'])" 2>/dev/null)
    cy=$(echo "$found" | python3 -c "import sys,json; print(json.load(sys.stdin)['cy'])" 2>/dev/null)
  else
    # ── AX-grid fallback ────────────────────────────────────────────────────
    # WDA /source didn't include this label. Common cases that hit this path:
    #   - System dialogs (notification permission, location prompts) live in
    #     SpringBoard / system UI overlay, not the foreground app's snapshot.
    #   - Some XCUIElement subclasses are simply missing from XCUITest's
    #     snapshot() — e.g., the bottom-right `+` button on Reminders main.
    #   - Dynamically-injected toolbars or overlays may not be captured yet.
    #
    # The AX hit-test path (`describe-point`) sees these elements correctly
    # because it queries the live macOS AX tree from the simulator window
    # server. We scan a coarse grid (50pt spacing → ~150 probes for a 4-inch
    # screen) in one batched call, then pick the best label match. The grid
    # is sparse enough that a single call finishes in ~1-2s — paid only when
    # the WDA path missed.
    local lw="${SIM_LOGICAL_W:-402}"
    local lh="${SIM_LOGICAL_H:-874}"
    local grid_input=""
    local gx gy
    # 40pt step starting at y=20 / x=20. The 20pt offset puts probes in the
    # MIDDLE of typical 44pt-tall iOS HIG control bands rather than on their
    # edges; AX hit-test at the exact rect edge sometimes returns the parent
    # AXGroup instead of the control (we observed this on the Settings root
    # search field at y=802-840, which AX returns as AXGroup at y=800/840 but
    # AXTextField at y=820). 40pt step ensures any 44pt control gets at least
    # one mid-band probe regardless of where it sits.
    for ((gy = 20; gy < lh; gy += 40)); do
      for ((gx = 20; gx < lw; gx += 40)); do
        grid_input+="$gx $gy"$'\n'
      done
    done

    local grid_results
    grid_results=$(printf '%s' "$grid_input" | _describe_points_batch)

    # Pick the best AX hit whose label matches (case-insensitive contains),
    # restricted to interactive roles. Preference: exact > prefix > contains.
    local needle_lower
    needle_lower=$(echo "$label" | tr '[:upper:]' '[:lower:]')
    local fallback_match
    local role_lower=""
    [[ -n "$role_filter" ]] && role_lower=$(echo "$role_filter" | tr '[:upper:]' '[:lower:]')
    fallback_match=$(echo "$grid_results" | NEEDLE="$needle_lower" ROLE="$role_lower" python3 -c "
import json, os, sys
needle = os.environ.get('NEEDLE', '')
role_filter = os.environ.get('ROLE', '')
ROLES = {'AXButton', 'AXTextField', 'AXSecureTextField', 'AXSearchField',
         'AXSwitch', 'AXSlider', 'AXLink', 'AXMenuItem', 'AXCheckBox'}
best = None
best_score = -1
seen = set()
for line in sys.stdin:
    line = line.strip()
    if not line or line == 'null': continue
    try: hit = json.loads(line)
    except Exception: continue
    role = hit.get('role') or ''
    if role not in ROLES: continue
    if role_filter and role_filter not in role.lower(): continue
    lbl = (hit.get('label') or '').lower()
    if not lbl: continue
    # Dedupe identical hits (the grid often hits the same large element
    # multiple times) so the scoring isn't biased by element size.
    key = (hit.get('role'), lbl, hit.get('x'), hit.get('y'))
    if key in seen: continue
    seen.add(key)
    if lbl == needle: score = 3
    elif lbl.startswith(needle): score = 2
    elif needle in lbl: score = 1
    else: continue
    if score > best_score:
        best_score = score
        best = hit
if best:
    cx_ = int(best.get('x', 0) + best.get('w', 0) / 2)
    cy_ = int(best.get('y', 0) + best.get('h', 0) / 2)
    print(cx_)
    print(cy_)
    print(best.get('label', ''))
" 2>/dev/null)

    if [[ -z "$fallback_match" ]]; then
      die "tap-element: '$label' not found in WDA /source or in AX grid scan"
    fi

    local fallback_label
    { read -r cx; read -r cy; read -r fallback_label; } <<< "$fallback_match"
    echo "NOTE: '$label' not in WDA /source — found via AX scan as '$fallback_label' at ($cx, $cy)"
    _wda_tap "$cx" "$cy"
    echo "OK: tapped $label at ($cx, $cy)"
    return 0
  fi

  # ── Verify WDA's reported center against the live AX hit-test ──────────────
  # XCUITest's snapshot() (which feeds WDA /source) returns layout-intended
  # frames. SwiftUI flex layouts (Spacer, .frame(maxHeight:.infinity), etc.)
  # collapse at render time — so an element WDA reports at y=663 may render
  # at y=460. Tapping at WDA's coords lands in empty space.
  #
  # Same two-phase pattern as layout-map's verify pass, scoped to a single
  # element: hit-test at WDA's center; if mismatch, walk ±300pt in 30pt steps
  # via one batched describe-points call to find the rendered position.
  local verify_lbl_lower
  verify_lbl_lower=$(echo "$label" | tr '[:upper:]' '[:lower:]')
  local hit_json hit_lbl
  hit_json=$(cmd_describe_point "$cx" "$cy" 2>/dev/null)
  hit_lbl=$(echo "$hit_json" | python3 -c "
import sys, json
try: print(json.load(sys.stdin).get('label',''))
except Exception: pass
" 2>/dev/null)
  if [[ -z "$hit_lbl" ]] || \
     ! echo "$hit_lbl" | tr '[:upper:]' '[:lower:]' | grep -qF "$verify_lbl_lower"; then
    # WDA's center was wrong. Walk ±300pt to find the rendered position.
    local walk_input walk_results
    walk_input=$(for off in -30 -60 -90 -120 -150 -180 -210 -240 -270 -300 \
                            30  60  90 120 150 180 210 240 270 300; do
      local sy=$((cy + off))
      [[ "$sy" -lt 50 ]] && echo "" || echo "$cx $sy"
    done)
    walk_results=$(printf '%s\n' "$walk_input" | _describe_points_batch)
    local correction
    correction=$(echo "$walk_results" | LBL="$verify_lbl_lower" python3 -c "
import json, os, sys
needle = os.environ.get('LBL', '')
for line in sys.stdin:
    line = line.strip()
    if not line or line == 'null': continue
    try: hit = json.loads(line)
    except Exception: continue
    if needle in (hit.get('label') or '').lower():
        x = hit.get('x', 0); y = hit.get('y', 0)
        w = hit.get('w', 0); h = hit.get('h', 0)
        print(int(x + w/2), int(y + h/2))
        sys.exit(0)
")
    if [[ -n "$correction" ]]; then
      read -r cx cy <<< "$correction"
      echo "NOTE: WDA reported wrong center for '$label' — corrected via AX hit-test to ($cx, $cy)"
    fi
  fi

  _wda_tap "$cx" "$cy"
  echo "OK: tapped $label at ($cx, $cy)"

  # Post-tap check: see if element is gone (navigation likely succeeded)
  sleep 0.4
  local post_xml
  post_xml=$(curl -sf --max-time 10 "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/source" 2>/dev/null)
  if [[ -n "$post_xml" ]]; then
    local tmp_post_xml tmp_post_py still_visible
    tmp_post_xml=$(mktemp -t sim_tapelement_post)
    tmp_post_py=$(mktemp -t sim_tapelement_postpy)
    printf '%s' "$post_xml" > "$tmp_post_xml"
    cat > "$tmp_post_py" << 'POSTEOF'
import sys, json, xml.etree.ElementTree as ET
search_label = sys.argv[1].lower()
xml_path = sys.argv[2]
raw = open(xml_path).read()
try:
    parsed = json.loads(raw)
    raw_xml = parsed.get("value", raw) if isinstance(parsed, dict) else raw
except Exception:
    raw_xml = raw
root = ET.fromstring(raw_xml)
for el in root.iter():
    lbl = el.get("label", "")
    nm  = el.get("name", "")
    n = (lbl if lbl else nm).lower()
    if search_label in n:
        sys.exit(0)
sys.exit(1)
POSTEOF
    python3 "$tmp_post_py" "$label" "$tmp_post_xml" 2>/dev/null && still_visible=0 || still_visible=$?
    rm -f "$tmp_post_xml" "$tmp_post_py"
    if [[ $still_visible -ne 0 ]]; then
      echo "NOTE: '$label' no longer visible — navigation likely succeeded"
    fi
  fi
}

cmd_tap_and_wait() {
  [[ $# -ge 1 ]] || die "Usage: sim tap-and-wait <label> [expected_label] [timeout]"
  local label="$1"
  local expected_label="${2:-}"
  local timeout="${3:-10}"
  _bootstrap

  # Step 1: locate element via find-element
  local json=""
  if ! json=$(cmd_find_element "$label" 2>/dev/null); then
    die "tap-and-wait: element not found: $label"
  fi

  # Step 2: compute center coordinates
  local lx ly lw lh cx cy
  lx=$(echo "$json" | jq -r '.x')
  ly=$(echo "$json" | jq -r '.y')
  lw=$(echo "$json" | jq -r '.w')
  lh=$(echo "$json" | jq -r '.h')
  cx=$(awk "BEGIN { printf \"%d\", $lx + $lw / 2 }")
  cy=$(awk "BEGIN { printf \"%d\", $ly + $lh / 2 }")

  # Step 3: tap at center using CGEvent
  local screen_coords sx sy
  screen_coords=$(_ios_to_screen "$cx" "$cy")
  sx=$(echo "$screen_coords" | awk '{print $1}')
  sy=$(echo "$screen_coords" | awk '{print $2}')
  _tap_cgevent "$sx" "$sy" || die "tap-and-wait: tap failed at ($cx, $cy)"
  echo "OK: tapped '$label' at ($cx, $cy)" >&2

  # Step 4: wait for expected label or settle
  if [[ -n "$expected_label" ]]; then
    cmd_wait_for "$expected_label" "$timeout" || \
      echo "WARNING: timed out waiting for '$expected_label' after ${timeout}s" >&2
  else
    sleep 1.5
  fi

  # Step 5: return layout-map JSON
  cmd_layout_map
}
