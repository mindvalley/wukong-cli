# lib/simclaw/wait.sh — wait-for and wait-for-stable commands
[[ -n "${_SIM_WAIT_LOADED:-}" ]] && return 0; _SIM_WAIT_LOADED=1

source "$SIM_LIB/bootstrap.sh"

cmd_wait_for() {
  [[ $# -ge 1 ]] || die "Usage: sim wait-for <label> [timeout_seconds]"
  local search_label="$1"
  local timeout="${2:-10}"
  # Bootstrap to get SIM_NAME for window targeting when multiple simulators are open.
  _bootstrap

  local out exit_code
  out=$(swift - "$search_label" "$timeout" "${SIM_NAME} – ${SIM_OS}" "$SIM_SCREEN_X" "$SIM_SCREEN_Y" 2>&1 << SWIFTEOF
import ApplicationServices
import AppKit
import Foundation

let args = CommandLine.arguments
let searchLabel = args.count > 1 ? args[1] : ""
let timeoutSec  = Double(args.count > 2 ? args[2] : "10") ?? 10.0
let deviceName  = args.count > 3 ? args[3] : ""
let screenX     = Double(args.count > 4 ? args[4] : "0") ?? 0.0
let screenY     = Double(args.count > 5 ? args[5] : "0") ?? 0.0

guard let simApp = NSWorkspace.shared.runningApplications
    .first(where: { \$0.bundleIdentifier == "com.apple.iphonesimulator" }) else {
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
func treeContains(_ el: AXUIElement, _ label: String, depth: Int = 0) -> Bool {
    guard depth < 25 else { return false }
    if getLabel(el).localizedCaseInsensitiveContains(label) { return true }
    guard let ch = getAttr(el, "AXChildren") as? [AXUIElement] else { return false }
    return ch.contains { treeContains(\$0, label, depth: depth + 1) }
}

$(cat "$SIM_LIB/swift/pickwindow.swift")

func checkTree() -> Bool {
    var ref: CFTypeRef?
    AXUIElementCopyAttributeValue(axApp, "AXWindows" as CFString, &ref)
    guard let wins = ref as? [AXUIElement],
          let win = pickWindow(wins, screenX: screenX, screenY: screenY, deviceName: deviceName) else { return false }
    return treeContains(win, searchLabel)
}

// Immediate check
if checkTree() { print("READY: \(searchLabel)"); exit(0) }

// Shared found flag
final class State { var found = false }
let state = State()

// AXObserver callback
let callback: AXObserverCallback = { _, _, _, refcon in
    let s = Unmanaged<State>.fromOpaque(refcon!).takeUnretainedValue()
    guard !s.found else { return }
    if checkTree() {
        s.found = true
        print("READY: \(searchLabel)")
        CFRunLoopStop(CFRunLoopGetCurrent())
    }
}

var observer: AXObserver?
guard AXObserverCreate(pid, callback, &observer) == .success, let obs = observer else {
    fputs("ERROR: AXObserverCreate failed\n", stderr); exit(1)
}
let ctxPtr = Unmanaged.passRetained(state).toOpaque()
CFRunLoopAddSource(CFRunLoopGetCurrent(), AXObserverGetRunLoopSource(obs), .defaultMode)
AXObserverAddNotification(obs, axApp, kAXLayoutChangedNotification as CFString, ctxPtr)

// Safety-net polling (1.5s interval) catches async changes between AX notifications
let pollTimer = DispatchSource.makeTimerSource(queue: .main)
pollTimer.schedule(deadline: .now() + 1.5, repeating: 1.5)
pollTimer.setEventHandler {
    guard !state.found else { pollTimer.cancel(); return }
    if checkTree() {
        state.found = true
        print("READY: \(searchLabel)")
        CFRunLoopStop(CFRunLoopGetCurrent())
    }
}
pollTimer.resume()

// Timeout
let timeoutTimer = DispatchSource.makeTimerSource(queue: .main)
timeoutTimer.schedule(deadline: .now() + timeoutSec)
timeoutTimer.setEventHandler { CFRunLoopStop(CFRunLoopGetCurrent()) }
timeoutTimer.resume()

CFRunLoopRun()
pollTimer.cancel()
timeoutTimer.cancel()

if state.found { exit(0) }
fputs("ERROR: wait-for timed out after \(Int(timeoutSec))s waiting for: \(searchLabel)\n", stderr)
exit(1)
SWIFTEOF
  )
  exit_code=$?
  if [[ $exit_code -eq 0 ]]; then
    echo "$out"
  else
    die "${out#ERROR: }"
  fi
}

cmd_wait_for_stable() {
  local timeout="${1:-5}"
  _bootstrap

  local out exit_code
  out=$(swift - "$timeout" "${SIM_NAME} – ${SIM_OS}" "$SIM_SCREEN_X" "$SIM_SCREEN_Y" 2>&1 << SWIFTEOF
import ApplicationServices
import AppKit
import Foundation

let args = CommandLine.arguments
let timeoutSec = Double(args.count > 1 ? args[1] : "5") ?? 5.0
let deviceName = args.count > 2 ? args[2] : ""
let screenX    = Double(args.count > 3 ? args[3] : "0") ?? 0.0
let screenY    = Double(args.count > 4 ? args[4] : "0") ?? 0.0

guard let simApp = NSWorkspace.shared.runningApplications
    .first(where: { \$0.bundleIdentifier == "com.apple.iphonesimulator" }) else {
    fputs("ERROR: Simulator not running\n", stderr); exit(1)
}
let pid = simApp.processIdentifier
let axApp = AXUIElementCreateApplication(pid)

func getAttr(_ el: AXUIElement, _ attr: String) -> AnyObject? {
    var v: CFTypeRef?
    guard AXUIElementCopyAttributeValue(el, attr as CFString, &v) == .success else { return nil }
    return v as AnyObject
}

$(cat "$SIM_LIB/swift/pickwindow.swift")

func snapshot(_ el: AXUIElement, _ depth: Int) -> String {
    guard depth >= 0 else { return "" }
    var parts: [String] = []
    for attr in ["AXRole", "AXLabel", "AXTitle", "AXDescription", "AXValue"] {
        if let v = getAttr(el, attr) as? String, !v.isEmpty { parts.append("\(attr)=\(v)") }
    }
    if let posVal = getAttr(el, kAXPositionAttribute),
       let sizeVal = getAttr(el, kAXSizeAttribute) {
        var pt = CGPoint.zero; var sz = CGSize.zero
        AXValueGetValue(posVal as! AXValue, .cgPoint, &pt)
        AXValueGetValue(sizeVal as! AXValue, .cgSize, &sz)
        parts.append("pos=\(Int(pt.x)),\(Int(pt.y))|sz=\(Int(sz.width)),\(Int(sz.height))")
    }
    guard let children = getAttr(el, "AXChildren") as? [AXUIElement] else { return parts.joined(separator: "|") }
    let childSnaps = children.map { snapshot(\$0, depth - 1) }.joined(separator: ";")
    return parts.joined(separator: "|") + "{" + childSnaps + "}"
}
func takeSnapshot() -> String {
    var ref: CFTypeRef?
    AXUIElementCopyAttributeValue(axApp, "AXWindows" as CFString, &ref)
    guard let wins = ref as? [AXUIElement],
          let win = pickWindow(wins, screenX: screenX, screenY: screenY, deviceName: deviceName) else { return "" }
    return snapshot(win, 25)
}

let deadline = Date().addingTimeInterval(timeoutSec)
var prev = takeSnapshot()
repeat {
    Thread.sleep(forTimeInterval: 0.2)
    let curr = takeSnapshot()
    if curr == prev && !curr.isEmpty { print("STABLE"); exit(0) }
    prev = curr
} while Date() < deadline
fputs("ERROR: wait-for-stable timed out after \(Int(timeoutSec))s\n", stderr)
exit(1)
SWIFTEOF
  )
  exit_code=$?
  if [[ $exit_code -eq 0 ]]; then
    echo "$out"
  else
    die "${out#ERROR: }"
  fi
}
