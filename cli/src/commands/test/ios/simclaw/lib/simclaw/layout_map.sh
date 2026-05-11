# lib/simclaw/layout_map.sh — layout-map command (primary navigation tool)
[[ -n "${_SIM_LAYOUT_MAP_LOADED:-}" ]] && return 0; _SIM_LAYOUT_MAP_LOADED=1

source "$SIM_LIB/bootstrap.sh"
source "$SIM_LIB/wda.sh"
source "$SIM_LIB/inspect.sh"

# Each class-name query completes in <1s vs the multi-second /source serialization.
_layout_map_elements_fallback() {
  local layout_filter="${1:-all}"
  local lw="${SIM_LOGICAL_W:-393}"
  local lh="${SIM_LOGICAL_H:-852}"
  local base="http://localhost:${WDA_PORT}/session/${WDA_SESSION}"

  # ── Build per-class element-query helper script ───────────────────────────────
  local tmp_el_py tmp_asm tmp_enrich
  tmp_el_py=$(mktemp -t sim_el_query)
  tmp_asm=$(mktemp -t sim_el_asm)
  tmp_enrich=$(mktemp -t sim_el_enrich)

  # element_query.py: POST /elements by class name, fetch rect+label per element
  python3 -c "
import textwrap, sys
script = textwrap.dedent('''
    import sys, json, urllib.request
    base = sys.argv[1]
    class_name = sys.argv[2]
    logical_h = int(sys.argv[3]) if len(sys.argv) > 3 else 852
    max_els = int(sys.argv[4]) if len(sys.argv) > 4 else 40

    def wda_get(path, timeout=5):
        try:
            with urllib.request.urlopen(base + path, timeout=timeout) as r:
                return json.loads(r.read())
        except Exception:
            return {}

    def wda_post(path, data, timeout=5):
        body = json.dumps(data).encode()
        req = urllib.request.Request(base + path, data=body,
                                     headers={\"Content-Type\": \"application/json\"})
        try:
            with urllib.request.urlopen(req, timeout=timeout) as r:
                return json.loads(r.read())
        except Exception:
            return {}

    resp = wda_post(\"/elements\", {\"using\": \"class name\", \"value\": class_name})
    els = resp.get(\"value\", [])
    if not isinstance(els, list):
        print(\"[]\"); sys.exit(0)

    results = []
    seen = set()
    for el in els[:max_els]:
        eid = el.get(\"ELEMENT\") or el.get(\"element-6066-11e4-a52e-4f735466cecf\", \"\")
        if not eid: continue
        lbl = (wda_get(f\"/element/{eid}/attribute/label\")).get(\"value\") or \"\"
        nm  = (wda_get(f\"/element/{eid}/attribute/name\")).get(\"value\") or \"\"
        label = lbl if lbl else nm
        r2 = (wda_get(f\"/element/{eid}/rect\")).get(\"value\", {})
        x = int(r2.get(\"x\", 0)); y = int(r2.get(\"y\", 0))
        w = int(r2.get(\"width\", 0)); h = int(r2.get(\"height\", 0))
        if w == 0 or h == 0: continue
        key = label + f\"|{x},{y}\"
        if key in seen: continue
        seen.add(key)
        if y > logical_h * 1.5: continue
        results.append({\"label\": label, \"x\": x, \"y\": y, \"w\": w, \"h\": h})
    print(json.dumps(results))
''').lstrip()
sys.stdout.write(script)
" > "$tmp_el_py"

  # ── Query WDA by class name ──────────────────────────────────────────────────
  local btn_json tabbar_json navbar_json tf_json sw_json sf_json
  btn_json=$(python3 "$tmp_el_py" "$base" "XCUIElementTypeButton" "$lh" "50") || btn_json="[]"
  tabbar_json=$(python3 "$tmp_el_py" "$base" "XCUIElementTypeTabBar" "$lh" "1") || tabbar_json="[]"
  navbar_json=$(python3 "$tmp_el_py" "$base" "XCUIElementTypeNavigationBar" "$lh" "1") || navbar_json="[]"
  if [[ "$layout_filter" != "buttons" ]]; then
    tf_json=$(python3 "$tmp_el_py" "$base" "XCUIElementTypeTextField" "$lh" "10") || tf_json="[]"
    sw_json=$(python3 "$tmp_el_py" "$base" "XCUIElementTypeSwitch" "$lh" "10") || sw_json="[]"
    sf_json=$(python3 "$tmp_el_py" "$base" "XCUIElementTypeSearchField" "$lh" "5") || sf_json="[]"
  else
    tf_json="[]"; sw_json="[]"; sf_json="[]"
  fi
  rm -f "$tmp_el_py"

  # ── Assemble raw layout-map JSON ─────────────────────────────────────────────
  python3 -c "
import textwrap, sys
script = textwrap.dedent('''
    import sys, json
    logical_w = int(sys.argv[1])
    logical_h = int(sys.argv[2])
    btn_json    = json.loads(sys.argv[3])
    tabbar_json = json.loads(sys.argv[4])
    navbar_json = json.loads(sys.argv[5])
    tf_json     = json.loads(sys.argv[6])
    sw_json     = json.loads(sys.argv[7])
    sf_json     = json.loads(sys.argv[8])

    tab_bar_top = logical_h
    tab_bar_rect = None
    if tabbar_json:
        tb = tabbar_json[0]
        tab_bar_top = tb[\"y\"]
        tab_bar_rect = tb

    nav_bar_bottom = 0; nav_title = None; back_info = None
    if navbar_json:
        nb = navbar_json[0]
        nav_bar_bottom = nb[\"y\"] + nb[\"h\"]
        nav_title = nb[\"label\"] if nb[\"label\"] else None

    tabs = []; content_buttons = []
    for btn in btn_json:
        bx = btn[\"x\"]; by = btn[\"y\"]; bw = btn[\"w\"]; bh = btn[\"h\"]
        cy_center = by + bh // 2
        if tab_bar_rect and cy_center >= tab_bar_top:
            tabs.append({\"label\": btn[\"label\"], \"selected\": False,
                         \"x\": bx + bw // 2, \"y\": by + bh // 2})
        elif nav_bar_bottom > 0 and cy_center < nav_bar_bottom:
            lbl_lower = btn[\"label\"].lower()
            if \"back\" in lbl_lower or btn[\"label\"] == \"\":
                if back_info is None:
                    back_info = {\"label\": btn[\"label\"] or \"Back\",
                                 \"x\": bx + bw // 2, \"y\": by + bh // 2}
        else:
            if by < tab_bar_top:
                content_buttons.append(btn)

    if back_info is None and not navbar_json:
        for btn in btn_json:
            bx = btn[\"x\"]; by = btn[\"y\"]; bw = btn[\"w\"]; bh = btn[\"h\"]
            if \"back\" in btn[\"label\"].lower() and by < logical_h * 0.15:
                back_info = {\"label\": btn[\"label\"] or \"Back\",
                             \"x\": bx + bw // 2, \"y\": by + bh // 2}; break

    interactive = []; seen_keys = set()
    def add_interactive(els, role):
        for el in els:
            key = role + \"|\" + el[\"label\"] + f\"|{el[chr(120)]},{el[chr(121)]}\"
            if key in seen_keys: continue
            seen_keys.add(key)
            interactive.append({\"role\": role, \"label\": el[\"label\"],
                \"x\": el[\"x\"], \"y\": el[\"y\"], \"w\": el[\"w\"], \"h\": el[\"h\"],
                \"value\": None, \"enabled\": True})
    add_interactive(content_buttons, \"AXButton\")
    add_interactive(tf_json, \"AXTextField\")
    add_interactive(sw_json, \"AXSwitch\")
    add_interactive(sf_json, \"AXSearchField\")

    output = {
        \"screen\": {\"w\": logical_w, \"h\": logical_h},
        \"modal\": False,
        \"navigation\": {\"title\": nav_title, \"back\": back_info, \"tabs\": tabs},
        \"interactive\": interactive,
        \"content\": {\"headings\": [], \"texts\": []},
        \"scroll\": {\"vertical\": True, \"horizontal\": False,
                   \"above_fold\": [], \"below_fold\": []},
        \"_source\": \"elements_fallback\",
    }
    print(json.dumps(output))
''').lstrip()
sys.stdout.write(script)
" > "$tmp_asm"

  local asm_result
  asm_result=$(python3 "$tmp_asm" \
    "$lw" "$lh" \
    "$btn_json" "$tabbar_json" "$navbar_json" \
    "$tf_json" "$sw_json" "$sf_json") || asm_result=""
  rm -f "$tmp_asm"

  if [[ -z "$asm_result" ]]; then
    echo "ERROR: element-query fallback assembly failed" >&2
    printf '{"screen":{"w":%s,"h":%s},"navigation":{"title":null,"tabs":[],"back":null},"interactive":[],"content":{"headings":[],"texts":[]},"scroll":{"vertical":true,"horizontal":false,"above_fold":[],"below_fold":[]},"context":"unknown","suggested_actions":[],"_warning":"Both WDA /source and element fallback failed."}\n' \
      "$lw" "$lh"
    return 1
  fi

  # ── Enrich with context + suggested_actions ───────────────────────────────────
  python3 -c "
import textwrap, sys
script = textwrap.dedent('''
    import json, sys
    with open(sys.argv[1]) as f: data = json.load(f)
    interactive = data.get(\"interactive\", [])
    navigation  = data.get(\"navigation\", {})
    scroll      = data.get(\"scroll\", {})
    modal       = data.get(\"modal\", False)
    tabs        = navigation.get(\"tabs\", [])
    back        = navigation.get(\"back\")
    nav_title   = navigation.get(\"title\")

    def has_role(r): return any(el.get(\"role\",\"\") == r for el in interactive)
    def has_lbl(words):
        for el in interactive:
            l = el.get(\"label\",\"\").lower()
            if any(w.lower() in l for w in words): return True
        return False
    def has_close(): return any(k in el.get(\"label\",\"\").lower() for el in interactive for k in [\"close\",\"dismiss\",\"cancel\"])

    context = \"unknown\"
    has_back = back is not None; has_tabs = len(tabs) >= 1
    if has_lbl([\"play\",\"pause\"]) or (has_role(\"AXSlider\") and has_back and not has_tabs): context = \"media_player\"
    elif modal or (not nav_title and has_close()): context = \"modal_sheet\"
    elif has_role(\"AXSearchField\"): context = \"search\"
    elif has_role(\"AXTextField\") or has_role(\"AXSecureTextField\"): context = \"form\"
    elif len(tabs) >= 3: context = \"tab_screen\"
    elif scroll.get(\"vertical\") and len(interactive) > 3: context = \"detail_screen\" if (has_back and not has_tabs) else \"list\"
    elif has_back and not has_tabs: context = \"detail_screen\"

    suggested = []
    for tab in tabs:
        if tab.get(\"selected\"): continue
        suggested.append(\"tap tab \" + repr(tab.get(\"label\",\"?\")) + \" at (\" + str(tab.get(\"x\",0)) + \", \" + str(tab.get(\"y\",0)) + \")\")
    if back: suggested.append(\"tap back \" + repr(back.get(\"label\",\"Back\")) + \" at (\" + str(back.get(\"x\",0)) + \", \" + str(back.get(\"y\",0)) + \") to return\")
    bl = back.get(\"label\",\"\").lower() if back else None
    cnt = 0
    for el in interactive:
        if cnt >= 6: break
        lbl = el.get(\"label\",\"\")
        if not lbl or (bl and lbl.lower() == bl): continue
        cx = el.get(\"x\",0) + el.get(\"w\",0)//2; cy = el.get(\"y\",0) + el.get(\"h\",0)//2
        suggested.append(\"tap \" + repr(lbl) + \" at (\" + str(cx) + \", \" + str(cy) + \")\")
        cnt += 1
    suggested = suggested[:10]
    data[\"context\"] = context; data[\"suggested_actions\"] = suggested
    print(json.dumps(data, indent=2))
''').lstrip()
sys.stdout.write(script)
" > "$tmp_enrich"

  local py_in enriched py_exit
  py_in=$(mktemp /tmp/sim_layout_py_XXXXXX.json)
  printf '%s' "$asm_result" > "$py_in"
  enriched=$(python3 "$tmp_enrich" "$py_in")
  py_exit=$?
  rm -f "$py_in" "$tmp_enrich"

  if [[ $py_exit -ne 0 ]] || [[ -z "$enriched" ]]; then
    echo "$asm_result"
  else
    echo "$enriched"
  fi
}

