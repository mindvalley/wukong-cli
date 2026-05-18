func pickWindow(_ windows: [AXUIElement], screenX: Double, screenY: Double, deviceName: String) -> AXUIElement? {
    // Primary: find the window whose bounds contain the known screen origin.
    // SIM_SCREEN_X/Y are the macOS screen coords of the iOS screen surface — unique per window.
    for win in windows {
        var posRef: CFTypeRef?
        var sizeRef: CFTypeRef?
        guard AXUIElementCopyAttributeValue(win, kAXPositionAttribute as CFString, &posRef) == .success,
              AXUIElementCopyAttributeValue(win, kAXSizeAttribute as CFString, &sizeRef) == .success,
              let pv = posRef, let sv = sizeRef else { continue }
        var pt = CGPoint.zero; var sz = CGSize.zero
        AXValueGetValue(pv as! AXValue, .cgPoint, &pt)
        AXValueGetValue(sv as! AXValue, .cgSize, &sz)
        if screenX >= Double(pt.x) && screenX <= Double(pt.x + sz.width) &&
           screenY >= Double(pt.y) && screenY <= Double(pt.y + sz.height) {
            return win
        }
    }
    // Fallback: match by title containing device name
    for win in windows {
        var titleRef: CFTypeRef?
        if AXUIElementCopyAttributeValue(win, kAXTitleAttribute as CFString, &titleRef) == .success,
           let title = titleRef as? String, title.contains(deviceName) { return win }
    }
    return windows.first
}
