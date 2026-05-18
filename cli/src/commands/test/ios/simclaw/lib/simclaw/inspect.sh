# lib/simclaw/inspect.sh — AX tree inspection commands
[[ -n "${_SIM_INSPECT_LOADED:-}" ]] && return 0; _SIM_INSPECT_LOADED=1

source "$SIM_LIB/bootstrap.sh"
source "$SIM_LIB/coords.sh"

cmd_describe() {
  # max_depth arg kept for API compatibility but ignored — Swift AX traversal
  # uses its own depth limit (25 levels) which is sufficient for all known screens.
  local interactive_only=false
  local max_depth="3"
  for arg in "$@"; do
    case "$arg" in
      --interactive|-i) interactive_only=true ;;
      [0-9]*)           max_depth="$arg" ;;
    esac
  done
  _bootstrap

  # Use Swift + AXUIElement API for fast, reliable UI element enumeration.
  # This avoids the AppleScript 'entire contents' approach which is O(n) IPC-slow
  # and degrades severely on deep trees (>100 elements takes minutes in AppleScript).
  local scrX="$SIM_SCREEN_X"
  local scrY="$SIM_SCREEN_Y"
  local zoom="$SIM_ZOOM"

  swift - "$scrX" "$scrY" "$zoom" "$interactive_only" "${SIM_NAME} – ${SIM_OS}" 2>/dev/null << SWIFTEOF
import ApplicationServices
import AppKit
import Foundation

let args = CommandLine.arguments
let screenX       = Double(args.count > 1 ? args[1] : "-439") ?? -439.0
let screenY       = Double(args.count > 2 ? args[2] : "148") ?? 148.0
let zoom          = Double(args.count > 3 ? args[3] : "1.0") ?? 1.0
let interactiveOnly = (args.count > 4 && args[4] == "true")
let deviceName    = args.count > 5 ? args[5] : ""
let interactiveRoles: Set<String> = [
    "AXButton", "AXTextField", "AXSecureTextField", "AXLink",
    "AXSwitch", "AXCheckBox", "AXRadioButton", "AXSlider", "AXMenuItem"
]

// Find Simulator PID
guard let simApp = NSWorkspace.shared.runningApplications
    .first(where: { \$0.bundleIdentifier == "com.apple.iphonesimulator" }) else {
  print("[]"); exit(0)
}
let simPID = simApp.processIdentifier

let axApp = AXUIElementCreateApplication(simPID)
var windowsRef: CFTypeRef?
AXUIElementCopyAttributeValue(axApp, kAXWindowsAttribute as CFString, &windowsRef)
guard let windows = windowsRef as? [AXUIElement], !windows.isEmpty else {
  print("[]"); exit(0)
}

$(cat "$SIM_LIB/swift/pickwindow.swift")

guard let win = pickWindow(windows, screenX: screenX, screenY: screenY, deviceName: deviceName) else {
  print("[]"); exit(0)
}

func getAttr(_ el: AXUIElement, _ attr: String) -> AnyObject? {
    var val: CFTypeRef?
    guard AXUIElementCopyAttributeValue(el, attr as CFString, &val) == .success else { return nil }
    return val as AnyObject
}

func getPosition(_ el: AXUIElement) -> CGPoint? {
    guard let val = getAttr(el, kAXPositionAttribute) else { return nil }
    var pt = CGPoint.zero
    guard AXValueGetValue(val as! AXValue, .cgPoint, &pt) else { return nil }
    return pt
}

func getSize(_ el: AXUIElement) -> CGSize? {
    guard let val = getAttr(el, kAXSizeAttribute) else { return nil }
    var sz = CGSize.zero
    guard AXValueGetValue(val as! AXValue, .cgSize, &sz) else { return nil }
    return sz
}

