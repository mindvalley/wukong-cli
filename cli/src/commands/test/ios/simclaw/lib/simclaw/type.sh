# lib/simclaw/type.sh — text input commands
[[ -n "${_SIM_TYPE_LOADED:-}" ]] && return 0; _SIM_TYPE_LOADED=1

source "$SIM_LIB/bootstrap.sh"
source "$SIM_LIB/wda.sh"

# _type_cgevent <text>
# Sets text in the currently focused field via the AX API (no keyboard events needed)
_type_cgevent() {
  local text="$1"
  # Pass the text via environment variable so the quoted heredoc can stay static
  local out
  out=$(SIM_TYPE_TEXT="$text" swift - "${SIM_NAME} – ${SIM_OS}" "$SIM_SCREEN_X" "$SIM_SCREEN_Y" 2>&1 << SWIFTEOF
import Cocoa
import ApplicationServices

let args = CommandLine.arguments
let deviceName = args.count > 1 ? args[1] : ""
let screenX    = Double(args.count > 2 ? args[2] : "0") ?? 0.0
let screenY    = Double(args.count > 3 ? args[3] : "0") ?? 0.0

guard let app = NSRunningApplication.runningApplications(
    withBundleIdentifier: "com.apple.iphonesimulator").first else {
  fputs("ERROR: Simulator not running\n", stderr); exit(1)
}

let axApp = AXUIElementCreateApplication(app.processIdentifier)

func getAttribute(_ el: AXUIElement, _ attr: String) -> CFTypeRef? {
    var v: CFTypeRef?
    AXUIElementCopyAttributeValue(el, attr as CFString, &v)
    return v
}

let textRoles: Set<String> = ["AXTextField", "AXTextArea", "AXSearchField"]

func isSettableTextField(_ el: AXUIElement) -> Bool {
    guard let role = getAttribute(el, "AXRole") as? String,
          textRoles.contains(role) else { return false }
    var settable: DarwinBoolean = false
    AXUIElementIsAttributeSettable(el, "AXValue" as CFString, &settable)
    return settable.boolValue
}

func findFocusedTextField(_ el: AXUIElement, _ depth: Int) -> AXUIElement? {
    if depth < 0 { return nil }
    if let focused = getAttribute(el, "AXFocused") as? Bool, focused,
       isSettableTextField(el) {
        return el
    }
    guard let children = getAttribute(el, "AXChildren") as? [AXUIElement] else { return nil }
    for child in children {
        if let found = findFocusedTextField(child, depth - 1) { return found }
    }
    return nil
}

func findFirstSettableTextField(_ el: AXUIElement, _ depth: Int) -> AXUIElement? {
    if depth < 0 { return nil }
    if isSettableTextField(el) { return el }
    guard let children = getAttribute(el, "AXChildren") as? [AXUIElement] else { return nil }
    for child in children {
        if let found = findFirstSettableTextField(child, depth - 1) { return found }
    }
    return nil
}

$(cat "$SIM_LIB/swift/pickwindow.swift")

guard let windows = getAttribute(axApp, "AXWindows") as? [AXUIElement],
      let window = pickWindow(windows, screenX: screenX, screenY: screenY, deviceName: deviceName) else {
    fputs("ERROR: no Simulator window\n", stderr); exit(1)
}

// 1. Try the shortcut focused element if it is itself a settable text field
var focusedEl: AXUIElement? = nil
if let f = getAttribute(axApp, "AXFocusedUIElement") {
    let candidate = f as! AXUIElement
    if isSettableTextField(candidate) {
        focusedEl = candidate
    }
}

// 2. Walk tree for a focused+settable text field
if focusedEl == nil {
    focusedEl = findFocusedTextField(window, 25)
}

// 3. Fall back: first settable text field anywhere in the tree
if focusedEl == nil {
    focusedEl = findFirstSettableTextField(window, 25)
}

guard let target = focusedEl else {
    fputs("ERROR: no focused element found — tap the text field first\n", stderr); exit(1)
}

// Read text from the environment variable
let text = ProcessInfo.processInfo.environment["SIM_TYPE_TEXT"] ?? ""

// Set the value directly
let result = AXUIElementSetAttributeValue(target, "AXValue" as CFString, text as CFTypeRef)
if result != .success {
    fputs("ERROR: AXUIElementSetAttributeValue failed (code \(result.rawValue)) — field may not support direct value setting\n", stderr)
    exit(1)
}

print("OK")
SWIFTEOF
  ) || die "type: failed — $out"

  if [[ "$out" == ERROR:* ]]; then
    die "type: $out"
  fi
}

cmd_type() {
  [[ $# -ge 1 ]] || die "Usage: sim type <text>"
  local text="$1"
  _bootstrap

  # Prefer WDA setValue (goes through proper iOS input path, handles Unicode/spaces correctly).
  # Falls back to AX direct-value setting if WDA is not running or has no active element.
  if _wda_type "$text"; then
    echo "Typed (WDA): $text"
    return 0
  fi

  # WDA not available or no active element — fall back to AX API
  _type_cgevent "$text"
  echo "Typed: $text"
}