cmd_layout_map() {
  # Optional per-call flags: --source-timeout <seconds>, --filter <buttons|all>
  local source_timeout="${LAYOUT_MAP_SOURCE_TIMEOUT:-8}"
  local layout_filter="all"
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --source-timeout) source_timeout="$2"; shift 2 ;;
      --filter)         layout_filter="$2";  shift 2 ;;
      *) break ;;
    esac
  done

  _bootstrap

  # ── Fetch WDA source XML ──────────────────────────────────────────────────────
  # Try /wda/accessibleSource first — pre-pruned accessibility-only tree, faster to serialize.
  # Fall back to full /source if that fails, returns empty, or returns non-XML (JSON dict).
  local wda_xml curl_rc
  wda_xml=$(curl -sf --max-time "${source_timeout}" \
    "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/wda/accessibleSource" 2>/dev/null)
  curl_rc=$?
  # accessibleSource may return a JSON dict tree instead of XML — detect and skip
  if [[ $curl_rc -ne 0 ]] || [[ -z "$wda_xml" ]] || ! echo "$wda_xml" | python3 -c "
import sys, json
raw = sys.stdin.read()
try:
    d = json.loads(raw)
    v = d.get('value', raw) if isinstance(d, dict) else raw
    sys.exit(0 if isinstance(v, str) and v.strip().startswith('<') else 1)
