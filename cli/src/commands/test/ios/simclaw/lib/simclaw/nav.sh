# lib/simclaw/nav.sh — tap-element and tap-and-wait compound navigation
[[ -n "${_SIM_NAV_LOADED:-}" ]] && return 0; _SIM_NAV_LOADED=1

source "$SIM_LIB/bootstrap.sh"
source "$SIM_LIB/wda.sh"
source "$SIM_LIB/coords.sh"
source "$SIM_LIB/inspect.sh"
source "$SIM_LIB/wait.sh"
source "$SIM_LIB/layout_map.sh"

cmd_tap_element() {
  [[ $# -ge 1 ]] || die "Usage: sim tap-element <label>"
  local label="$1"
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
  found=$(python3 "$tmp_py" "$label" "$SIM_LOGICAL_W" "$SIM_LOGICAL_H" "$tmp_xml" 2>/dev/null) || true
  rm -f "$tmp_xml" "$tmp_py"

  [[ -n "$found" ]] || die "tap-element: '$label' not found in current viewport"

  local cx cy
  cx=$(echo "$found" | python3 -c "import sys,json; print(json.load(sys.stdin)['cx'])" 2>/dev/null)
  cy=$(echo "$found" | python3 -c "import sys,json; print(json.load(sys.stdin)['cy'])" 2>/dev/null)

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