func getLabel(_ el: AXUIElement) -> String {
    if let d = getAttr(el, kAXDescriptionAttribute) as? String, !d.isEmpty { return d }
    if let t = getAttr(el, kAXTitleAttribute) as? String, !t.isEmpty { return t }
    if let v = getAttr(el, kAXValueAttribute) as? String, !v.isEmpty { return v }
    return ""
}

func getRole(_ el: AXUIElement) -> String {
    return (getAttr(el, kAXRoleAttribute) as? String) ?? ""
}

var results: [String] = []

func enumerate(_ el: AXUIElement, depth: Int = 0) {
    guard depth < 25 else { return }

    let pos = getPosition(el)
    let sz  = getSize(el)
    let label = getLabel(el)
    let role  = getRole(el)

    if let p = pos, let s = sz, s.width >= 5, s.height >= 5 {
        let lx = Int(round((p.x - screenX) / zoom))
        let ly = Int(round((p.y - screenY) / zoom))
        let lw = Int(round(s.width / zoom))
        let lh = Int(round(s.height / zoom))
        if !role.isEmpty || !label.isEmpty {
            if !interactiveOnly || interactiveRoles.contains(role) {
                let safeLabel = label
                    .replacingOccurrences(of: "\\\\", with: "\\\\\\\\")
                    .replacingOccurrences(of: "\"", with: "\\\\\\"")
                results.append(
                    "{\"role\":\"\(role)\",\"label\":\"\(safeLabel)\",\"x\":\(lx),\"y\":\(ly),\"w\":\(lw),\"h\":\(lh)}"
                )
            }
        }
    }

    // Fetch children — for Tab Bar groups, also probe AXTabs (UITabBarButton items
    // are not exposed via kAXChildrenAttribute on the iOS Simulator AX bridge)
    var childrenRef: CFTypeRef?
    AXUIElementCopyAttributeValue(el, kAXChildrenAttribute as CFString, &childrenRef)
    var children = (childrenRef as? [AXUIElement]) ?? []

    if children.isEmpty && role == "AXGroup" && label == "Tab Bar" {
        var tabsRef: CFTypeRef?
        AXUIElementCopyAttributeValue(el, "AXTabs" as CFString, &tabsRef)
        if let tabs = tabsRef as? [AXUIElement] {
            children = tabs
        }
    }

    for child in children {
        enumerate(child, depth: depth + 1)
    }
}

enumerate(win)
print("[\(results.joined(separator: ","))]")
SWIFTEOF
}

