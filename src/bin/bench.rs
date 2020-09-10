pub struct Builder {
    files: Vec<(PathBuf, PathBuf)>,
}

impl Bench {
    fn run(self) -> Result<()> {
        let cwd = env::current_dir()?;
        let results = cwd.join("results");
        let status = process::Command::new("docker")
            .arg("run")
            .arg("-v")
            .arg(format!("{}:/target", results.display()))
            .args(self.extra_docker_args)
            .arg("-it")
            .arg("zsh-plugin-manager-benchmark")
            .args(self.cmd)
            .stdout(process::Stdio::inherit())
            .stderr(process::Stdio::inherit())
            .status()?;
        if !status.success() {
            bail!("command did not execute successfully")
        }
        Ok(())
    }
}
