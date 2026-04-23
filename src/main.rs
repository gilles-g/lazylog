use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset, Utc};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use std::sync::Arc;

use lazylog::log::detect::detect_from_path;
use lazylog::log::format::LogFormat;
use lazylog::log::geoip::{self, GeoDb};
use lazylog::log::loader;
use lazylog::log::scanner;
use lazylog::log::source::FileSource;
use lazylog::query::filter::Filter;
use lazylog::ui::app::App;
use lazylog::ui::panels::daterange::{parse_bound, DateRangeModal, Mode as DateRangeMode, Preset};
use lazylog::ui::panels::picker::{PickerOutcome, PickerState};

type DateBound = Option<DateTime<FixedOffset>>;

/// Files larger than this threshold trigger a mandatory date-range prompt
/// unless --from / --to are provided.
const LARGE_FILE_BYTES: u64 = 100 * 1024 * 1024;

#[derive(Parser, Debug)]
#[command(
    name = "lazylog",
    version = concat!(
        env!("CARGO_PKG_VERSION"),
        " (commit ", env!("LL_GIT_COMMIT"),
        ", built ", env!("LL_BUILD_DATE"),
        ", ", env!("LL_OS"), "/", env!("LL_ARCH"), ")",
    ),
    about = "TUI for browsing log files (Symfony, nginx, apache, PHP, …)"
)]
struct Cli {
    /// Path to the log file. If omitted, a picker scans var/log, logs/ and /var/log.
    path: Option<PathBuf>,

    /// Force a specific format (symfony, php, nginx-access, nginx-error, apache-access, apache-error, generic).
    #[arg(long)]
    format: Option<String>,

    /// Skip events before this timestamp. Accepts RFC3339 ("2026-04-22T10:00:00Z"),
    /// "YYYY-MM-DD HH:MM:SS", or "YYYY-MM-DD".
    #[arg(long)]
    from: Option<String>,

    /// Skip events after this timestamp (same formats as --from).
    #[arg(long)]
    to: Option<String>,

    /// Skip the upfront date-range prompt on large files (load everything).
    #[arg(long)]
    all: bool,

    /// Path to a GeoIP2 / GeoLite2 .mmdb database. Enables a "Country" facet
    /// on access logs. Also read from $LAZYLOG_GEOIP, or auto-detected under
    /// $XDG_DATA_HOME/lazylog/, ~/.local/share/lazylog/, ~/.lazylog/.
    #[arg(long)]
    geoip: Option<PathBuf>,
}

fn main() -> Result<()> {
    init_logger();
    let cli = Cli::parse();

    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal, cli);
    restore_terminal(&mut terminal)?;
    result
}

fn run<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, cli: Cli) -> Result<()> {
    let path = match cli.path {
        Some(p) => p,
        None => match pick_file(terminal)? {
            Some(p) => p,
            None => return Ok(()),
        },
    };

    if !path.is_file() {
        anyhow::bail!("not a regular file: {}", path.display());
    }

    let format = match cli.format.as_deref() {
        Some(s) => s
            .parse::<LogFormat>()
            .map_err(|e| anyhow::anyhow!(e))
            .context("invalid --format")?,
        None => detect_from_path(&path),
    };

    let mut filter = Filter::default();
    if let Some(s) = cli.from.as_deref() {
        filter.from = Some(
            parse_bound(s)
                .map_err(|e| anyhow::anyhow!(e))
                .with_context(|| format!("invalid --from: {s}"))?
                .ok_or_else(|| anyhow::anyhow!("--from cannot be empty"))?,
        );
    }
    if let Some(s) = cli.to.as_deref() {
        filter.to = Some(
            parse_bound(s)
                .map_err(|e| anyhow::anyhow!(e))
                .with_context(|| format!("invalid --to: {s}"))?
                .ok_or_else(|| anyhow::anyhow!("--to cannot be empty"))?,
        );
    }

    let file_size = path.metadata().map(|m| m.len()).unwrap_or(0);
    if file_size >= LARGE_FILE_BYTES && filter.from.is_none() && filter.to.is_none() && !cli.all {
        let reference = probe_reference(&path, format)
            .unwrap_or_else(|| Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap()));
        if let Some((from, to)) = prompt_date_range(terminal, &path, file_size, reference)? {
            filter.from = from;
            filter.to = to;
        } else {
            // user cancelled
            return Ok(());
        }
    }

    let geo = open_geoip(cli.geoip.as_deref());
    let mut app = App::new(path, format, filter, geo);
    app.run(terminal)
}

