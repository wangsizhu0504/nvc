use super::command::Command;
use crate::config::NvcConfig;
use crate::doctor::{build_report, CheckStatus};
use clap::Parser;

#[derive(Parser, Debug, Default)]
pub struct Doctor {
    /// Print JSON instead of human-readable output
    #[clap(long)]
    json: bool,
}

fn icon_for(status: CheckStatus) -> &'static str {
    match status {
        CheckStatus::Ok => "ok",
        CheckStatus::Warn => "warn",
        CheckStatus::Error => "error",
    }
}

impl Command for Doctor {
    type Error = serde_json::Error;

    fn apply(self, config: &NvcConfig) -> Result<(), Self::Error> {
        let report = build_report(config);

        if self.json {
            println!("{}", serde_json::to_string_pretty(&report)?);
            return Ok(());
        }

        println!("nvc doctor: {}", icon_for(report.status));
        for check in report.checks {
            println!(
                "[{}] {}: {}",
                icon_for(check.status),
                check.name,
                check.detail
            );
        }

        Ok(())
    }
}