except: sys.exit(0 if raw.strip().startswith('<') else 1)
" 2>/dev/null; then
    # Fall back to full source
    wda_xml=$(curl -sf --max-time "${source_timeout}" \
      "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/source" 2>/dev/null); curl_rc=$?; true
  fi

  if [[ $curl_rc -eq 28 ]]; then
    # Timeout (exit 28): WDA is alive but the AX tree is too large to serialize.
    # Fall back to targeted /elements queries — much faster than /source on large trees.
    echo "WARNING: WDA /source timed out after ${source_timeout}s (AX tree too large). Using element-query fallback." >&2
    _layout_map_elements_fallback "$layout_filter"
    return $?
  fi

  if [[ $curl_rc -ne 0 ]] || [[ -z "$wda_xml" ]]; then
    # Non-timeout error: session may have expired — if WDA server is still up, create a new session and retry.
    local wda_status
    wda_status=$(curl -s -o /dev/null -w "%{http_code}" --max-time 3 \
      "http://localhost:${WDA_PORT}/status" 2>/dev/null) || true
    if [[ "$wda_status" == "200" ]]; then
      local session_resp session_id
      session_resp=$(curl -s -X POST \
        -H 'Content-Type: application/json' \
        -d '{"capabilities":{"alwaysMatch":{"platformName":"iOS"}}}' \
        --max-time 15 \
        "http://localhost:${WDA_PORT}/session" 2>/dev/null) || true
      session_id=$(echo "$session_resp" | python3 -c \
        "import sys,json; d=json.load(sys.stdin); print(d.get('sessionId') or d.get('value',{}).get('sessionId',''))" \
        2>/dev/null) || true
      if [[ -n "$session_id" ]]; then
        WDA_SESSION="$session_id"
        _wda_write_cache
        # Re-pin active application after session recreate to avoid background-window mismatch on iPad
        if [[ -n "${WDA_APP_BUNDLE_ID:-}" ]]; then
          curl -s -X POST \
            -H "Content-Type: application/json" \
            -d "{\"settings\":{\"defaultActiveApplication\":\"${WDA_APP_BUNDLE_ID}\"}}" \
            --max-time 5 \
            "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/appium/settings" \
            > /dev/null 2>&1 || true
        fi
        wda_xml=$(curl -sf --max-time "${source_timeout}" "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/source" 2>/dev/null) || true
      fi
    fi
    if [[ -z "$wda_xml" ]]; then
      echo '{"error":"WDA unavailable"}' >&2
      return 1
    fi
  fi

  # ── Parse XML with Python, produce intermediate JSON ─────────────────────────
  local tmp_xml tmp_py_parse result
  tmp_xml=$(mktemp -t sim_wda_xml)
  tmp_py_parse=$(mktemp -t sim_pyparse)
  printf '%s' "$wda_xml" > "$tmp_xml"
  cat > "$tmp_py_parse" << 'PYPARSE'
