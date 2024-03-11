use core::fmt;
use std::{
    error::Error,
    path::Path,
    process::{Output, Stdio},
};

use anyhow::{bail, Context};
use tokio::{
    fs,
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::Command,
};

const PROVER_PATH: &str = "./stone-prover";

#[derive(Debug)]
struct StoneRunner(String);
impl Error for StoneRunner {}
impl fmt::Display for StoneRunner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

async fn run(mut command: Command, input: Option<String>) -> anyhow::Result<String> {
    command.current_dir(Path::new(PROVER_PATH));
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = command.spawn()?;

    if let Some(input) = input {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input.as_bytes()).await?;
        }
    }

    println!("hererereer");

    let stdout = child.stdout.take().context("failed to open stdout")?;
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        println!("{}", line);
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
    } else {
        if let Some(mut output) = child.stdout.take() {
            let mut out = Vec::new();
            output.read_to_end(&mut out).await?;

            // Handle error output
            let out = String::from_utf8(out)?;
            return Ok(out);
        }
    }
    Ok(String::new())
}

pub async fn rebuild() -> anyhow::Result<()> {
    let mut command = Command::new("podman");
    command
        .arg("build")
        .arg("-t")
        .arg("fibonacci-prover")
        .arg("-f")
        .arg("prover.dockerfile")
        .arg(".");

    match run(command, None).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub async fn prove() -> anyhow::Result<String> {
    let filename = Path::new(PROVER_PATH).join("program_input.json");
    let file_content = fs::read_to_string(filename).await?;

    let mut command = Command::new("podman");
    command
        .arg("run")
        .arg("-i")
        .arg("--rm")
        .arg("fibonacci-prover");

    println!("Running verification");

    run(command, Some(file_content)).await
}

pub async fn verify(proof: String) -> anyhow::Result<()> {
    Ok(())
}

#[tokio::test]
async fn test_build() {
    rebuild().await.unwrap();
    println!("Build successful");
    let proof = prove().await.unwrap();
    println!("{}", proof);
    // verify(proof).await.unwrap();
}
