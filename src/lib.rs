use std::{path::Path, process::Stdio};

use tokio::{io::AsyncReadExt, process::Command};

const PROVER_PATH: &str = "./stone-prover";

pub async fn rebuild() -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::new("podman");
    command
        .arg("build")
        .arg("-t")
        .arg("fibonacci-prover")
        .arg("-f")
        .arg("prover.dockerfile")
        .arg(".")
        .current_dir(Path::new(PROVER_PATH));

    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = command.spawn()?;
    let status = child.wait().await?;

    if !status.success() {
        if let Some(mut output) = child.stderr.take() {
            let mut err = Vec::new();
            output.read_to_end(&mut err).await?;

            // Handle error output
            eprintln!("Error executing command: {}", String::from_utf8(err)?);
        }
    } else {
        if let Some(mut output) = child.stdout.take() {
            let mut err = Vec::new();
            output.read_to_end(&mut err).await?;

            // Handle error output
            println!("Command output: {}", String::from_utf8(err)?);
        }
        println!("Command executed successfully!");
    }

    Ok(())
}

#[tokio::test]
async fn test_build() {
    rebuild().await.unwrap();
}
