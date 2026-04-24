# vhs demo-geoip.tape — showcase GeoIP country facet on an access log
#
# Prereqs:  cargo build --release
#           a .mmdb at ~/.local/share/lazylog/geoip.mmdb  (or $LAZYLOG_GEOIP)
#           go install github.com/charmbracelet/vhs@latest
#           system packages: ttyd, ffmpeg
# Render:   vhs demo-geoip.tape      → demo-geoip.gif

Output .github/assets/demo-geoip.gif

Set Shell       "bash"
Set FontSize    14
Set Width       1280
Set Height      720
Set Padding     20
Set Theme       "Catppuccin Mocha"
Set TypingSpeed 90ms
Set PlaybackSpeed 1.0

# ── Launch on an nginx access log with worldwide IPs ─────────────────────────
Type "./target/release/lazylog testdata/access-demo.log"
Sleep 800ms
Enter
Sleep 4s

# ── Focus Facets. Cursor starts on the first value of the first group ───────
# ── (Status class → 2xx). Scroll all the way down to the Country group. ─────
Type "f"
Sleep 1500ms

# Walk past: Status class (3), Status (7), Method (4) ≈ 14 values before Country.
# Adjust the count below if your build shows more/fewer values.
Down@200ms 14
Sleep 3s

# ── Toggle the top country (depends on GeoIP data) ──────────────────────────
Space
Sleep 3500ms
Space
Sleep 1500ms

# ── Try another country to show geographic diversity ────────────────────────
Down@200ms 1
Sleep 1s
Space
Sleep 3500ms
Space
Sleep 1500ms

# ── Reset, then isolate the suspicious scanner by UA / status ───────────────
Type "r"
Sleep 1500ms
Type "e"
Sleep 800ms

# Highlight the scanner with full-text search on its user-agent
Type "/"
Sleep 800ms
Type "masscan"
Sleep 1200ms
Enter
Sleep 4s

# Reset and drill into 4xx via the Status-class facet instead
Type "r"
Sleep 1200ms
Type "f"
Sleep 800ms
Down@200ms 1
Sleep 800ms
Space
Sleep 4s
Space
Sleep 1200ms

# ── Reset and quit ───────────────────────────────────────────────────────────
Type "r"
Sleep 1500ms
Type "q"
Sleep 1s
