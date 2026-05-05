// src-tauri/src/download_manager.rs
use anyhow::{bail, Result};
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

pub async fn download_file(url: &str, destination: &Path) -> Result<()> {
    let response = reqwest::get(url).await?;
    if !response.status().is_success() {
        bail!("Failed to download file: {}", response.status());
    }

    let mut file = File::create(destination)?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(chunk.as_ref())?;
    }

    Ok(())
}

pub fn verify_sha256(path: &Path, expected: &str) -> Result<()> {
    let expected = expected.trim();
    if expected.is_empty() || expected.eq_ignore_ascii_case("replace-with-real-sha256") {
        bail!("Model checksum is not configured for {}", path.display());
    }

    let actual = sha256_file(path)?;
    if actual != expected.to_ascii_lowercase() {
        bail!(
            "Checksum mismatch for {}. expected {}, got {}",
            path.display(),
            expected,
            actual
        );
    }

    Ok(())
}
