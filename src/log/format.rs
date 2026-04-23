use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogFormat {
    SymfonyMonolog,
    PhpError,
    NginxAccess,
    NginxError,
    ApacheAccess,
    ApacheError,
    Generic,
}

impl LogFormat {
    pub fn label(&self) -> &'static str {
        match self {
            LogFormat::SymfonyMonolog => "Symfony (Monolog)",
            LogFormat::PhpError => "PHP error",
            LogFormat::NginxAccess => "Nginx access",
            LogFormat::NginxError => "Nginx error",
            LogFormat::ApacheAccess => "Apache access",
            LogFormat::ApacheError => "Apache error",
            LogFormat::Generic => "Generic",
        }
    }

    pub fn all() -> &'static [LogFormat] {
        &[
            LogFormat::SymfonyMonolog,
            LogFormat::PhpError,
            LogFormat::NginxAccess,
            LogFormat::NginxError,
            LogFormat::ApacheAccess,
            LogFormat::ApacheError,
            LogFormat::Generic,
        ]
    }
}

impl fmt::Display for LogFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl FromStr for LogFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "symfony" | "monolog" | "symfony-monolog" => Ok(LogFormat::SymfonyMonolog),
            "php" | "php-error" => Ok(LogFormat::PhpError),
            "nginx-access" => Ok(LogFormat::NginxAccess),
            "nginx-error" => Ok(LogFormat::NginxError),
            "apache-access" => Ok(LogFormat::ApacheAccess),
            "apache-error" => Ok(LogFormat::ApacheError),
            "generic" => Ok(LogFormat::Generic),
            other => Err(format!("unknown format: {other}")),
        }
    }
}
