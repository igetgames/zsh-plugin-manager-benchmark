mod manage;

use std::str;

use anyhow::{bail, Error, Result};
use structopt::clap::AppSettings;
use structopt::StructOpt;

use crate::manage::*;

#[derive(Debug)]
enum Kind {
    Antibody,
    Antigen,
    Sheldon,
    Zgen,
    Zinit,
    Zplug,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Benchmark the 'install' step.
    Install,
    /// Benchmark the 'load' step.
    Load,
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
    #[structopt(long, short, name = "KIND")]
    kind: Option<Kind>,

    #[structopt(subcommand)]
    command: Command,
}

impl str::FromStr for Kind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let kind = match s {
            "antibody" => Self::Antibody,
            "antigen" => Self::Antigen,
            "sheldon" => Self::Sheldon,
            "zgen" => Self::Zgen,
            "zinit" => Self::Zinit,
            "zplug" => Self::Zplug,
            _ => bail!("`{}` is not a valid kind", s),
        };
        Ok(kind)
    }
}

fn main() {
    let opt = Opt::from_args();

    let managers: Vec<dyn Manager> = vec![];

    match opt.command {}

    println!("{:?}", opt);
}
