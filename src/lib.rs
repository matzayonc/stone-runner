use core::fmt;
use std::{
    error::Error,
    path::Path,
    process::{Output, Stdio},
};

use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
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

async fn run(
    mut command: Command,
    stdin: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    command.current_dir(Path::new(PROVER_PATH));
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = command.spawn()?;

    if let Some(input) = stdin {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input.as_bytes()).await?;
        }
    }

    let status = child.wait().await?;

    if !status.success() {
        let err = if let Some(mut output) = child.stderr.take() {
            let mut err = Vec::new();
            output.read_to_end(&mut err).await?;

            // Handle error output
            String::from_utf8(err)?
        } else {
            String::from("Error without stderr")
        };

        Err(Box::new(StoneRunner(err)))
    } else {
        if let Some(mut output) = child.stdout.take() {
            let mut out = Vec::new();
            output.read_to_end(&mut out).await?;

            // Handle error output
            let out = String::from_utf8(out)?;
            Ok(out)
        } else {
            Ok(String::new())
        }
    }
}

pub async fn rebuild() -> Result<(), Box<dyn std::error::Error>> {
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

pub async fn prove() -> Result<(), Box<dyn std::error::Error>> {
    let filename = Path::new(PROVER_PATH).join("program_input.json");
    let file_content = fs::read_to_string(filename).await?;

    let mut command = Command::new("podman");
    command
        .arg("build")
        .arg("-t")
        .arg("fibonacci-prover")
        .arg("-f")
        .arg("prover.dockerfile")
        .arg(".");

    match run(command, Some(file_content)).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[tokio::test]
async fn test_build() {
    rebuild().await.unwrap();
    prove().await.unwrap();
}
