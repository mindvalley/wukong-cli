# lib/simclaw/login.sh — sim login command
# Fills a standard email+password login form in any iOS app running in the simulator.
#
# Design rationale:
#   Login forms vary widely across iOS apps:
#     - Some use XCUIElementTypeTextField with proper accessibilityLabel.
#     - Some (especially SwiftUI) wrap the field in XCUIElementTypeOther, with the
#       visible "Email" / "Password" text rendered by a child SwiftUI Text view —
#       leaving the tappable button/container with no accessibilityLabel of its own.
#     - On iPad, class-name search for XCUIElementTypeTextField often returns 0
#       results regardless of how the field is built.
#
#   This command delegates label-search to sim's own `cmd_tap_element`, which
#   walks the full WDA AX tree and rolls up visible text from descendant elements
#   into the parent's effective label. That handles all three cases above without
#   apps needing to add explicit accessibilityLabel annotations. Focus is verified
#   via /element/active before calling cmd_type, and each field gets up to 3
#   attempts before the command fails.
#
# Usage:
#   sim --device <UDID> login --email <email> --password <password>
#
# Exit codes:
#   0 = login submitted successfully
#   1 = could not enter email after 3 attempts
#   2 = could not enter password after 3 attempts

[[ -n "${_SIM_LOGIN_LOADED:-}" ]] && return 0; _SIM_LOGIN_LOADED=1

source "$SIM_LIB/bootstrap.sh"
source "$SIM_LIB/wda.sh"
source "$SIM_LIB/nav.sh"
source "$SIM_LIB/touch.sh"
source "$SIM_LIB/type.sh"

# _login_tap_label <label>
# Tap an element by visible label using sim's AX tree traversal (cmd_tap_element).
# This handles SwiftUI apps where the visible text lives on a child Text view
# rather than the parent button's accessibilityLabel — cmd_tap_element rolls up
# child labels while parsing WDA /source, whereas raw WDA "link text" search
# (used previously) misses those buttons entirely.
#
# cmd_tap_element calls `die` (exit 1) when the label isn't in the current
# viewport; the subshell here isolates that exit so login.sh can continue to its
# next fallback label instead of aborting.
#
# Returns 0 if the tap succeeded, 1 if the label wasn't found.
_login_tap_label() {
  ( cmd_tap_element "$1" >/dev/null 2>&1 )
}