cmd_screen_title() {
  _bootstrap
  swift - "${SIM_SCREEN_X}" "${SIM_SCREEN_Y}" "${SIM_SCREEN_W}" "${SIM_ZOOM}" "${SIM_NAME} – ${SIM_OS}" << SWIFTEOF 2>/dev/null
import Cocoa
import ApplicationServices

let args = CommandLine.arguments
let screenX  = Double(args.count > 1 ? args[1] : "0") ?? 0.0
let screenY  = Double(args.count > 2 ? args[2] : "0") ?? 0.0
let screenW  = Double(args.count > 3 ? args[3] : "393") ?? 393.0
let zoom     = Double(args.count > 4 ? args[4] : "1.0") ?? 1.0
let deviceName = args.count > 5 ? args[5] : ""

func getAttribute(_ el: AXUIElement, _ attr: String) -> CFTypeRef? {
    var v: CFTypeRef?
    AXUIElementCopyAttributeValue(el, attr as CFString, &v)
    return v
}

func getRole(_ el: AXUIElement) -> String {
    return (getAttribute(el, "AXRole") as? String) ?? ""
}

func getRoleDesc(_ el: AXUIElement) -> String {
    return (getAttribute(el, "AXRoleDescription") as? String) ?? ""
}

func getTitle(_ el: AXUIElement) -> String {
    for attr in ["AXTitle", "AXLabel", "AXDescription", "AXValue"] {
        if let v = getAttribute(el, attr) as? String, !v.isEmpty { return v }
    }
    return ""
}

func getPosition(_ el: AXUIElement) -> CGPoint? {
    guard let posVal = getAttribute(el, "AXPosition") else { return nil }
    var pt = CGPoint.zero
    guard AXValueGetValue(posVal as! AXValue, .cgPoint, &pt) else { return nil }
    return pt
}

func getSize(_ el: AXUIElement) -> CGSize? {
    guard let sizeVal = getAttribute(el, "AXSize") else { return nil }
    var sz = CGSize.zero
    guard AXValueGetValue(sizeVal as! AXValue, .cgSize, &sz) else { return nil }
    return sz
}

guard let app = NSRunningApplication.runningApplications(
    withBundleIdentifier: "com.apple.iphonesimulator").first else {
    print("unknown"); exit(0)
}
let axApp = AXUIElementCreateApplication(app.processIdentifier)
guard let windows = getAttribute(axApp, "AXWindows") as? [AXUIElement],
      !windows.isEmpty else {
    print("unknown"); exit(0)
}

$(cat "$SIM_LIB/swift/pickwindow.swift")

guard let window = pickWindow(windows, screenX: screenX, screenY: screenY, deviceName: deviceName) else {
    print("unknown"); exit(0)
}

// Depth-first search for a navigation bar element.
// Matches both UIKit (AXNavigationBar) and SwiftUI (AXGroup with AXRoleDescription="Nav bar").
// Returns the nav bar element if found.
func findNavBar(_ el: AXUIElement, _ depth: Int) -> AXUIElement? {
    if depth < 0 { return nil }
    let role = getRole(el)
    if role == "AXNavigationBar" { return el }
    if role == "AXGroup" && getRoleDesc(el) == "Nav bar" { return el }
    guard let children = getAttribute(el, "AXChildren") as? [AXUIElement] else { return nil }
    for child in children {
        if let found = findNavBar(child, depth - 1) { return found }
    }
    return nil
}

// Search up to 25 levels deep for a navigation bar
if let navBar = findNavBar(window, 25) {
    // Strategy 1: child AXStaticText or AXHeading of the nav bar via kAXChildrenAttribute
    if let children = getAttribute(navBar, "AXChildren") as? [AXUIElement] {
        for child in children {
            let r = getRole(child)
            if r == "AXStaticText" || r == "AXHeading" {
                let t = getTitle(child)
                if !t.isEmpty { print(t); exit(0) }
            }
        }
    }

    // Strategy 2: probe the center of the nav bar using AXUIElementCopyElementAtPosition.
    // iOS Simulator's AX bridge sometimes omits nav title children from kAXChildrenAttribute
    // but the element IS hit-testable at its screen position.
    if let pos = getPosition(navBar), let sz = getSize(navBar) {
        // Sample a few points across the center of the nav bar to find the title
        let centerY = Float(pos.y + sz.height / 2)
        let sampleXs: [Float] = [
            Float(pos.x + sz.width * 0.5),   // center
            Float(pos.x + sz.width * 0.35),  // slight left of center
            Float(pos.x + sz.width * 0.65),  // slight right of center
        ]
        for sampleX in sampleXs {
            var hitEl: AXUIElement?
            let err = AXUIElementCopyElementAtPosition(axApp, sampleX, centerY, &hitEl)
            if err == .success, let el = hitEl {
                let r = getRole(el)
                // Accept AXStaticText or AXHeading; skip the nav bar group itself
                if r == "AXStaticText" || r == "AXHeading" {
                    let t = getTitle(el)
                    if !t.isEmpty { print(t); exit(0) }
                }
            }
        }
    }

    // Strategy 3: nav bar's own AX label (rare but possible)
    let title = getTitle(navBar)
    if !title.isEmpty { print(title); exit(0) }
}

// No nav bar found — probe the top ~100pt band of the screen with hit-testing.
// This catches screens with inline/large title navigation or custom headers.
let navBandTop    = Float(screenY + 40 * zoom)
let navBandBottom = Float(screenY + 110 * zoom)
let centerX       = Float(screenX + screenW / 2)
let probeYs: [Float] = [
    Float((Double(navBandTop) + Double(navBandBottom)) / 2),
    navBandTop + Float(10 * zoom),
    navBandBottom - Float(10 * zoom),
]
for probeY in probeYs {
    var hitEl: AXUIElement?
    let err = AXUIElementCopyElementAtPosition(axApp, centerX, probeY, &hitEl)
    if err == .success, let el = hitEl {
        let r = getRole(el)
        if r == "AXStaticText" || r == "AXHeading" {
            let t = getTitle(el)
            if !t.isEmpty && t.count > 1 { print(t); exit(0) }
        }
    }
}

// Last resort: AXHeading in the top-300pt band via tree walk
func findTopHeading(_ el: AXUIElement, _ depth: Int) -> String? {
    if depth < 0 { return nil }
    if getRole(el) == "AXHeading" {
        if let pos = getPosition(el) {
            if pos.y < CGFloat(screenY + 300 * zoom) {
                let t = getTitle(el)
                if !t.isEmpty { return t }
            }
        }
    }
    guard let children = getAttribute(el, "AXChildren") as? [AXUIElement] else { return nil }
    for child in children {
        if let found = findTopHeading(child, depth - 1) { return found }
    }
    return nil
}

if let title = findTopHeading(window, 25) {
    print(title)
    exit(0)
}

// Strategy 5: Broadened scan — topmost prominent AXStaticText in top 200pt
// Returns with ~ prefix to signal heuristic match
func findProminentText(_ el: AXUIElement, _ depth: Int) -> (String, CGFloat, CGFloat)? {
    if depth < 0 { return nil }
    let role = getRole(el)
    if role == "AXStaticText" || role == "AXHeading" {
        if let pos = getPosition(el), let sz = getSize(el) {
            if pos.y >= CGFloat(screenY + 40 * zoom) &&
               pos.y < CGFloat(screenY + 300 * zoom) && sz.height >= 14 {
                let t = getTitle(el)
                if !t.isEmpty && t.count >= 3 { return (t, pos.y, sz.height) }
            }
        }
    }
    guard let children = getAttribute(el, "AXChildren") as? [AXUIElement] else { return nil }
    var best: (String, CGFloat, CGFloat)? = nil
    for child in children {
        if let found = findProminentText(child, depth - 1) {
            if best == nil || found.1 < best!.1 || (found.1 == best!.1 && found.2 > best!.2) {
                best = found
            }
        }
    }
    return best
}

if let (text, _, _) = findProminentText(window, 25) {
    print("~\(text)")
    exit(0)
}

print("unknown")
SWIFTEOF
}

