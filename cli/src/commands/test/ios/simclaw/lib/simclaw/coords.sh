# lib/simclaw/coords.sh — iOS logical ↔ macOS screen coordinate mapping
[[ -n "${_SIM_COORDS_LOADED:-}" ]] && return 0; _SIM_COORDS_LOADED=1

# _ios_to_screen <lx> <ly>
# Converts iOS logical coordinates to macOS screen coordinates, accounting for device orientation.
# Reads orientation from com.apple.iphonesimulator.plist using SIM_UDID (set by _bootstrap).
# Outputs: "<sx> <sy>" space-separated macOS screen coordinates.
_ios_to_screen() {
  local lx="$1" ly="$2"
  local orient
  orient=$(plutil -extract "DevicePreferences.${SIM_UDID}.SimulatorWindowOrientation" raw \
    ~/Library/Preferences/com.apple.iphonesimulator.plist 2>/dev/null || echo "Portrait")
  case "$orient" in
    PortraitUpsideDown)
      awk "BEGIN { printf \"%d %d\", \
        ${SIM_SCREEN_X} + int((${SIM_LOGICAL_W} - ${lx}) * ${SIM_ZOOM} + 0.5), \
        ${SIM_SCREEN_Y} + int((${SIM_LOGICAL_H} - ${ly}) * ${SIM_ZOOM} + 0.5) }"
      ;;
    LandscapeLeft)
      # iOS (lx, ly) -> macOS: originX + (logicalH - ly)*zoom, originY + lx*zoom
      awk "BEGIN { printf \"%d %d\", \
        ${SIM_SCREEN_X} + int((${SIM_LOGICAL_H} - ${ly}) * ${SIM_ZOOM} + 0.5), \
        ${SIM_SCREEN_Y} + int(${lx} * ${SIM_ZOOM} + 0.5) }"
      ;;
    LandscapeRight)
      # iOS (lx, ly) -> macOS: originX + ly*zoom, originY + (logicalW - lx)*zoom
      awk "BEGIN { printf \"%d %d\", \
        ${SIM_SCREEN_X} + int(${ly} * ${SIM_ZOOM} + 0.5), \
        ${SIM_SCREEN_Y} + int((${SIM_LOGICAL_W} - ${lx}) * ${SIM_ZOOM} + 0.5) }"
      ;;
    *)  # Portrait (default)
      awk "BEGIN { printf \"%d %d\", \
        ${SIM_SCREEN_X} + int(${lx} * ${SIM_ZOOM} + 0.5), \
        ${SIM_SCREEN_Y} + int(${ly} * ${SIM_ZOOM} + 0.5) }"
      ;;
  esac
}

# Keep backward-compat wrappers that use _ios_to_screen internally.
# NOTE: these are only correct for Portrait orientation. Callers that need
# accurate non-portrait coordinates should use _ios_to_screen directly.
to_screen_x() { _ios_to_screen "$1" 0 | awk '{print $1}'; }
to_screen_y() { _ios_to_screen 0 "$1" | awk '{print $2}'; }
