mod app;

use std::str;

use anyhow::Result;
use parse_display::{Display, FromStr};
use structopt::clap::AppSettings;
use structopt::StructOpt;

use crate::app::Bench;

/// The kind of plugin manager.
#[derive(Debug, Display, FromStr)]
#[display(style = "snake_case")]
pub enum Kind {
    Antibody,
    Antigen,
    Sheldon,
    Zgen,
    Zinit,
    Zplug,
}

/// The subcommand.
#[derive(Debug, StructOpt)]
enum Command {
    /// Benchmark the 'install' step.
    Install,
    /// Benchmark the 'load' step.
    Load,
    /// Benchmark the 'load' step with deferred loading.
    Defer,
    /// Open 'zsh' with a particular plugin manager.
    Run,
    /// Output the versions of the plugin managers.
    Version,
}

/// Benchmark different Zsh plugin managers.
#[derive(Debug, StructOpt)]
#[structopt(
    setting = AppSettings::SubcommandRequired,
    setting = AppSettings::DisableHelpSubcommand,
)]
struct Opt {
    /// The kind of plugin manager to benchmark.
    #[structopt(long, short, global = true, name = "KIND")]
    kind: Option<Kind>,

    /// The subcommand.
    #[structopt(subcommand)]
    command: Command,
}

impl Kind {
    fn iter() -> Vec<Self> {
        vec![
            Self::Antibody,
            Self::Antigen,
            Self::Sheldon,
            Self::Zgen,
            Self::Zinit,
            Self::Zplug,
        ]
    }
}

impl Command {
    fn run(&self, kind: Kind) -> Result<()> {
        match self {
            Self::Install => crate::app::bench(Bench::Install, kind),
            Self::Load => crate::app::bench(Bench::Load, kind),
            Self::Defer => crate::app::bench(Bench::Defer, kind),
            Self::Run => crate::app::run(kind),
            Self::Version => crate::app::version(kind),
        }
    }
}

fn main() -> Result<()> {
    let Opt { command, kind } = Opt::from_args();
    femme::start();
    match kind {
        Some(kind) => {
            command.run(kind)?;
        }
        None => {
            for kind in Kind::iter() {
                log::info!("kind is {}", kind);
                command.run(kind)?;
            }
        }
    }
    Ok(())
}