cmd_find_element() {
  [[ $# -ge 1 ]] || die "Usage: sim find-element <label>"
  local search_label="$1"
  _bootstrap

  local resp element_id

  # 1. Try exact accessibility id match
  resp=$(curl -s -X POST "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/elements" \
    -H 'Content-Type: application/json' \
    -d "{\"using\": \"accessibility id\", \"value\": \"${search_label}\"}" 2>/dev/null)

  element_id=$(echo "$resp" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    els = data.get('value', [])
    if els:
        print(els[0].get('ELEMENT', ''))
except: pass
" 2>/dev/null)

  # 2. If not found, try case-insensitive xpath partial match on @name or @label
  if [[ -z "$element_id" ]]; then
    local lower_label
    lower_label=$(echo "$search_label" | tr '[:upper:]' '[:lower:]')
    resp=$(curl -s -X POST "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/elements" \
      -H 'Content-Type: application/json' \
      -d "{\"using\": \"xpath\", \"value\": \"//*[contains(translate(@name,'ABCDEFGHIJKLMNOPQRSTUVWXYZ','abcdefghijklmnopqrstuvwxyz'),'${lower_label}') or contains(translate(@label,'ABCDEFGHIJKLMNOPQRSTUVWXYZ','abcdefghijklmnopqrstuvwxyz'),'${lower_label}')]\"}" 2>/dev/null)
    element_id=$(echo "$resp" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    els = data.get('value', [])
    if els:
        print(els[0].get('ELEMENT', ''))
except: pass
" 2>/dev/null)
  fi

  if [[ -n "$element_id" ]]; then
    # Get the element's bounding rect (WDA returns iOS logical coordinates directly)
    local rect_resp
    rect_resp=$(curl -s "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/element/${element_id}/rect" 2>/dev/null)

    echo "$rect_resp" | _FIND_LABEL="$search_label" python3 -c "
import sys, json, os
data = json.load(sys.stdin)
r = data.get('value', {})
x = int(r.get('x', 0))
y = int(r.get('y', 0))
w = int(r.get('width', 0))
h = int(r.get('height', 0))
label = os.environ.get('_FIND_LABEL', '')
print(json.dumps({'role': 'XCUIElement', 'label': label, 'x': x, 'y': y, 'w': w, 'h': h}))
" 2>/dev/null || die "find-element: failed to get rect for '$search_label'"
    return 0
  fi

  # ── AX-grid fallback ────────────────────────────────────────────────────────
  # WDA /source didn't include this label. Same gap that hits cmd_tap_element
  # on system dialogs, swipe-revealed actions, custom toggles like Settings >
  # Maps > Air Quality Index, etc. Run the same coarse-grid AX hit-test so
  # find-element returns a usable rect for those elements too. Picks the best
  # label match (exact > prefix > contains) over interactive + static-text
  # roles. Static text is included here because find-element is also used as
  # a "is this label visible?" probe by callers like scroll-to-visible.
  local lw="${SIM_LOGICAL_W:-402}"
  local lh="${SIM_LOGICAL_H:-874}"
  local grid_input=""
  local gx gy
  # 40pt step starting at y=20 / x=20. The 20pt offset puts probes in the
  # MIDDLE of typical 44pt-tall iOS HIG control bands rather than on their
  # edges; AX hit-test at the exact rect edge sometimes returns the parent
  # AXGroup instead of the control (we observed this on the Settings root
  # search field at y=802-840, which AX returns as AXGroup at y=800/840 but
  # AXTextField at y=820).
  for ((gy = 20; gy < lh; gy += 40)); do
    for ((gx = 20; gx < lw; gx += 40)); do
      grid_input+="$gx $gy"$'\n'
    done
  done

  local grid_results
  grid_results=$(printf '%s' "$grid_input" | _describe_points_batch)

  local needle_lower
  needle_lower=$(echo "$search_label" | tr '[:upper:]' '[:lower:]')
  local ax_match
  ax_match=$(echo "$grid_results" | NEEDLE="$needle_lower" python3 -c "
import json, os, sys
needle = os.environ.get('NEEDLE', '')
ROLES = {'AXButton', 'AXTextField', 'AXSecureTextField', 'AXSearchField',
         'AXSwitch', 'AXSlider', 'AXLink', 'AXMenuItem', 'AXCheckBox',
         'AXStaticText'}
best = None
best_score = -1
seen = set()
for line in sys.stdin:
    line = line.strip()
    if not line or line == 'null': continue
    try: hit = json.loads(line)
    except Exception: continue
    if hit.get('role') not in ROLES: continue
    lbl = (hit.get('label') or '').lower()
    if not lbl: continue
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
    print(json.dumps({
        'role': best.get('role'),
        'label': best.get('label'),
        'x': best.get('x', 0),
        'y': best.get('y', 0),
        'w': best.get('w', 0),
        'h': best.get('h', 0),
    }))
" 2>/dev/null)

  if [[ -n "$ax_match" ]]; then
    echo "$ax_match"
    return 0
  fi

  die "find-element: '$search_label' not found in WDA /source or in AX grid scan"
}

cmd_describe_point() {
  [[ $# -ge 2 ]] || die "Usage: sim describe-point <x> <y>"
  _is_integer "$1" || die "describe-point: x must be an integer, got: $1"
  _is_integer "$2" || die "describe-point: y must be an integer, got: $2"
  _bootstrap
  local screen_coords sx sy
  screen_coords=$(_ios_to_screen "$1" "$2")
  sx=$(echo "$screen_coords" | awk '{print $1}')
  sy=$(echo "$screen_coords" | awk '{print $2}')

  local sim_orientation
  sim_orientation=$(plutil -extract "DevicePreferences.${SIM_UDID}.SimulatorWindowOrientation" raw \
    ~/Library/Preferences/com.apple.iphonesimulator.plist 2>/dev/null || echo "Portrait")

  local result
  result=$(swift - "$sx" "$sy" "$SIM_SCREEN_X" "$SIM_SCREEN_Y" "$SIM_ZOOM" "${SIM_NAME} – ${SIM_OS}" "$sim_orientation" "$SIM_LOGICAL_W" "$SIM_LOGICAL_H" 2>/dev/null << 'SWIFTEOF'
import ApplicationServices
import AppKit
import Foundation

let args = CommandLine.arguments
let screenX    = Double(args[1])!
let screenY    = Double(args[2])!
let originX    = Double(args[3])!
let originY    = Double(args[4])!
let zoom       = Double(args[5])!
let deviceName = args.count > 6 ? args[6] : ""
let orientation = args.count > 7 ? args[7] : "Portrait"
let logicalW    = Double(args.count > 8 ? args[8] : "390")  ?? 390.0
let logicalH    = Double(args.count > 9 ? args[9] : "844") ?? 844.0

guard let simApp = NSWorkspace.shared.runningApplications
    .first(where: { $0.bundleIdentifier == "com.apple.iphonesimulator" }) else {
    fputs("ERROR: Simulator not running\n", stderr); exit(1)
}
let pid = simApp.processIdentifier
let axApp = AXUIElementCreateApplication(pid)

func getAttr(_ el: AXUIElement, _ attr: String) -> AnyObject? {
    var v: CFTypeRef?
    guard AXUIElementCopyAttributeValue(el, attr as CFString, &v) == .success else { return nil }
    return v as AnyObject
}
func getLabel(_ el: AXUIElement) -> String {
    for a in ["AXDescription", "AXTitle", "AXLabel", "AXValue"] {
        if let s = getAttr(el, a) as? String, !s.isEmpty { return s }
    }
    return ""
}
func getRole(_ el: AXUIElement) -> String {
    return (getAttr(el, kAXRoleAttribute) as? String) ?? ""
}

var element: AXUIElement?
let err = AXUIElementCopyElementAtPosition(axApp, Float(screenX), Float(screenY), &element)
guard err == .success, let el = element else {
    fputs("ERROR: no element at point (\(Int(screenX)), \(Int(screenY)))\n", stderr)
    exit(1)
}

let role  = getRole(el)
let label = getLabel(el)

var lx = 0, ly = 0, lw = 0, lh = 0
if let posVal = getAttr(el, kAXPositionAttribute),
   let sizeVal = getAttr(el, kAXSizeAttribute) {
    var pt = CGPoint.zero; var sz = CGSize.zero
    AXValueGetValue(posVal as! AXValue, .cgPoint, &pt)
    AXValueGetValue(sizeVal as! AXValue, .cgSize, &sz)
    let wrx = (Double(pt.x) - originX) / zoom
    let wry = (Double(pt.y) - originY) / zoom
    let wrw = Double(sz.width)  / zoom
    let wrh = Double(sz.height) / zoom
    switch orientation {
    case "PortraitUpsideDown":
        lx = Int(logicalW - wrx - wrw + 0.5); ly = Int(logicalH - wry - wrh + 0.5)
        lw = Int(wrw + 0.5);                  lh = Int(wrh + 0.5)
    case "LandscapeLeft":
        lx = Int(wry + 0.5);                            ly = Int(logicalH - wrx - wrw + 0.5)
        lw = Int(wrh + 0.5);                            lh = Int(wrw + 0.5)
    case "LandscapeRight":
        lx = Int(logicalW - wry - wrh + 0.5);           ly = Int(wrx + 0.5)
        lw = Int(wrh + 0.5);                            lh = Int(wrw + 0.5)
    default: // Portrait
        lx = Int(wrx + 0.5); ly = Int(wry + 0.5)
        lw = Int(wrw + 0.5); lh = Int(wrh + 0.5)
    }
}

let safeLabel = label
    .replacingOccurrences(of: "\\", with: "\\\\")
    .replacingOccurrences(of: "\"", with: "\\\"")
print("{\"role\":\"\(role)\",\"label\":\"\(safeLabel)\",\"x\":\(lx),\"y\":\(ly),\"w\":\(lw),\"h\":\(lh)}")
SWIFTEOF
  )

  local swift_exit=$?
  if [[ $swift_exit -ne 0 ]] || [[ -z "$result" ]] || [[ "$result" == ERROR:* ]]; then
    die "describe-point: ${result:-no element found at ($1, $2)}"
  fi
  echo "$result"
}

# _describe_points_batch
# Hit-test many points in one swift invocation. Reads "logical_x logical_y"
# pairs from stdin (one per line) and writes one result per line to stdout —
# either a {role,label,x,y,w,h} JSON object for hits or the literal string
# "null" for points where AX returned no element.
#
# WHY: every cmd_describe_point call pays ~150ms swift cold-start. Calling it
# in a loop for 50+ points takes ~7s. Batching amortizes the swift startup
# across all points so the same workload runs in ~300-800ms (200ms startup +
# ~5-15ms per AX hit-test). Used by layout-map's verify pass.
#
# The output is positional — N input lines produce N output lines, even when
# some hit-tests fail. Callers can zip input and output by line index.
_describe_points_batch() {
  _bootstrap

  local sim_orientation
  sim_orientation=$(plutil -extract "DevicePreferences.${SIM_UDID}.SimulatorWindowOrientation" raw \
    ~/Library/Preferences/com.apple.iphonesimulator.plist 2>/dev/null || echo "Portrait")

  # Convert each "logical_x logical_y" line on stdin to a "screen_x screen_y"
  # line in bash before sending to swift. _ios_to_screen handles the orientation
  # / zoom / origin transform; doing it here keeps the swift script focused on
  # just the AX hit-test loop. Note: _ios_to_screen prints without a trailing
  # newline, so we wrap with echo to ensure one line per pair.
  local screen_pairs
  screen_pairs=$(while read -r lx ly; do
    if [[ -z "$lx" || -z "$ly" ]]; then
      echo ""
    else
      echo "$(_ios_to_screen "$lx" "$ly")"
    fi
  done)

  # Stage the swift script in a temp file so the input pipe can deliver coord
  # pairs to swift's stdin (a heredoc on `swift -` would shadow the pipe).
  local swift_script
  swift_script=$(mktemp -t sim_describe_batch).swift
  : > "$swift_script"
  cat > "$swift_script" << 'SWIFTEOF'
import ApplicationServices
import AppKit
import Foundation

let args = CommandLine.arguments
let originX     = Double(args[1])!
let originY     = Double(args[2])!
let zoom        = Double(args[3])!
let orientation = args.count > 4 ? args[4] : "Portrait"
let logicalW    = Double(args.count > 5 ? args[5] : "390") ?? 390.0
let logicalH    = Double(args.count > 6 ? args[6] : "844") ?? 844.0

guard let simApp = NSWorkspace.shared.runningApplications
    .first(where: { $0.bundleIdentifier == "com.apple.iphonesimulator" }) else {
    fputs("ERROR: Simulator not running\n", stderr); exit(1)
}
let pid = simApp.processIdentifier
let axApp = AXUIElementCreateApplication(pid)

func getAttr(_ el: AXUIElement, _ attr: String) -> AnyObject? {
    var v: CFTypeRef?
    guard AXUIElementCopyAttributeValue(el, attr as CFString, &v) == .success else { return nil }
    return v as AnyObject
}
func getLabel(_ el: AXUIElement) -> String {
    for a in ["AXDescription", "AXTitle", "AXLabel", "AXValue"] {
        if let s = getAttr(el, a) as? String, !s.isEmpty { return s }
    }
    return ""
}
func getRole(_ el: AXUIElement) -> String {
    return (getAttr(el, kAXRoleAttribute) as? String) ?? ""
}

while let line = readLine() {
    let trimmed = line.trimmingCharacters(in: .whitespaces)
    if trimmed.isEmpty { print("null"); continue }
    let parts = trimmed.split(separator: " ")
    guard parts.count >= 2,
          let sx = Double(parts[0]),
          let sy = Double(parts[1]) else {
        print("null"); continue
    }
    var element: AXUIElement?
    let err = AXUIElementCopyElementAtPosition(axApp, Float(sx), Float(sy), &element)
    guard err == .success, let el = element else {
        print("null"); continue
    }
    let role  = getRole(el)
    let label = getLabel(el)
    var lx = 0, ly = 0, lw = 0, lh = 0
    if let posVal = getAttr(el, kAXPositionAttribute),
       let sizeVal = getAttr(el, kAXSizeAttribute) {
        var pt = CGPoint.zero; var sz = CGSize.zero
        AXValueGetValue(posVal as! AXValue, .cgPoint, &pt)
        AXValueGetValue(sizeVal as! AXValue, .cgSize, &sz)
        let wrx = (Double(pt.x) - originX) / zoom
        let wry = (Double(pt.y) - originY) / zoom
        let wrw = Double(sz.width)  / zoom
        let wrh = Double(sz.height) / zoom
        switch orientation {
        case "PortraitUpsideDown":
            lx = Int(logicalW - wrx - wrw + 0.5); ly = Int(logicalH - wry - wrh + 0.5)
            lw = Int(wrw + 0.5);                  lh = Int(wrh + 0.5)
        case "LandscapeLeft":
            lx = Int(wry + 0.5);                            ly = Int(logicalH - wrx - wrw + 0.5)
            lw = Int(wrh + 0.5);                            lh = Int(wrw + 0.5)
        case "LandscapeRight":
            lx = Int(logicalW - wry - wrh + 0.5);           ly = Int(wrx + 0.5)
            lw = Int(wrh + 0.5);                            lh = Int(wrw + 0.5)
        default:
            lx = Int(wrx + 0.5); ly = Int(wry + 0.5)
            lw = Int(wrw + 0.5); lh = Int(wrh + 0.5)
        }
    }
    let safeLabel = label
        .replacingOccurrences(of: "\\", with: "\\\\")
        .replacingOccurrences(of: "\"", with: "\\\"")
    print("{\"role\":\"\(role)\",\"label\":\"\(safeLabel)\",\"x\":\(lx),\"y\":\(ly),\"w\":\(lw),\"h\":\(lh)}")
}
SWIFTEOF

  printf '%s\n' "$screen_pairs" | swift "$swift_script" \
    "$SIM_SCREEN_X" "$SIM_SCREEN_Y" "$SIM_ZOOM" "$sim_orientation" \
    "$SIM_LOGICAL_W" "$SIM_LOGICAL_H" 2>/dev/null
  rm -f "$swift_script"
}
