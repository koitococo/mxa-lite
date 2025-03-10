use std::{
    fs::File,
    io::{Read, Write},
    process::Stdio,
};

use anyhow::Result;
use log::{error, info};
use tokio::process::Command;

/// Get the machine UUID from the DMI table.
pub(crate) fn get_machine_id() -> Result<String> {
    let mut fd = File::open("/sys/firmware/dmi/entries/1-0/raw")?;
    let mut buf: [u8; 24] = [0u8; 24];
    fd.read_exact(&mut buf)?;
    let buf2: [u8; 16] = buf[8..24].try_into()?;
    Ok(buf2.iter().map(|b| format!("{:02x}", b)).collect())
}

/// Download a file from the given URL and save it to the given path.
pub(crate) async fn download_file(url: &str, path: &str) -> Result<()> {
    info!("Downloading file from {} to {}", url, path);
    let client = reqwest::Client::new();
    let mut response = client.get(url).send().await?;
    if response.status().is_success() {
        let mut out = File::create(path)?;
        loop {
            let chunk = response.chunk().await?;
            if chunk.is_none() {
                return Ok(());
            }
            out.write_all(&chunk.unwrap())?;
        }
    } else {
        error!(
            "Failed to download file from {}. Server returned an error.",
            url
        );
        anyhow::bail!("Failed to download file from {}", url);
    }
}

/// Upload a file to the given URL.
pub(crate) async fn upload_file(url: &str, path: &str) -> Result<()> {
    info!("Uploading file from {} to {}", path, url);
    let client = reqwest::Client::new();
    let file = File::open(path)?;
    let req = client
        .put(url)
        .body(tokio::fs::File::from_std(file))
        .send()
        .await?;
    if req.status().is_success() {
        Ok(())
    } else {
        error!(
            "Failed to upload file to {}. Server returned an error.",
            url
        );
        anyhow::bail!("Failed to upload file to {}", url);
    }
}

/// Execute an external command and return its output.
pub(crate) async fn execute_command_with_output<'a>(
    cmd: &String,
    args: Vec<String>,
) -> Result<(i32, String, String)> {
    info!("Executing external command: {} {:?}", cmd, args);
    let child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output().await?;
    Ok((
        output.status.code().unwrap_or(-1),
        String::from_utf8(output.stdout)?,
        String::from_utf8(output.stderr)?,
    ))
}

pub(crate) async fn execute_shell_with_output<'a>(cmd: &String) -> Result<(i32, String, String)> {
    execute_command_with_output(&("sh".to_string()), vec!["-c".to_string(), cmd.to_string()]).await
}
