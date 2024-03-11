use std::{path::Path, process::Stdio};

use anyhow::{bail, Context};
use tokio::{
    fs,
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::Command,
};

const PROVER_PATH: &str = "./stone-prover";

async fn run(mut command: Command, input: Option<String>) -> anyhow::Result<String> {
    command
        .current_dir(Path::new(PROVER_PATH))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped());

    let mut child = command.spawn()?;

    if let Some(input) = input {
        let mut stdin = child.stdin.take().context("failed to open stdin")?;

        tokio::spawn(async move {
            stdin.write_all(input.as_bytes()).await.unwrap();
        });
    }

    let stdout = child.stdout.take().context("failed to open stdout")?;
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();
    let mut out = String::new();
    while let Some(line) = lines.next_line().await? {
        println!("{}", line);
        out.push_str(&line);
    }

    let status = child.wait().await?;

    if !status.success() {
        if let Some(mut output) = child.stderr.take() {
            let mut err = Vec::new();
            output.read_to_end(&mut err).await?;

            // Handle error output
            let err = String::from_utf8(err).context("failed to parse stderr")?;
            bail!("Podman error: {}", err)
        };
        bail!("Error without stderr")
    }

    Ok(out)
}

impl Prover {
    pub async fn pull(&self, prover: &str, verifier: &str) -> anyhow::Result<()> {
        // podman pull neotheprogramist/verifier:latest
        let mut command = Command::new("podman");
        command.arg("pull").arg(format!("docker.io/{}", prover));

        run(command, None).await.context("Failed to pull prover")?;

        let mut command = Command::new("podman");
        command.arg("pull").arg(format!("docker.io/{}", verifier));

        run(command, None)
            .await
            .context("Failed to pull verifier")?;

        Ok(())
    }

    pub async fn rebuild(&self) -> anyhow::Result<()> {
        let mut rebuild_prover = Command::new("podman");
        rebuild_prover
            .arg("build")
            .arg("-t")
            .arg(&self.0)
            .arg("-f")
            .arg("prover.dockerfile")
            .arg(".");

        run(rebuild_prover, None)
            .await
            .context("Failed to rebuild prover")?;

        let mut rebuild_verifier = Command::new("podman");
        rebuild_verifier
            .arg("build")
            .arg("-t")
            .arg("verifier")
            .arg("-f")
            .arg("verifier.dockerfile")
            .arg(".");

        run(rebuild_verifier, None)
            .await
            .context("Failed to rebuild verifier")?;

        Ok(())
    }

    pub async fn prove(&self) -> anyhow::Result<String> {
        let filename = Path::new(PROVER_PATH).join("program_input.json");
        let file_content = fs::read_to_string(filename).await?;

        let mut command = Command::new("podman");
        command.arg("run").arg("-i").arg("--rm").arg(&self.0);

        run(command, Some(file_content)).await
    }

    pub async fn verify(proof: String) -> anyhow::Result<()> {
        let mut command = Command::new("podman");
        command.arg("run").arg("-i").arg("--rm").arg("verifier");

        run(command, Some(proof)).await?;

        Ok(())
    }
}

pub struct Prover(String);

#[tokio::test]
async fn test_build() {
    // prepare
    let prover = Prover("fibonacci-prover".to_string());
    prover
        .pull(
            "neotheprogramist/fibonacci-prover:latest",
            "neotheprogramist/verifier",
        )
        .await
        .unwrap();

    // proof and verify
    let proof = prover.prove().await.unwrap();
    Prover::verify(proof).await.unwrap();
}
