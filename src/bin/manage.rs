use std::env;
use std::ffi;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use anyhow::{bail, Result};

const PLUGINS: &[&str] = &[
    "zsh-users/zsh-autosuggestions",
    "wting/autojump",
    "zsh-users/zsh-syntax-highlighting",
    "StackExchange/blackbox",
    "sobolevn/git-secret",
    "b4b4r07/enhancd",
    "fcambus/ansiweather",
    "chriskempson/base16-shell",
    "supercrabtree/k",
    "zsh-users/zsh-history-substring-search",
    "wfxr/forgit",
    "zdharma/fast-syntax-highlighting",
    "iam4x/zsh-iterm-touchbar",
    "unixorn/git-extra-commands",
    "MichaelAquilina/zsh-you-should-use",
    "mfaerevaag/wd",
    "zsh-users/zaw",
    "Tarrasch/zsh-autoenv",
    "mafredri/zsh-async",
    "djui/alias-tips",
    "agkozak/zsh-z",
    "changyuheng/fz",
    "b4b4r07/emoji-cli",
    "Tarrasch/zsh-bd",
    "Vifon/deer",
    "zdharma/history-search-multi-word",
];

/////////////////////////////////////////////////////////////////////////
// Base traits and functions
/////////////////////////////////////////////////////////////////////////

pub trait Manager {
    fn name() -> &'static str;

    fn extra_docker_args() -> Vec<String> {
        vec![]
    }

    fn docker_run(cmd: Vec<String>) -> Result<()> {
        let cwd = env::current_dir()?;
        let results = cwd.join("results");
        let status = process::Command::new("docker")
            .arg("run")
            .arg("-v")
            .arg(format!("{}:/target", results.display()))
            .args(Self::extra_docker_args())
            .arg("-it")
            .arg("zsh-plugin-manager-benchmark")
            .args(cmd)
            .stdout(process::Stdio::inherit())
            .stderr(process::Stdio::inherit())
            .status()?;
        if !status.success() {
            bail!("command did not execute successfully")
        }
        Ok(())
    }
}

pub trait Install: Manager {
    fn prepare() -> String;

    fn install() -> Result<()> {
        docker_run()
    }
}

pub trait Load: Manager {
    fn load() -> Result<()>;
}

pub trait Defer: Manager {
    fn defer() -> Result<()>;
}

/////////////////////////////////////////////////////////////////////////
// Antibody
/////////////////////////////////////////////////////////////////////////

pub struct Antibody;

impl Install for Antibody {
    fn render(bench: &mut Bench) {
        bench.file(
            "~/.zshrc",
            "\
#!/usr/bin/env zsh
source <(antibody init)
antibody bundle < ~/.antibody/plugins.txt
        ",
        );
        bench.file("~/.antibody/plugins.txt", PLUGINS.join("\n"));
    }

    fn install(bench: &mut Bench) -> Result<()> {
        bench.hyperfine()
    }
}
impl Install for Antibody {}
