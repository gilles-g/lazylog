# LazyLog

A terminal user interface (TUI) for exploring log files (Symfony / Monolog, nginx access & error, Apache access & error, PHP errors, generic text), inspired by [lazygit](https://github.com/jesseduffield/lazygit).

![Rust](https://img.shields.io/badge/rust-stable-orange)

> **Beta** — This project is under active development. Expect rough edges and breaking changes.

Built with [ratatui](https://ratatui.rs) + [crossterm](https://github.com/crossterm-rs/crossterm).

## Features

- **Memory-mapped file opening** — no copy, handles multi-GB files.
- **Background parsing** — the UI never blocks.
- **Clickable facets** (level, channel, method, status, IP, country…), full-text search, date-range filter, time histogram.
- **Vim-style keyboard navigation**: `j/k`, `g/G`, `PgUp/PgDn`.
- The **most recent events are displayed at the bottom** of the list, like `tail -f`.

## Prerequisites

- Rust (stable) — only needed if you build from source

## Installation

### From GitHub releases (recommended)

Pre-built binaries are available on the [releases page](https://github.com/gilles-g/lazylog/releases).

Download the archive matching your platform, extract it, and move the binary to a directory in your `PATH`:

```bash
# Example for Linux x86_64 — adjust the version and asset name as needed
curl -L https://github.com/gilles-g/lazylog/releases/latest/download/lazylog-linux-x86_64.tar.gz \
  | tar -xz
mv lazylog ~/.local/bin/
```

### From source

```bash
git clone https://github.com/gilles-g/lazylog.git
cd lazylog
cargo build --release
```

The binary will be at `./target/release/lazylog`.

To make it available globally, add it to your `PATH`:

```bash
cp ./target/release/lazylog ~/.local/bin/
```

> Make sure `~/.local/bin` is in your `PATH`. If not, add this to your shell config (`~/.bashrc`, `~/.zshrc`, etc.):
>
> ```bash
> export PATH="$HOME/.local/bin:$PATH"
> ```

### From crates.io

```bash
cargo install --path .
```

## Usage

```bash
# Open a file directly
lazylog /var/log/nginx/access.log

# No path → interactive picker that scans var/log, logs/ and /var/log
lazylog

# Force a format if auto-detection gets it wrong
lazylog --format nginx-access access.log

# Restrict the loaded time range (faster on large files)
lazylog --from '2026-04-22' --to '2026-04-22 18:00:00' access.log
lazylog --all huge.log   # disables the date-range prompt for files > 100 MB
```

Recognized values for `--format`:
`symfony`, `php`, `nginx-access`, `nginx-error`, `apache-access`, `apache-error`, `generic`.

### Keybindings

| Key              | Action                                      |
|------------------|---------------------------------------------|
| `q` / `Ctrl-C`   | quit                                        |
| `?`              | show / hide help                            |
| `1` / `2`        | Events / Histogram tab                      |
| `j` / `↓`        | move down (toward most recent)              |
| `k` / `↑`        | move up (toward oldest)                     |
| `g`              | top of the list (oldest)                    |
| `G`              | bottom of the list (most recent, tail)      |
| `PgUp` / `PgDn`  | jump 10 lines                               |
| `f` / `e`        | focus Facets / Events panel                 |
| `Space`          | toggle a facet value                        |
| `/`              | full-text search                            |
| `d`              | date-range modal                            |
| `r`              | reset all filters                           |
| `x`              | export menu (focused facet / filtered log → `.txt`) |
| `Enter`          | open detail (coming soon)                   |
| `Esc`            | close popup / clear search                  |

## "Country" facet via GeoIP

`lazylog` can display a **Country** facet on nginx/apache access (and error)
logs, resolving each client IP to its country using a GeoIP2 database in
`.mmdb` format. Without a database, the facet simply does not appear — the
rest of the TUI works normally.

### 1. Obtain an `.mmdb` database

You need a free GeoIP2 country database in MaxMind DB (`.mmdb`) format. Several
free providers exist (DB-IP Lite, MaxMind GeoLite2, etc.) — pick one, respect
its license, and figure out how to download the file yourself. Any
`country`-level `.mmdb` will work.

### 2. Place the database in an auto-detected location

`lazylog` looks, in order, at:

1. The file passed to `--geoip /path/to/geoip.mmdb`
2. `$LAZYLOG_GEOIP` (environment variable)
3. `$XDG_DATA_HOME/lazylog/geoip.mmdb`
4. `~/.local/share/lazylog/geoip.mmdb`
5. `~/.lazylog/geoip.mmdb`

Typical install (no flag needed afterwards):

```bash
mkdir -p ~/.local/share/lazylog
mv /path/to/your.mmdb ~/.local/share/lazylog/geoip.mmdb
```

Or one-off usage:

```bash
lazylog --geoip ~/Downloads/dbip.mmdb access.log
# or
LAZYLOG_GEOIP=~/Downloads/dbip.mmdb lazylog access.log
```

### 3. Verify

At startup, a log line is written to `$XDG_CACHE_HOME/lazylog/lazylog.log`
(or `~/.cache/lazylog/lazylog.log`):

```
[INFO  lazylog] geoip database loaded: /home/you/.local/share/lazylog/geoip.mmdb
```

In the TUI, on an nginx/apache access log, a **Country** section appears in
the Facets panel (top 15 countries by volume). Press `Space` to filter.

### Notes

- Resolution is done at load time, in the background, with an in-memory cache
  per IP (frequent IPs only pay the cost once).
- No network request is made at runtime: the `.mmdb` database is fully local.
- Private IPs (10.0.0.0/8, 192.168.0.0/16, etc.) are not geolocated and do
  not appear in the facet.
- Respect the license of your chosen database if you redistribute the results.

## Application log

If something goes wrong with parsing or loading, the log lives at:

```
$XDG_CACHE_HOME/lazylog/lazylog.log
# or, as a fallback:
~/.cache/lazylog/lazylog.log
```

Log level tunable via `RUST_LOG=debug lazylog …`.

## License

MIT.
