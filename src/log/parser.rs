use super::event::LogEvent;
use super::format::LogFormat;
use super::parsers;

pub trait LogParser: Send + Sync {
    fn parse(&self, line_no: u32, offset: u64, len: u32, text: &str) -> Option<LogEvent>;
}

pub fn parser_for(format: LogFormat) -> Box<dyn LogParser> {
    match format {
        LogFormat::SymfonyMonolog => Box::new(parsers::symfony::SymfonyParser),
        LogFormat::PhpError => Box::new(parsers::php::PhpErrorParser),
        LogFormat::NginxAccess => Box::new(parsers::nginx_access::NginxAccessParser),
        LogFormat::NginxError => Box::new(parsers::nginx_error::NginxErrorParser),
        LogFormat::ApacheAccess => Box::new(parsers::apache_access::ApacheAccessParser),
        LogFormat::ApacheError => Box::new(parsers::apache_error::ApacheErrorParser),
        LogFormat::Generic => Box::new(parsers::generic::GenericParser),
    }
}