import sys, json, xml.etree.ElementTree as ET

logical_w = int(sys.argv[1]) if len(sys.argv) > 1 else 393
logical_h = int(sys.argv[2]) if len(sys.argv) > 2 else 852
layout_filter = sys.argv[4] if len(sys.argv) > 4 else "all"

raw = open(sys.argv[3]).read() if len(sys.argv) > 3 else sys.stdin.read()
# WDA /source returns JSON {"value": "<xml..."} — unwrap if needed
try:
    parsed = json.loads(raw)
    raw_xml = parsed.get("value", raw) if isinstance(parsed, dict) else raw
except Exception:
    raw_xml = raw
try:
    root = ET.fromstring(raw_xml)
except Exception as e:
    sys.stderr.write("layout-map: XML parse error: " + str(e) + "\n")
    sys.exit(1)

# ── Helpers ────────────────────────────────────────────────────────────────────

def iattr(el, key, default=0):
    try:
        return int(el.get(key, default))
    except (TypeError, ValueError):
        return default

def name(el):
    # Prefer label over name — name often has internal IDs like "TabBar.Settings.Item"
    lbl = el.get("label", "")
    nm  = el.get("name", "")
    return lbl if lbl else nm

def is_visible(el):
    return iattr(el, "width") > 0 and iattr(el, "height") > 0

def iter_all(node):
    yield node
    for child in node:
        yield from iter_all(child)