fn open_geoip(explicit: Option<&Path>) -> Option<Arc<GeoDb>> {
    let path = explicit.map(PathBuf::from).or_else(geoip::autodetect)?;
    match GeoDb::open(&path) {
        Ok(db) => {
            log::info!("geoip database loaded: {}", path.display());
            Some(Arc::new(db))
        }
        Err(e) => {
            log::warn!("geoip disabled ({}): {e}", path.display());
            None
        }
    }
}

fn probe_reference(path: &Path, format: LogFormat) -> Option<DateTime<FixedOffset>> {
    let source = FileSource::open(path).ok()?;
    loader::probe_last_timestamp(&source, format)
}

fn prompt_date_range<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    path: &Path,
    size: u64,
    reference: DateTime<FixedOffset>,
) -> Result<Option<(DateBound, DateBound)>> {
    use ratatui::layout::{Constraint, Direction, Layout};
    use ratatui::style::{Color, Style};
    use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

    let mut modal = DateRangeModal::default();
    let file_label = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("<file>")
        .to_string();
    let info = format!(
        "\n  File: {file_label}\n  Size: {:.1} MB — large files are loaded faster with a date range.\n  Last event: {}\n\n  ↑/↓ pick preset · Enter apply · Esc cancel · a load all",
        size as f64 / 1_048_576.0,
        reference.format("%Y-%m-%d %H:%M %:z")
    );
    loop {
        terminal.draw(|f| {
            let area = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(9)])
                .split(area);
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" lazylog — pick a date range ");
            let body = Paragraph::new(info.as_str())
                .wrap(Wrap { trim: false })
                .block(block);
            f.render_widget(body, chunks[0]);
            modal.render(f, chunks[1]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    continue;
                }
                match modal.mode {
                    DateRangeMode::Presets => match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => return Ok(None),
                        KeyCode::Char('a') => return Ok(Some((None, None))),
                        KeyCode::Up | KeyCode::Char('k') => modal.up(),
                        KeyCode::Down | KeyCode::Char('j') => modal.down(),
                        KeyCode::Enter => {
                            if modal.selected() == Preset::Custom {
                                modal.enter_custom();
                            } else {
                                let (from, to) = modal.selected().range(reference);
                                return Ok(Some((from, to)));
                            }
                        }
                        _ => {}
                    },
                    DateRangeMode::Custom => match key.code {
                        KeyCode::Esc => modal.exit_custom(),
                        KeyCode::Tab | KeyCode::BackTab | KeyCode::Up | KeyCode::Down => {
                            modal.toggle_field();
                        }
                        KeyCode::Enter => {
                            if let Some((from, to)) = modal.parse_custom() {
                                return Ok(Some((from, to)));
                            }
                        }
                        KeyCode::Backspace => modal.backspace(),
                        KeyCode::Char(c) => modal.push(c),
                        _ => {}
                    },
                }
            }
        }
    }
}

fn pick_file<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<Option<PathBuf>> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut state = PickerState::new(scanner::scan(&cwd));
    loop {
        terminal.draw(|f| state.render(f, f.area()))?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    continue;
                }
                match state.handle_key(key) {
                    PickerOutcome::Pending => {}
                    PickerOutcome::Cancelled => return Ok(None),
                    PickerOutcome::Selected(p) => return Ok(Some(p)),
                }
            }
        }
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn init_logger() {
    let dir = dirs_cache();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("lazylog.log");
    if let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let target = Box::new(file);
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .target(env_logger::Target::Pipe(target))
            .try_init();
    }
}

fn dirs_cache() -> PathBuf {
    if let Some(x) = std::env::var_os("XDG_CACHE_HOME") {
        return PathBuf::from(x).join("lazylog");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".cache").join("lazylog");
    }
    std::env::temp_dir().join("lazylog")
}
