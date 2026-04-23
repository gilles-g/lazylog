use std::time::Instant;

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime};
use lazylog::log::detect::detect_from_path;
use lazylog::log::loader::{self, DateFilter, LoadMsg};

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .expect("usage: bench_load <file> [--from D] [--to D]");
    let path = std::path::PathBuf::from(path);
    let mut filter = DateFilter::default();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--from" => filter.from = Some(parse_dt(&args.next().unwrap())),
            "--to" => filter.to = Some(parse_dt(&args.next().unwrap())),
            _ => panic!("unknown arg: {arg}"),
        }
    }

    let format = detect_from_path(&path);
    println!("detected format: {format}");
    if !filter.is_empty() {
        println!("filter: from={:?} to={:?}", filter.from, filter.to);
    }

    let start = Instant::now();
    let handle = loader::load(&path, format, filter, None);
    let mut kept: u32 = 0;
    let mut source_bytes: usize = 0;
    let mut first_line: Option<u32> = None;
    let mut last_line: Option<u32> = None;
    let total: u32 = loop {
        match handle.rx.recv() {
            Ok(LoadMsg::Source(src)) => source_bytes = src.len(),
            Ok(LoadMsg::Chunk(chunk)) => {
                if first_line.is_none() {
                    first_line = chunk.first().map(|e| e.line_no);
                }
                if let Some(last) = chunk.last() {
                    last_line = Some(last.line_no);
                }
                kept += chunk.len() as u32;
            }
            Ok(LoadMsg::Done { total, .. }) => break total,
            Ok(LoadMsg::Error(e)) => {
                eprintln!("error: {e}");
                return;
            }
            Err(_) => return,
        }
    };
    let dt = start.elapsed();
    println!("scanned: {total} lines, kept: {kept} events in {dt:?}");
    println!(
        "first parsed line: {:?}, last parsed line: {:?} (reverse order = first > last)",
        first_line, last_line
    );
    println!(
        "throughput: {:.1} MB/s",
        source_bytes as f64 / 1_048_576.0 / dt.as_secs_f64()
    );
}

fn parse_dt(s: &str) -> DateTime<FixedOffset> {
    if let Ok(d) = DateTime::parse_from_rfc3339(s) {
        return d;
    }
    let tz = FixedOffset::east_opt(0).unwrap();
    if let Ok(n) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return DateTime::from_naive_utc_and_offset(n, tz);
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return DateTime::from_naive_utc_and_offset(d.and_hms_opt(0, 0, 0).unwrap(), tz);
    }
    panic!("invalid date: {s}");
}
