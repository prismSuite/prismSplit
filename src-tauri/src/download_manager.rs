// src-tauri/src/download_manager.rs
use anyhow::{bail, Result};
use futures_util::StreamExt;
use md5::{Digest as _, Md5};
use sha2::{Digest as _, Sha256};
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

pub fn md5_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Md5::new();
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
    download_file_with_progress(url, destination, |_, _| {}).await
}

pub async fn download_file_with_progress<F>(
    url: &str,
    destination: &Path,
    mut on_progress: F,
) -> Result<()>
where
    F: FnMut(u64, u64) + Send + 'static,
{
    let response = reqwest::get(url).await?;
    if !response.status().is_success() {
        bail!("Failed to download file: {}", response.status());
    }

    let total_size = response
        .content_length()
        .ok_or_else(|| anyhow::anyhow!("Failed to get content length"))?;

    let mut file = File::create(destination)?;
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(chunk.as_ref())?;
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total_size);
    }

    Ok(())
}

pub fn verify_sha256(path: &Path, expected: &str) -> Result<()> {
    let expected = expected.trim();
    if expected.is_empty() || expected.eq_ignore_ascii_case("replace-with-real-sha256") {
        // Skip verification for models without a known hash
        return Ok(());
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
