use std::collections::HashMap;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Context, Result};
use maxminddb::{geoip2, Reader};

/// A GeoIP database backed by a `.mmdb` file (MaxMind GeoLite2 or DB-IP lite).
/// Resolves IPs to a country label. Each resolved IP is cached in-memory so
/// repeated lookups during filtering / facet computation are essentially free.
pub struct GeoDb {
    reader: Reader<Vec<u8>>,
    cache: Mutex<HashMap<IpAddr, Option<String>>>,
}

impl GeoDb {
    pub fn open(path: &Path) -> Result<Self> {
        let reader = Reader::open_readfile(path)
            .with_context(|| format!("open geoip db: {}", path.display()))?;
        Ok(Self {
            reader,
            cache: Mutex::new(HashMap::new()),
        })
    }

    /// Returns a human-readable country label (English name, falls back to
    /// the ISO 3166-1 alpha-2 code). `None` when the IP is not in the DB, is
    /// private, or is malformed.
    pub fn country_label(&self, ip_str: &str) -> Option<String> {
        let ip: IpAddr = ip_str.parse().ok()?;
        if let Ok(cache) = self.cache.lock() {
            if let Some(hit) = cache.get(&ip) {
                return hit.clone();
            }
        }
        let label = self.resolve(ip);
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(ip, label.clone());
        }
        label
    }

    fn resolve(&self, ip: IpAddr) -> Option<String> {
        let result = self.reader.lookup(ip).ok()?;
        let country: Option<geoip2::Country> = result.decode().ok().flatten();
        country.and_then(|c| {
            c.country
                .names
                .english
                .map(|s| s.to_string())
                .or_else(|| c.country.iso_code.map(|s| s.to_string()))
        })
    }
}

/// Tries to locate a `.mmdb` file from common places, in order:
///   1. `$LAZYLOG_GEOIP`
///   2. `$XDG_DATA_HOME/lazylog/geoip.mmdb`
///   3. `~/.local/share/lazylog/geoip.mmdb`
///   4. `~/.lazylog/geoip.mmdb`
pub fn autodetect() -> Option<PathBuf> {
    if let Some(v) = std::env::var_os("LAZYLOG_GEOIP") {
        let p = PathBuf::from(v);
        if p.is_file() {
            return Some(p);
        }
    }
    let candidates = default_paths();
    candidates.into_iter().find(|p| p.is_file())
}

fn default_paths() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
        out.push(PathBuf::from(xdg).join("lazylog").join("geoip.mmdb"));
    }
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        out.push(
            home.join(".local")
                .join("share")
                .join("lazylog")
                .join("geoip.mmdb"),
        );
        out.push(home.join(".lazylog").join("geoip.mmdb"));
    }
    out
}
