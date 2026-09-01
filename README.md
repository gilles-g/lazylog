# LazyLog

A terminal UI for exploring log files — Symfony/Monolog, nginx and Apache (access + error), PHP errors, generic text. Inspired by [lazygit](https://github.com/jesseduffield/lazygit), built with [ratatui](https://ratatui.rs) + [crossterm](https://github.com/crossterm-rs/crossterm).

![Rust](https://img.shields.io/badge/rust-stable-orange)

> **Beta** — under active development, expect breaking changes.

## Demo

Browse a Symfony/Monolog log: navigate, open detail, toggle facets, full-text search, cluster every line of a request with `*`.

![lazylog demo](.github/assets/demo.gif)

GeoIP **Country** facet on an nginx access log: filter by country, isolate a `masscan` scanner via search, drill into 4xx.

![GeoIP demo](.github/assets/demo-geoip.gif)

## Features

- **Memory-mapped, parsed in the background** — multi-GB files, no copy, live progress bar, `Esc` to stop and browse what's already loaded.
- **Transparent gzip** — a rotated `access.log.1.gz` opens like any other file.
- **Follow mode** (`F`) — live `tail -f`, new lines stream in at the bottom. Plain files only.
- **Trace correlation** (`*`) — filters on the `trace_id` / `request_id` / Monolog `token` of the selected event, clustering every line of that request.
- **Facets** — level, channel, exception, method, status, vhost, IP, subnet /24, country. Clickable, on top of full-text search and a date-range filter.
- **Export** (`x`) — facet tally or the filtered log, to `.txt`.
- Yank to clipboard (`y`), mouse (click + wheel), resizable panels (`Ctrl-N`), vim keys.
- Newest events at the bottom, like `tail -f`.

## Install

Pre-built binaries on the [releases page](https://github.com/gilles-g/lazylog/releases):

```bash
# swap the triple for aarch64-unknown-linux-gnu, x86_64-apple-darwin or aarch64-apple-darwin
curl -L https://github.com/gilles-g/lazylog/releases/latest/download/lazylog-x86_64-unknown-linux-gnu.tar.gz | tar -xz
mv lazylog ~/.local/bin/
```

Linux binaries are built against glibc 2.35 (Debian 12+, Ubuntu 22.04+).

From source:

```bash
cargo install --path .
```

## Usage

```bash
lazylog /var/log/nginx/access.log   # a file
lazylog /var/log/nginx              # a directory → picker limited to it
lazylog                             # no argument → picker over var/log, logs/, /var/log

lazylog --format nginx-access access.log                          # force detection
lazylog --from 2026-04-22 --to '2026-04-22 18:00:00' access.log   # load a time window only
lazylog --all huge.log                                            # skip the date prompt on files > 100 MB
```

`--format` accepts `symfony`, `php`, `nginx-access`, `nginx-error`, `apache-access`, `apache-error`, `generic`.

### Keybindings

| Key                 | Action                                              |
|---------------------|-----------------------------------------------------|
| `q` / `Ctrl-C`      | quit                                                |
| `?`                 | toggle help                                         |
| `j` `k` / `↓` `↑`   | move cursor (down = toward most recent)             |
| `g` / `G`           | oldest / newest                                     |
| `PgUp` / `PgDn`     | jump 10 lines                                       |
| `f` / `e`           | focus Facets / Events                               |
| `h` `l` / `←` `→`   | switch focus                                        |
| `Space`             | toggle a facet value                                |
| `/`                 | full-text search                                    |
| `d`                 | date-range modal                                    |
| `r`                 | reset all filters                                   |
| `x`                 | export menu                                         |
| `F`                 | toggle follow mode                                  |
| `*`                 | correlate on the selected event's trace id          |
| `y`                 | yank the selected line to the clipboard             |
| `Ctrl-N`            | resize mode (arrows to adjust, `Esc` to exit)       |
| mouse               | click to select, wheel to scroll                    |
| `Esc`               | close popup / clear search / stop loading           |

## Country facet (GeoIP)

Needs a country-level GeoIP2 `.mmdb` — DB-IP Lite, MaxMind GeoLite2, whichever you like, mind their licenses. Without one the facet simply doesn't appear.

```bash
mkdir -p ~/.local/share/lazylog
mv your.mmdb ~/.local/share/lazylog/geoip.mmdb
```

Lookup order: `--geoip PATH`, `$LAZYLOG_GEOIP`, `$XDG_DATA_HOME/lazylog/geoip.mmdb`, `~/.local/share/lazylog/geoip.mmdb`, `~/.lazylog/geoip.mmdb`.

Resolution happens at load time, in-process, cached per IP — no network call. Private ranges are skipped.

## Troubleshooting

Runtime log: `${XDG_CACHE_HOME:-~/.cache}/lazylog/lazylog.log`. Raise the level with `RUST_LOG=debug lazylog …`.

## License

MIT.