# _login_tap_button <label>
# Tap a BUTTON whose label matches (case-insensitive contains).
#
# Two-phase strategy to handle a SwiftUI/WDA quirk found in the wild on Mindvalley:
#
# Phase 1 (xpath find + hit-test verify):
#   Find the button via WDA xpath restricted to XCUIElementTypeButton (this
#   alone disambiguates the form-title vs submit-button case where both share
#   the same visible label). Then hit-test `describe-point` at the rect's
#   center. If the hit-test returns the same label, WDA's coords are trustworthy
#   and we tap there.
#
# Phase 2 (vertical AX scan fallback):
#   When a SwiftUI form uses a Spacer/.frame(maxHeight:.infinity) between the
#   last input and the submit button, WDA's /source XML reports the button at
#   its layout-intended y (with the spacer counted at full height) while the
#   real rendered y is much higher (spacer collapses to fit). Tapping at WDA's
#   y lands in empty space below the social-buttons row. The AX-based
#   describe-point hit-test, however, returns the correct rendered position.
#   So when phase 1's hit-test disagrees with the xpath rect, we walk down the
#   x-column from the top of the form via describe-point until we find an
#   AXButton with the matching label, then tap its center.
#
# Returns 0 if a tap was sent at a verified position, 1 otherwise.
_login_tap_button() {
  local label="$1"
  local lower
  lower=$(echo "$label" | tr '[:upper:]' '[:lower:]')

  # ── Phase 1: WDA xpath candidate + hit-test verify ──────────────────────────
  local resp eid
  resp=$(curl -sf -X POST -H "Content-Type: application/json" \
    -d "{\"using\":\"xpath\",\"value\":\"//XCUIElementTypeButton[contains(translate(@label,'ABCDEFGHIJKLMNOPQRSTUVWXYZ','abcdefghijklmnopqrstuvwxyz'),'${lower}')]\"}" \
    --max-time 5 \
    "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/elements" 2>/dev/null) || return 1
  eid=$(echo "$resp" | python3 -c "
import sys, json
try:
    e = json.load(sys.stdin).get('value', [])
    if e: print(list(e[0].values())[0])
except Exception: pass
" 2>/dev/null)

  local cx=""
  local cy=""
  if [[ -n "$eid" ]]; then
    local rect
    rect=$(curl -sf --max-time 5 \
      "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/element/${eid}/rect" 2>/dev/null)
    cx=$(echo "$rect" | python3 -c "
import sys, json
try:
    r = json.load(sys.stdin).get('value', {})
    print(int(r.get('x', 0) + r.get('width', 0) / 2))
except Exception: pass
" 2>/dev/null)
    cy=$(echo "$rect" | python3 -c "
import sys, json
try:
    r = json.load(sys.stdin).get('value', {})
    print(int(r.get('y', 0) + r.get('height', 0) / 2))
except Exception: pass
" 2>/dev/null)
  fi

  if [[ -n "$cx" && -n "$cy" ]]; then
    local hit_label
    hit_label=$(cmd_describe_point "$cx" "$cy" 2>/dev/null | python3 -c "
import sys, json
try:
    print(json.load(sys.stdin).get('label', ''))
except Exception: pass
" 2>/dev/null)
    if [[ -n "$hit_label" ]] && echo "$hit_label" | tr '[:upper:]' '[:lower:]' | grep -qF "$lower"; then
      cmd_tap "$cx" "$cy" >/dev/null 2>&1 && return 0
    fi
  fi

  # ── Phase 2: targeted AX scan upward from WDA's reported y ──────────────────
  # The known SwiftUI/WDA quirk only puts the button HIGHER than WDA reports
  # (Spacer collapses upward at render time). So we don't need to scan the
  # whole screen — just walk UP from WDA's wrong y in coarse steps until we
  # find an AXButton with the matching label, then tap its actual rect center.
  # Bounded to ~10 iterations max (300pt range / 30pt step) so a single call
  # finishes in under 2 seconds even on a fully-failing scan.
  if [[ -n "$cx" && -n "$cy" ]]; then
    local scan_y
    for ((scan_y = cy; scan_y >= cy - 300 && scan_y > 100; scan_y -= 30)); do
      local hit hit_role hit_lbl
      hit=$(cmd_describe_point "$cx" "$scan_y" 2>/dev/null) || continue
      hit_role=$(echo "$hit" | python3 -c "import sys,json
try: print(json.load(sys.stdin).get('role',''))
except Exception: pass" 2>/dev/null)
      hit_lbl=$(echo "$hit" | python3 -c "import sys,json
try: print(json.load(sys.stdin).get('label',''))
except Exception: pass" 2>/dev/null)
      if [[ "$hit_role" == "AXButton" ]] && \
         echo "$hit_lbl" | tr '[:upper:]' '[:lower:]' | grep -qF "$lower"; then
        # Use describe-point's rect (AX-reported, correct) and tap its center.
        local rx ry rw rh
        rx=$(echo "$hit" | python3 -c "import sys,json; print(json.load(sys.stdin).get('x',0))")
        ry=$(echo "$hit" | python3 -c "import sys,json; print(json.load(sys.stdin).get('y',0))")
        rw=$(echo "$hit" | python3 -c "import sys,json; print(json.load(sys.stdin).get('w',0))")
        rh=$(echo "$hit" | python3 -c "import sys,json; print(json.load(sys.stdin).get('h',0))")
        if cmd_tap $((rx + rw / 2)) $((ry + rh / 2)) >/dev/null 2>&1; then
          return 0
        fi
      fi
    done
  fi

  return 1
}

# _login_field_focused
# Returns 0 if WDA reports a focused active element, 1 otherwise.
# Must be called after tapping a field and sleeping 0.5s to allow iOS to settle.
_login_field_focused() {
  local resp
  resp=$(curl -sf \
    --max-time 5 \
    "http://localhost:${WDA_PORT}/session/${WDA_SESSION}/element/active" 2>/dev/null) || return 1
  # NOTE: `except Exception` (not bare `except`). Bare `except:` catches
  # `SystemExit` raised by `sys.exit(0)` itself, which then triggers the
  # except-branch `sys.exit(1)` and makes the function ALWAYS return 1.
  # This footgun made the focus check incorrectly fail every login attempt.
  echo "$resp" | python3 -c "
import sys, json
try:
    val = json.load(sys.stdin).get('value')
    sys.exit(0 if val else 1)
except Exception:
    sys.exit(1)
" 2>/dev/null
}

cmd_login() {
  local email="" password=""

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --email)    email="$2";    shift 2 ;;
      --password) password="$2"; shift 2 ;;
      *) die "login: unknown flag '$1'. Usage: sim --device <UDID> login --email <email> --password <password>" ;;
    esac
  done

  [[ -n "$email" ]]    || die "login: --email is required"
  [[ -n "$password" ]] || die "login: --password is required"

  _bootstrap
  _wda_ensure

  # ── Step 1: Open login form ──────────────────────────────────────────────────
  echo "login: opening login form..."
  local opened=0
  for label in "Log in" "Log In" "Login" "Sign in" "Sign In" "Sign Up" "Continue"; do
    if _login_tap_label "$label"; then
      echo "login: tapped '$label'"
      sleep 1.2
      opened=1
      break
    fi
  done
  [[ $opened -eq 1 ]] || echo "login: no login button found — assuming already on login form"

  # ── Step 2: Enter email (max 3 attempts, strategy escalates on failure) ──────
  echo "login: entering email..."
  local email_done=0 attempt
  for attempt in 1 2 3; do
    for label in "Email" "Email or Username" "Username" "email" "E-mail" "Phone / Email / Username" "Phone, email, or username"; do
      if _login_tap_label "$label"; then
        sleep 0.5
        if _login_field_focused; then
          cmd_type "$email"
          echo "login: email entered (attempt ${attempt}, field='${label}')"
          email_done=1
          break
        fi
      fi
    done
    [[ $email_done -eq 1 ]] && break

    echo "login: attempt ${attempt}: email field not focused — escalating"
    # Escalation: dismiss any stray keyboard/modal then re-open the login form
    for reset in "Cancel" "cancel" "Close" "Dismiss"; do
      _login_tap_label "$reset" || true
    done
    sleep 0.5
    for label in "Log in" "Log In" "Login" "Sign in" "Sign In"; do
      _login_tap_label "$label" && sleep 0.8 && break || true
    done
  done

  if [[ $email_done -ne 1 ]]; then
    echo "login: FATAL — could not enter email after 3 attempts" >&2
    exit 1
  fi

  # Dismiss keyboard / advance to password field
  sleep 0.3
  for label in "Next" "next" "Return"; do
    _login_tap_label "$label" && break || true
  done
  sleep 0.3

  # ── Step 3: Enter password (max 3 attempts) ───────────────────────────────────
  echo "login: entering password..."
  local password_done=0
  for attempt in 1 2 3; do
    for label in "Password" "password" "Pass"; do
      if _login_tap_label "$label"; then
        sleep 0.5
        if _login_field_focused; then
          cmd_type "$password"
          echo "login: password entered (attempt ${attempt})"
          password_done=1
          break
        fi
      fi
    done
    [[ $password_done -eq 1 ]] && break

    echo "login: attempt ${attempt}: password field not focused — retrying"
    sleep 0.5
  done

  if [[ $password_done -ne 1 ]]; then
    echo "login: FATAL — could not enter password after 3 attempts" >&2
    exit 2
  fi

  # ── Step 4: Submit ────────────────────────────────────────────────────────────
  # Use _login_tap_button (xpath restricted to XCUIElementTypeButton) instead of
  # _login_tap_label here so we don't accidentally tap a same-text form heading
  # instead of the actual submit button — see _login_tap_button for the full
  # rationale and the Mindvalley "Log In" duplicate-label example.
  echo "login: submitting..."
  sleep 0.3
  for label in "Log in" "Log In" "Login" "Sign in" "Sign In" "Continue" "Submit" "Done"; do
    if _login_tap_button "$label"; then
      echo "login: tapped submit '${label}'"
      break
    fi
  done

  echo "login: waiting for authentication (3s)..."
  sleep 3
  echo "login: done"
}
