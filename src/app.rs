use std::collections::HashMap;
use std::env;
use std::ffi;
use std::fs;
use std::path::Path;
use std::process;

use anyhow::{bail, Context, Result};

use crate::Kind;

const TEMPLATES: include_dir::Dir = include_dir::include_dir!("src/templates");

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

/// What to benchmark.
#[derive(Debug)]
pub enum Bench {
    /// Benchmark the 'install' step.
    Install,
    /// Benchmark the 'load' step.
    Load,
    /// Benchmark the 'load' step with DEFER=true.
    Defer,
}

/// Loads included files in the `templates/` folder for the given kind.
fn load_files(kind: &Kind) -> Result<HashMap<&'static str, &'static str>> {
    let mut map = HashMap::new();
    let kind = kind.to_string();
    for entry in TEMPLATES.find(&format!("{}/*", kind))? {
        match entry {
            include_dir::DirEntry::File(file) => {
                map.insert(
                    file.path()
                        .strip_prefix(&kind)?
                        .to_str()
                        .context("file name not UTF-8")?,
                    file.contents_utf8().context("contents not UTF-8")?,
                );
            }
            include_dir::DirEntry::Dir(_) => {}
        }
    }
    Ok(map)
}

/// Builds the docker container.
fn docker_build() -> Result<()> {
    let status = process::Command::new("docker")
        .args(&[
            "build",
            "--tag",
            "zsh-plugin-manager-benchmark",
            env!("CARGO_MANIFEST_DIR"),
        ])
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .status()?;
    if !status.success() {
        bail!("command did not execute successfully")
    }
    Ok(())
}

/// Runs a command in the docker container.
fn docker_run<I1, S1, I2, S2>(args: I1, cmd: I2) -> Result<()>
where
    I1: IntoIterator<Item = S1>,
    S1: AsRef<ffi::OsStr>,
    I2: IntoIterator<Item = S2>,
    S2: AsRef<ffi::OsStr>,
{
    let status = process::Command::new("docker")
        .arg("run")
        .args(&[
            "-v",
            concat!(env!("CARGO_MANIFEST_DIR"), "/results:/results"),
        ])
        .args(args)
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

/// Renders templates to `rendered/` and generates Docker args to mount them.
fn docker_args(kind: &Kind) -> Result<Vec<String>> {
    let mut ctx = tera::Context::new();
    ctx.insert("plugins", PLUGINS);

    let rendered: HashMap<_, _> = load_files(&kind)
        .context("failed to load files")?
        .into_iter()
        .map::<Result<(_, _)>, _>(|(name, contents)| {
            Ok((
                name,
                tera::Tera::one_off(contents, &ctx, false)
                    .with_context(|| format!("failed to render `{}`", name))?,
            ))
        })
        .collect::<Result<_, _>>()?;

    let mut args = Vec::new();

    let render_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("rendered")
        .join(kind.to_string());

    fs::remove_dir_all(&render_dir).ok();
    for (path, contents) in rendered {
        let file_path = render_dir.join(path);
        log::info!("rendered `{}`", file_path.display());
        fs::create_dir_all(file_path.parent().unwrap())?;
        fs::write(&file_path, contents)?;
        let dest = Path::new("/root").join(&format!(".{}", path));

        args.push("-v".to_string());
        args.push(format!("{}:{}", file_path.display(), dest.display()));
    }

    Ok(args)
}

/// Returns the hyperfine 'prepare' command for the given kind.
fn prepare_cmd(kind: &Kind) -> &'static str {
    match kind {
        Kind::Antibody => "rm -rf /root/.cache/antibody",
        Kind::Antigen => "rm -rf /root/.antigen",
        Kind::Sheldon => "find /root/.sheldon -mindepth 1 -maxdepth 1 ! -name \"plugins*.toml\" -exec rm -rf {} \\;",
        Kind::Zgen => "git -C /root/.zgen clean -dffx",
        Kind::Zinit => "find /root/.zinit -mindepth 1 -maxdepth 1 ! -name \"bin\" -exec rm -rf {} \\;",
        Kind::Zplug => "rm -rf /root/.zplug/repos",
    }
}

/// Returns the bench command.
fn bench_cmd(bench: &Bench, kind: &Kind) -> Vec<String> {
    macro_rules! vec_into {
        ($($i:expr),*) => (vec![$($i.into()),*]);
    }
    match bench {
        Bench::Install => vec_into![
            "hyperfine",
            "--prepare",
            prepare_cmd(&kind),
            "--warmup",
            "3",
            "--export-json",
            format!("/results/install-{}.json", kind),
            "zsh -ic exit"
        ],
        Bench::Load => vec_into![
            "hyperfine",
            "--warmup",
            "3",
            "--export-json",
            format!("/results/load-{}.json", kind),
            "zsh -ic exit"
        ],
        Bench::Defer => vec_into![
            "hyperfine",
            "--warmup",
            "3",
            "--export-json",
            format!("/results/load-{}.json", kind),
            "DEFER=true zsh -ic exit"
        ],
    }
}

/// Returns the version command.
fn version_cmd(kind: &Kind) -> &[&'static str] {
    match kind {
        Kind::Antibody => &["antibody", "--version"],
        Kind::Antigen => &["zsh", "-c", "source /root/antigen.zsh && antigen-version"],
        Kind::Sheldon => &["sheldon", "--version"],
        Kind::Zgen => &["git", "-C", "/root/.zgen", "rev-parse", "--short", "HEAD"],
        Kind::Zinit => &[
            "git",
            "-C",
            "/root/.zinit/bin",
            "rev-parse",
            "--short",
            "HEAD",
        ],
        Kind::Zplug => &["git", "-C", "/root/.zplug", "rev-parse", "--short", "HEAD"],
    }
}

pub fn bench(bench: Bench, kind: Kind) -> Result<()> {
    let args = docker_args(&kind)?;
    let cmd = bench_cmd(&bench, &kind);
    log::info!("building docker container");
    docker_build().context("failed to run docker build command")?;
    log::info!("running benchmark");
    docker_run(&args, &cmd).context("failed to run docker run command")?;
    Ok(())
}

pub fn run(kind: Kind) -> Result<()> {
    let args = docker_args(&kind)?;
    log::info!("building docker container");
    docker_build().context("failed to run docker build command")?;
    log::info!("opening a shell");
    docker_run(&args, &["zsh"]).context("failed to run docker command")?;
    Ok(())
}

pub fn version(kind: Kind) -> Result<()> {
    let cmd = version_cmd(&kind);
    log::info!("building docker container");
    docker_build().context("failed to run docker build command")?;
    log::info!("fetch version");
    docker_run::<_, &String, _, _>(&[], &*cmd).context("failed to run docker command")?;
    Ok(())
}
