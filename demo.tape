# vhs demo.tape — generate demo.gif for the README
#
# Prereqs:  cargo build --release  &&  go install github.com/charmbracelet/vhs@latest
#           + system packages: ttyd, ffmpeg
# Render:   vhs demo.tape           → demo.gif (alongside this file)

Output .github/assets/demo.gif

Set Shell       "bash"
Set FontSize    14
Set Width       1280
Set Height      720
Set Padding     20
Set Theme       "Catppuccin Mocha"
Set TypingSpeed 60ms
Set PlaybackSpeed 1.0

# ── Launch on a Symfony/Monolog sample with trace_ids ────────────────────────
Type "./target/release/lazylog testdata/demo.log"
Sleep 500ms
Enter
Sleep 2s

# ── Browse events, open detail on the highlighted line ───────────────────────
Down@150ms 4
Sleep 800ms
Enter
Sleep 2500ms
Escape
Sleep 600ms

# ── Jump to Facets, toggle a level filter (ERROR only) ───────────────────────
Type "f"
Sleep 600ms
Down@150ms 2
Sleep 400ms
Space
Sleep 1500ms

# ── Reset filters, back to Events panel ──────────────────────────────────────
Type "r"
Sleep 800ms
Type "e"
Sleep 400ms

# ── Text search — Enter applies the filter, r resets ────────────────────────
Type "/"
Sleep 400ms
Type "authenticated"
Sleep 800ms
Enter
Sleep 2s
Type "r"
Sleep 800ms

# ── Correlate: land on a payment error, cluster every line of that request ──
Down@120ms 14
Sleep 800ms
Type "*"
Sleep 2500ms
Type "r"
Sleep 800ms

# ── Help overlay ─────────────────────────────────────────────────────────────
Type "?"
Sleep 3s
Type "?"
Sleep 600ms

# ── Quit ─────────────────────────────────────────────────────────────────────
Type "q"
Sleep 800ms