# ── Single pass over full tree — cache in list so we don't re-traverse for each type ──

all_nodes = list(iter_all(root))

# ── Screen dimensions ──────────────────────────────────────────────────────────

app_el = next(
    (e for e in all_nodes if e.tag == "XCUIElementTypeApplication"),
    root
)
screen_w = iattr(app_el, "width", logical_w)
screen_h = iattr(app_el, "height", logical_h)

# ── Modal detection ────────────────────────────────────────────────────────────

is_modal = any(
    e.tag in ("XCUIElementTypeSheet", "XCUIElementTypeAlert")
    for e in all_nodes
)

# ── Tab bar ────────────────────────────────────────────────────────────────────

tab_bar = next(
    (e for e in all_nodes if e.tag == "XCUIElementTypeTabBar"),
    None
)

tabs = []
tab_bar_top = screen_h
tab_bar_elems = set()
if tab_bar is not None and is_visible(tab_bar):
    tab_bar_top = iattr(tab_bar, "y")
    tab_bar_elems = set(id(e) for e in iter_all(tab_bar))
    for btn in tab_bar:
        if btn.tag == "XCUIElementTypeButton" and is_visible(btn):
            bx = iattr(btn, "x")
            by = iattr(btn, "y")
            bw = iattr(btn, "width")
            bh = iattr(btn, "height")
            tabs.append({
                "label": name(btn),
                "selected": btn.get("selected", "false").lower() == "true",
                "x": bx + bw // 2,
                "y": by + bh // 2,
            })

# ── Navigation bar ─────────────────────────────────────────────────────────────

nav_bar = next(
    (e for e in all_nodes if e.tag == "XCUIElementTypeNavigationBar"),
    None
)

nav_title = None
nav_bar_bottom = 0
back_info = None
nav_bar_elems = set()

