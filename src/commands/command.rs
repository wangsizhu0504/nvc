use crate::config::NvcConfig;
use crate::outln;
use colored::Colorize;

pub trait Command: Sized {
    type Error: std::error::Error;
    fn apply(self, config: &NvcConfig) -> Result<(), Self::Error>;

    fn handle_error(err: Self::Error, config: &NvcConfig) {
        let err_s = format!("{err}");
        outln!(config, Error, "{} {}", "error:".red().bold(), err_s.red());
        let mut source = std::error::Error::source(&err);
        while let Some(err) = source {
            outln!(config, Error, "  caused by: {}", err);
            source = err.source();
        }
        std::process::exit(1);
    }

    fn call(self, config: NvcConfig) {
        match self.apply(&config) {
            Ok(()) => (),
            Err(err) => Self::handle_error(err, &config),
        }
    }
}
