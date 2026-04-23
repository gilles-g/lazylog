use std::path::PathBuf;

use lazylog::log::detect::detect_from_path;
use lazylog::log::format::LogFormat;

fn td(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testdata")
        .join(name)
}

#[test]
fn detects_symfony() {
    assert_eq!(
        detect_from_path(&td("symfony.log")),
        LogFormat::SymfonyMonolog
    );
}

#[test]
fn detects_nginx_access() {
    assert_eq!(
        detect_from_path(&td("nginx-access.log")),
        LogFormat::NginxAccess
    );
}

#[test]
fn detects_nginx_error() {
    assert_eq!(
        detect_from_path(&td("nginx-error.log")),
        LogFormat::NginxError
    );
}

#[test]
fn detects_apache_access() {
    assert_eq!(
        detect_from_path(&td("apache-access.log")),
        LogFormat::ApacheAccess
    );
}

#[test]
fn detects_apache_error() {
    assert_eq!(
        detect_from_path(&td("apache-error.log")),
        LogFormat::ApacheError
    );
}

#[test]
fn detects_php() {
    assert_eq!(detect_from_path(&td("php-error.log")), LogFormat::PhpError);
}