if nav_bar is not None and is_visible(nav_bar):
    nav_bar_bottom = iattr(nav_bar, "y") + iattr(nav_bar, "height")
    nav_bar_elems = set(id(e) for e in iter_all(nav_bar))
    # Title: first StaticText child or nav bar name
    for child in nav_bar:
        n = name(child)
        if child.tag == "XCUIElementTypeStaticText" and n:
            nav_title = n
            break
    if nav_title is None:
        n = name(nav_bar)
        if n:
            nav_title = n
    # Back button: first Button in NavBar whose name contains "back" or is empty (chevron)
    for child in nav_bar:
        if child.tag == "XCUIElementTypeButton":
            n_lower = name(child).lower()
            if "back" in n_lower or n_lower == "":
                lbl = name(child) or "Back"
                bx = iattr(child, "x"); by = iattr(child, "y")
                bw = iattr(child, "width"); bh = iattr(child, "height")
                back_info = {"label": lbl, "x": bx + bw // 2, "y": by + bh // 2}
                break

# ── Interactive elements ───────────────────────────────────────────────────────

INTERACTIVE_TAGS = {
    "XCUIElementTypeButton",
    "XCUIElementTypeTextField",
    "XCUIElementTypeSecureTextField",
    "XCUIElementTypeSwitch",
    "XCUIElementTypeSlider",
    "XCUIElementTypeSearchField",
}
if layout_filter == "buttons":
    INTERACTIVE_TAGS = {"XCUIElementTypeButton"}

TAG_TO_ROLE = {
    "XCUIElementTypeButton":          "AXButton",
    "XCUIElementTypeTextField":       "AXTextField",
    "XCUIElementTypeSecureTextField": "AXSecureTextField",
    "XCUIElementTypeSwitch":          "AXSwitch",
    "XCUIElementTypeSlider":          "AXSlider",
    "XCUIElementTypeSearchField":     "AXSearchField",
}

interactive = []
seen_interactive = set()
for el in all_nodes:
    if el.tag not in INTERACTIVE_TAGS:
        continue
    if id(el) in tab_bar_elems or id(el) in nav_bar_elems:
        continue
    if not is_visible(el):
        continue
    n = name(el)
    key = el.tag + "|" + n
    if key in seen_interactive:
        continue
    seen_interactive.add(key)
    interactive.append({
        "role":    TAG_TO_ROLE.get(el.tag, el.tag),
        "label":   n,
        "x":       iattr(el, "x"),
        "y":       iattr(el, "y"),
        "w":       iattr(el, "width"),
        "h":       iattr(el, "height"),
        "value":   el.get("value"),
        "enabled": el.get("enabled", "true").lower() != "false",
    })

# ── Content: headings and texts ────────────────────────────────────────────────

content_top = nav_bar_bottom
content_h = tab_bar_top - content_top if tab_bar_top > content_top else screen_h - content_top
heading_threshold_y = content_top + content_h * 0.30

headings = []
texts = []
seen_text = set()

if layout_filter != "buttons":
    for el in all_nodes:
        if el.tag != "XCUIElementTypeStaticText":
            continue
        if not is_visible(el):
            continue
        n = name(el)
        if not n or n in seen_text:
            continue
        ey = iattr(el, "y")
        if ey < content_top or ey > tab_bar_top:
            continue
        seen_text.add(n)
        if ey <= heading_threshold_y and len(headings) < 5:
            headings.append(n)
        elif len(texts) < 10:
            texts.append(n)

# Remove texts that duplicate headings
heading_set = set(headings)
texts = [t for t in texts if t not in heading_set][:10]

# ── Scroll awareness ────────────────────────────────────────────────────────────

scroll_views = [e for e in all_nodes if e.tag == "XCUIElementTypeScrollView" and is_visible(e)]
has_vertical_scroll = len(scroll_views) > 0
has_horizontal_scroll = any(
    iattr(sv, "width") > iattr(sv, "height") for sv in scroll_views
)

visible_bottom = tab_bar_top - 50
above_fold = []
below_fold = []
seen_fold = set()

FOLD_TAGS = {
    "XCUIElementTypeStaticText",
    "XCUIElementTypeButton",
    "XCUIElementTypeCell",
}

for el in all_nodes:
    if el.tag not in FOLD_TAGS:
        continue
    if not is_visible(el):
        continue
    n = name(el)
    if not n or n in seen_fold:
        continue
    if id(el) in tab_bar_elems or id(el) in nav_bar_elems:
        continue
    ey = iattr(el, "y")
    eh = iattr(el, "height")
    if ey + eh > visible_bottom and len(below_fold) < 8:
        seen_fold.add(n)
        below_fold.append(n)
    elif ey + eh < content_top and len(above_fold) < 8:
        seen_fold.add(n)
        above_fold.append(n)

# ── Assemble JSON ──────────────────────────────────────────────────────────────

output = {
    "screen":     {"w": screen_w, "h": screen_h},
    "modal":      is_modal,
    "navigation": {
        "title": nav_title,
        "back":  back_info,
        "tabs":  tabs,
    },
    "interactive": interactive,
    "content": {
        "headings": headings,
        "texts":    texts,
    },
    "scroll": {
        "vertical":    has_vertical_scroll,
        "horizontal":  has_horizontal_scroll,
        "above_fold":  above_fold,
        "below_fold":  below_fold,
    },
}

print(json.dumps(output))
PYPARSE
  result=$(python3 "$tmp_py_parse" "$SIM_LOGICAL_W" "$SIM_LOGICAL_H" "$tmp_xml" "$layout_filter")
  local py_parse_exit=$?
  rm -f "$tmp_xml" "$tmp_py_parse"
  if [[ $py_parse_exit -ne 0 ]] || [[ -z "$result" ]]; then
    echo '{"error":"WDA XML parse failed"}' >&2
    return 1
  fi

  # Write parsed JSON to a temp file for the enrichment script
  local py_in
  py_in=$(mktemp /tmp/sim_layout_py_XXXXXX.json)
  printf '%s' "$result" > "$py_in"

  # ── Enrichment block follows ──────────────────────────────────────────────────
  local tmp_py_enrich enriched
  tmp_py_enrich=$(mktemp -t sim_pyenrich)
  cat > "$tmp_py_enrich" << 'PYEOF'
import json, sys

input_path = sys.argv[1]
with open(input_path) as f:
    raw = f.read()

try:
    data = json.loads(raw)
except Exception:
    print(raw, end="")
    sys.exit(0)

interactive = data.get("interactive", [])
navigation  = data.get("navigation", {})
scroll      = data.get("scroll", {})
modal       = data.get("modal", False)
tabs        = navigation.get("tabs", [])
back        = navigation.get("back")
nav_title   = navigation.get("title")

# ── Helpers ───────────────────────────────────────────────────────────────────

def has_role(role):
    return any(el.get("role", "") == role for el in interactive)

def has_label_containing(words):
    for el in interactive:
        lbl = el.get("label", "").lower()
        for w in words:
            if w.lower() in lbl:
                return True
    return False

def has_close_button():
    for el in interactive:
        lbl = el.get("label", "").lower()
        if "close" in lbl or "dismiss" in lbl or "cancel" in lbl:
            return True
    return False

# ── Context (first match wins) ────────────────────────────────────────────────
context = "unknown"

has_play_pause = has_label_containing(["play", "pause", "btnplay", "btnpause"])
has_slider     = has_role("AXSlider")
has_back       = back is not None
has_tabs       = len(tabs) >= 1

if has_play_pause or (has_slider and has_back and not has_tabs):
    context = "media_player"
elif modal or (not nav_title and has_close_button()):
    context = "modal_sheet"
elif has_role("AXSearchField"):
    context = "search"
elif has_role("AXTextField") or has_role("AXSecureTextField"):
    context = "form"
elif len(tabs) >= 3:
    context = "tab_screen"
elif scroll.get("vertical") and len(interactive) > 3:
    if has_back and not has_tabs:
        context = "detail_screen"
    else:
        context = "list"
elif has_back and not has_tabs and scroll.get("vertical"):
    context = "detail_screen"

# ── Suggested actions ─────────────────────────────────────────────────────────
suggested = []

# 1. Scroll down
below_fold = scroll.get("below_fold", [])
if below_fold:
    labels = ", ".join(below_fold[:4])
    suggested.append("scroll down to reveal: " + labels)

# 2. Scroll up
above_fold = scroll.get("above_fold", [])
if above_fold:
    labels = ", ".join(above_fold[:4])
    suggested.append("scroll up to reveal: " + labels)

# 3. Tabs — x/y from Swift are already center coords
for tab in tabs:
    if tab.get("selected"):
        continue
    lbl = tab.get("label", "")
    cx  = tab.get("x", 0)
    cy  = tab.get("y", 0)
    suggested.append("tap tab '" + lbl + "' at (" + str(cx) + ", " + str(cy) + ")")

# 4. Back button — x/y from Swift are already center coords
if back:
    lbl = back.get("label", "Back")
    cx  = back.get("x", 0)
    cy  = back.get("y", 0)
    suggested.append("tap back '" + lbl + "' at (" + str(cx) + ", " + str(cy) + ") to return")

# 5. Interactive elements (up to 6, skip empty label, skip back)
back_label_lower = back.get("label", "").lower() if back else None
count = 0
for el in interactive:
    if count >= 6:
        break
    lbl = el.get("label", "")
    if not lbl:
        continue
    if back_label_lower and lbl.lower() == back_label_lower:
        continue
    cx = el.get("x", 0) + el.get("w", 0) // 2
    cy = el.get("y", 0) + el.get("h", 0) // 2
    suggested.append("tap '" + lbl + "' at (" + str(cx) + ", " + str(cy) + ")")
    count += 1

# Cap at 10
suggested = suggested[:10]

data["context"]           = context
data["suggested_actions"] = suggested

print(json.dumps(data, indent=2))
PYEOF
  enriched=$(python3 "$tmp_py_enrich" "$py_in")
  local py_exit=$?
  rm -f "$py_in" "$tmp_py_enrich"

  if [[ $py_exit -ne 0 ]] || [[ -z "$enriched" ]]; then
    # Python failed — fall back to original Swift output unchanged
    echo "$result"
    return 0
  fi

  # ── Spot-check WDA's reported coordinates against the live AX hit-test ──────
  # XCUITest's snapshot() (which feeds WDA /source) returns layout-intended
  # frames. SwiftUI flex layouts (Spacer, .frame(maxHeight:.infinity), etc.)
  # collapse at render time, leaving WDA's reported y for elements after the
  # spacer way below their actual on-screen position. macOS AX hit-test
  # (describe-point) returns the rendered position, which is what taps
  # actually need.
  #
  # Strategy: ALWAYS hit-test every interactive element via the batched
  # describe-point helper (one swift invocation, ~300ms even for 50 points).
  # If WDA's reported center for an element doesn't match the AX hit-test
  # there, replace its rect with the AX-correct one and mark the response
  # `"layout_corrected": true` so consumers know we patched it.
  #
  # When the initial probe at WDA's cy misses, we look up to ±150pt around it
  # in the SAME batched call (extra coords appended up front so the swift
  # invocation count stays at one). This covers both the SwiftUI Spacer-
  # collapse case (button rendered HIGHER than reported) and rarer cases
  # where elements render lower than reported.
  local interactive_count
  interactive_count=$(echo "$enriched" | python3 -c "
import json, sys
try: d = json.loads(sys.stdin.read())
except Exception: sys.exit(0)
print(len(d.get('interactive', [])))
" 2>/dev/null)

  if [[ -z "$interactive_count" || "$interactive_count" -eq 0 ]]; then
    echo "$enriched"
    return 0
  fi

  # ── Phase 1: hit-test each element's WDA-reported center (one batched call)
  # Cost: N AX hit-tests + 1 swift cold-start ≈ 200-400ms regardless of N.
  # Output is one JSON-or-"null" per input line, in order.
  local phase1_input phase1_results
  phase1_input=$(echo "$enriched" | python3 -c "
import json, sys
d = json.loads(sys.stdin.read())
for el in d.get('interactive', []):
    cx = int(el.get('x', 0) + el.get('w', 0) / 2)
    cy = int(el.get('y', 0) + el.get('h', 0) / 2)
    print(cx, cy)
" 2>/dev/null)

  phase1_results=$(printf '%s\n' "$phase1_input" | _describe_points_batch)

  # Identify which element indices failed phase 1. Build the phase-2 input
  # plan: for each failed element, emit 20 walk probes (±30..±300 in 30pt
  # steps). Skip phase 2 entirely when nothing failed.
  local need_phase2 phase2_plan
  need_phase2=$(echo "$enriched" | NEED_RES="$phase1_results" python3 -c "
import json, os, sys
data = json.loads(sys.stdin.read())
results = []
for line in os.environ.get('NEED_RES', '').split('\n'):
    line = line.strip()
    if not line or line == 'null':
        results.append(None)
    else:
        try: results.append(json.loads(line))
        except Exception: results.append(None)

# Failed = WDA hit-test returned a different label than expected.
failed = []
for idx, el in enumerate(data.get('interactive', [])):
    if idx >= len(results):
        failed.append(idx); continue
    expected = (el.get('label') or '').lower()
    if not expected:
        continue
    hit = results[idx]
    if not hit:
        failed.append(idx); continue
    if (hit.get('label') or '').lower() != expected:
        failed.append(idx)

print(','.join(str(i) for i in failed))
" 2>/dev/null)

  if [[ -z "$need_phase2" ]]; then
    # All elements verified clean — return raw enriched (no correction needed).
    echo "$enriched"
    return 0
  fi

  # ── Phase 2: walk ±300pt around each failed element's WDA y in 30pt steps
  # ±300pt covers SwiftUI Spacer heights up to a full screen third, enough for
  # every real-world flex layout we've seen (Mindvalley's worst case = 226pt).
  # 20 probes × failed_element_count, one batched call.
  local phase2_input phase2_results
  phase2_input=$(echo "$enriched" | FAILED="$need_phase2" python3 -c "
import json, os, sys
data = json.loads(sys.stdin.read())
failed = set(int(x) for x in os.environ.get('FAILED', '').split(',') if x)
for idx, el in enumerate(data.get('interactive', [])):
    if idx not in failed:
        continue
    cx = int(el.get('x', 0) + el.get('w', 0) / 2)
    cy = int(el.get('y', 0) + el.get('h', 0) / 2)
    for off in (-30, -60, -90, -120, -150, -180, -210, -240, -270, -300,
                 30,  60,  90, 120, 150, 180, 210, 240, 270, 300):
        sy = cy + off
        if sy < 50:
            print('')
        else:
            print(cx, sy)
" 2>/dev/null)

  phase2_results=$(printf '%s\n' "$phase2_input" | _describe_points_batch)

  # Apply corrections: for each failed element, take the first phase-2 probe
  # whose label matches and use its AX-reported rect. Mark layout_corrected.
  local corrected
  corrected=$(echo "$enriched" | FAILED="$need_phase2" P2RES="$phase2_results" python3 -c "
import json, os, sys
data = json.loads(sys.stdin.read())
failed = [int(x) for x in os.environ.get('FAILED', '').split(',') if x]
results = []
for line in os.environ.get('P2RES', '').split('\n'):
    line = line.strip()
    if not line or line == 'null':
        results.append(None)
    else:
        try: results.append(json.loads(line))
        except Exception: results.append(None)

WALKS_PER_ELEMENT = 20
n_fixed = 0
interactive = data.get('interactive', [])
for slot, idx in enumerate(failed):
    if idx >= len(interactive):
        continue
    el = interactive[idx]
    expected = (el.get('label') or '').lower()
    if not expected:
        continue
    base = slot * WALKS_PER_ELEMENT
    block = results[base:base + WALKS_PER_ELEMENT]
    for probe in block:
        if probe and (probe.get('label') or '').lower() == expected:
            for k in ('x', 'y', 'w', 'h'):
                el[k] = probe.get(k, el.get(k, 0))
            n_fixed += 1
            break

if n_fixed > 0:
    data['layout_corrected'] = True
    data['_layout_correction_count'] = n_fixed
print(json.dumps(data, indent=2))
" 2>/dev/null)

  if [[ -n "$corrected" ]]; then
    echo "$corrected"
  else
    echo "$enriched"
  fi
}
