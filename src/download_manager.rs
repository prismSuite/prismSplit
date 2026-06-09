// src-tauri/src/download_manager.rs
use anyhow::{bail, Result};
use futures_util::StreamExt;
use sha2::Digest;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationResult {
    Verified,
    SkippedPlaceholder,
    Failed { expected: String, actual: String },
}

fn hash_file<D: Digest>(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = D::new();
    let mut buffer = [0u8; 8192];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    let hash_result = hasher.finalize();
    Ok(hash_result.iter().map(|b| format!("{:02x}", b)).collect())
}

pub fn sha256_file(path: &Path) -> Result<String> {
    hash_file::<sha2::Sha256>(path)
}

pub fn md5_file(path: &Path) -> Result<String> {
    hash_file::<md5::Md5>(path)
}

pub async fn download_file_with_progress<F>(
    url: &str,
    destination: &Path,
    mut on_progress: F,
) -> Result<()>
where
    F: FnMut(u64, u64) + Send + 'static,
{
    let client = reqwest::Client::builder()
        .user_agent(format!("PrismSplit/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(300)) // 5 minute timeout for large models
        .http1_only()
        .build()?;

    let rand_suffix = uuid::Uuid::new_v4().to_string();
    let temp_name = format!(
        "{}.incomplete.{}",
        destination.file_name().unwrap_or_default().to_string_lossy(),
        rand_suffix
    );
    let temp_path = destination.with_file_name(temp_name);

    let mut attempts = 0;
    let max_attempts = 3;
    let mut delay = std::time::Duration::from_secs(2);
    let mut last_error = anyhow::anyhow!("Download failed");

    while attempts < max_attempts {
        attempts += 1;
        match download_attempt(&client, url, &temp_path, &mut on_progress).await {
            Ok(()) => {
                if let Err(e) = std::fs::rename(&temp_path, destination) {
                    let _ = std::fs::remove_file(&temp_path);
                    return Err(e.into());
                }
                return Ok(());
            }
            Err(e) => {
                last_error = e;
                if attempts < max_attempts {
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                }
            }
        }
    }

    let _ = std::fs::remove_file(&temp_path);
    Err(last_error)
}

async fn download_attempt<F>(
    client: &reqwest::Client,
    url: &str,
    temp_path: &Path,
    on_progress: &mut F,
) -> Result<()>
where
    F: FnMut(u64, u64) + Send,
{
    let mut file_size = 0;
    if temp_path.exists() {
        if let Ok(metadata) = std::fs::metadata(temp_path) {
            file_size = metadata.len();
        }
    }

    let mut request = client.get(url);
    if file_size > 0 {
        request = request.header("Range", format!("bytes={}-", file_size));
    }

    let response = request.send().await?;
    let status = response.status();

    let (mut file, downloaded, total_size) = if status == reqwest::StatusCode::PARTIAL_CONTENT {
        let file = std::fs::OpenOptions::new()
            .append(true)
            .open(temp_path)?;
        
        let total_size = if let Some(content_range) = response.headers().get("Content-Range") {
            if let Ok(content_range_str) = content_range.to_str() {
                content_range_str.split('/')
                    .next_back()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        };

        (file, file_size, total_size)
    } else if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
        let _ = std::fs::remove_file(temp_path);
        bail!("Range not satisfiable (resetting temp file): {}", status);
    } else if status.is_success() {
        let file = File::create(temp_path)?;
        let total_size = response.content_length().unwrap_or(0);
        (file, 0, total_size)
    } else {
        bail!("Failed to download file: {}", status);
    };

    let mut stream = response.bytes_stream();
    let mut downloaded = downloaded;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(chunk.as_ref())?;
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total_size);
    }

    Ok(())
}

pub fn verify_sha256(path: &Path, expected: &str) -> Result<VerificationResult> {
    let expected = expected.trim();
    if expected.is_empty() || expected.eq_ignore_ascii_case("replace-with-real-sha256") {
        return Ok(VerificationResult::SkippedPlaceholder);
    }

    let actual = sha256_file(path)?;
    if actual != expected.to_ascii_lowercase() {
        return Ok(VerificationResult::Failed {
            expected: expected.to_string(),
            actual,
        });
    }

    Ok(VerificationResult::Verified)
}
