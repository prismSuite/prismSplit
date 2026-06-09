use crate::app_paths::AppPaths;
use crate::download_manager::{download_file_with_progress, md5_file, verify_sha256};
use crate::engine_bridge::{EngineBridge, EngineEvent};
use crate::job_manager;
use crate::model_registry::ModelRegistry;
use crate::models::{
    AppConfig, EngineHealth, ModelCatalogEntry, ProcessAudioResponse, SeparationRequest,
    SetupStatus,
};
use crate::runtime_manager::RuntimeManager;
use anyhow::{anyhow, bail, Result};
use futures_util::stream::{self, StreamExt};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug)]
pub struct Backend {
    runtime_manager: Arc<RuntimeManager>,
    model_registry: Arc<ModelRegistry>,
    config_path: PathBuf,
    pub active_processes: Arc<tokio::sync::Mutex<std::collections::HashMap<String, tokio::process::Child>>>,
}

impl Backend {
    pub fn new(app_root: PathBuf, resource_dir: PathBuf, config_path: PathBuf) -> Self {
        let config = load_config(&config_path);
        let mut paths = AppPaths::new(app_root, resource_dir);
        if let Some(custom_models) = config.models_dir.clone() {
            paths.models_dir = PathBuf::from(custom_models);
        }
        if let Some(custom_cache) = config.cache_dir.clone() {
            paths.cache_dir = PathBuf::from(custom_cache);
        }

        let runtime_manager = Arc::new(RuntimeManager::new(paths.clone()));
        let model_registry = Arc::new(ModelRegistry::new(
            paths.models_dir.clone(),
            paths.manifest_catalog_path(),
        ));
        let active_processes = Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));

        Self {
            runtime_manager,
            model_registry,
            config_path,
            active_processes,
        }
    }

    pub fn load_config(&self) -> AppConfig {
        load_config(&self.config_path)
    }

    pub fn update_config(&self, config: AppConfig) -> Result<()> {
        let old_config = self.load_config();
        let models_dir_changed = config.models_dir != old_config.models_dir;

        save_config(&self.config_path, &config)?;

        if models_dir_changed {
            if let Some(new_path) = &config.models_dir {
                self.model_registry.set_models_dir(PathBuf::from(new_path));
            }
        }

        Ok(())
    }

    pub async fn get_engine_health(&self) -> Result<EngineHealth> {
        self.runtime_manager.doctor().await
    }

    pub async fn prepare_engine(&self) -> Result<SetupStatus> {
        self.runtime_manager.prepare().await
    }

    pub async fn repair_engine(&self) -> Result<SetupStatus> {
        self.runtime_manager.smart_repair().await
    }

    pub async fn list_model_catalog(&self) -> Result<Vec<ModelCatalogEntry>> {
        let python_exe = self.runtime_manager.paths().venv_python_executable();
        let engine_script = self.runtime_manager.paths().installed_engine_script();

        if python_exe.is_file() && engine_script.is_file() {
            let payload = serde_json::json!({
                "catalog_path": self.runtime_manager.paths().manifest_catalog_path(),
                "models_dir": self
                    .model_registry
                    .models_dir
                    .read()
                    .map_err(|_| anyhow!("Failed to read models_dir"))?
                    .clone(),
            });
            let bridge = EngineBridge::new(python_exe, engine_script);
            let (events, mut child) = bridge.run_command_collect("list_models", payload).await?;
            let _ = child.wait().await;
            let terminal = events
                .last()
                .ok_or_else(|| anyhow!("Engine returned no events for list_models"))?;
            if terminal.event != "result" {
                return Err(anyhow!(
                    "{}",
                    terminal
                        .message
                        .clone()
                        .unwrap_or_else(|| "Engine failed to load models".into())
                ));
            }

            let payload = terminal
                .payload
                .clone()
                .ok_or_else(|| anyhow!("Engine did not return model payload"))?;
            let models = payload
                .get("models")
                .cloned()
                .ok_or_else(|| anyhow!("Engine list_models payload missing models"))?;
            return Ok(serde_json::from_value(models)?);
        }

        self.model_registry.load_catalog()
    }

    pub async fn download_model<F>(
        &self,
        model_id: String,
        mut on_progress: F,
    ) -> Result<ModelCatalogEntry>
    where
        F: FnMut(f32) + Send + 'static,
    {
        let entry = self.model_registry.get_entry(&model_id)?;
        self.model_registry.validate_downloadable(&entry)?;

        let destination = {
            let dir = self
                .model_registry
                .models_dir
                .read()
                .map_err(|_| anyhow!("Failed to read models_dir"))?;
            std::fs::create_dir_all(&*dir)?;
            dir.join(&entry.filename)
        };
        let temp_destination = destination.with_extension("download");

        let mut last_percent = -1.0_f32;
        let mut last_emit = std::time::Instant::now() - std::time::Duration::from_secs(1);

        if let Err(error) =
            download_file_with_progress(&entry.url, &temp_destination, move |downloaded, total| {
                if total > 0 {
                    let percent = (downloaded as f32 / total as f32) * 100.0;
                    let now = std::time::Instant::now();
                    if percent - last_percent >= 1.0 || now.duration_since(last_emit).as_millis() >= 500 {
                        on_progress(percent);
                        last_percent = percent;
                        last_emit = now;
                    }
                }
            })
            .await
        {
            let _ = std::fs::remove_file(&temp_destination);
            return Err(error);
        }

        match verify_sha256(&temp_destination, &entry.sha256)? {
            crate::download_manager::VerificationResult::Verified |
            crate::download_manager::VerificationResult::SkippedPlaceholder => {
                if let Err(e) = std::fs::rename(&temp_destination, &destination) {
                    let _ = std::fs::remove_file(&temp_destination);
                    return Err(e.into());
                }
            }
            crate::download_manager::VerificationResult::Failed { expected, actual } => {
                let _ = std::fs::remove_file(&temp_destination);
                bail!(
                    "Checksum mismatch for {}. expected {}, got {}",
                    entry.filename,
                    expected,
                    actual
                );
            }
        }

        Ok(entry)
    }

    pub async fn process_audio<F>(
        &self,
        file_path: String,
        model: String,
        output_dir: String,
        _quality: String,
        on_event: F,
    ) -> Result<ProcessAudioResponse>
    where
        F: FnMut(&EngineEvent) + Send + 'static,
    {
        let request = SeparationRequest {
            input_path: file_path.clone(),
            model_id: model.clone(),
            output_dir: output_dir.clone(),
            format: "wav".into(),
        };
        job_manager::validate_request(&request)?;

        let entry = self.model_registry.get_entry(&model)?;
        let model_path = if let Some(local_path) = &entry.local_path {
            PathBuf::from(local_path)
        } else {
            self.model_registry.installed_model_path(&entry)?
        };

        if !model_path.is_file() {
            return Err(anyhow!(
                "Model `{}` is not installed yet. Download it from the Model Registry tab first.",
                entry.name
            ));
        }

        let python_exe = self.runtime_manager.paths().venv_python_executable();
        let engine_script = self.runtime_manager.paths().installed_engine_script();
        if !python_exe.is_file() {
            return Err(anyhow!(
                "Engine runtime is not ready. Missing {}",
                python_exe.display()
            ));
        }
        if !engine_script.is_file() {
            return Err(anyhow!(
                "Engine script is not ready. Missing {}",
                engine_script.display()
            ));
        }

        std::fs::create_dir_all(&output_dir)?;

        let bridge = EngineBridge::new(python_exe, engine_script);
        let payload = serde_json::json!({
            "job_id": "job-local",
            "backend": entry.backend,
            "model_name": std::path::Path::new(&entry.filename)
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or(&entry.id),
            "input_path": file_path,
            "model_path": model_path,
            "output_dir": output_dir,
        });

        let job_id = "job-local".to_string();
        let mut child = bridge.spawn_command("separate", payload).await?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to capture engine stdout"))?;

        {
            let mut procs = self.active_processes.lock().await;
            procs.insert(job_id.clone(), child);
        }

        let stream_res = EngineBridge::stream_stdout(stdout, on_event).await;

        let wait_result = {
            // Block inside this scope to keep lock short, but we wait asynchronously
            let mut procs = self.active_processes.lock().await;
            if let Some(mut active_child) = procs.remove(&job_id) {
                active_child.wait().await
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Process missing from active registry",
                ))
            }
        };
        let _ = wait_result;

        let terminal = stream_res?;

        match terminal.event.as_str() {
            "result" => {
                let payload = terminal
                    .payload
                    .clone()
                    .ok_or_else(|| anyhow!("Engine result did not include payload"))?;
                let vocals_path = payload
                    .get("vocals_path")
                    .and_then(|value| value.as_str())
                    .ok_or_else(|| anyhow!("Engine result missing vocals_path"))?;
                let instrumental_path = payload
                    .get("instrumental_path")
                    .and_then(|value| value.as_str())
                    .ok_or_else(|| anyhow!("Engine result missing instrumental_path"))?;

                Ok(ProcessAudioResponse {
                    job_id: terminal.job_id.unwrap_or_else(|| "job-local".into()),
                    vocals_path: vocals_path.into(),
                    instrumental_path: instrumental_path.into(),
                    backend: entry.backend,
                })
            }
            "error" => Err(anyhow!(
                "{}",
                terminal
                    .message
                    .unwrap_or_else(|| "Engine returned an unspecified error".into())
            )),
            other => Err(anyhow!("Unexpected terminal engine event `{}`", other)),
        }
    }

    pub async fn sync_uvr_catalog(&self) -> Result<usize> {
        let client = reqwest::Client::builder()
            .user_agent("PrismSplit/0.1.0")
            .timeout(std::time::Duration::from_secs(10))
            .http1_only()
            .build()?;
        let url = "https://raw.githubusercontent.com/TRvlvr/application_data/main/filelists/download_checks.json";
        let response = client.get(url).send().await?;
        let data: serde_json::Value = response.json().await?;

        let mut new_entries = Vec::new();
        let lists = [
            ("mdx_download_list", "mdx"),
            ("vr_download_list", "vr"),
            ("mdx23_download_list", "mdx"),
            ("mdx23c_download_list", "mdx"),
            ("roformer_download_list", "mdx"),
        ];

        for (list_key, backend) in lists {
            if let Some(list) = data.get(list_key).and_then(|value| value.as_object()) {
                for (name, filename) in list {
                    if let Some(filename_str) = filename.as_str() {
                        let id = format!("{}_{}", backend, filename_str.replace('.', "_"));
                        new_entries.push(ModelCatalogEntry {
                            id,
                            name: name
                                .replace("MDX-Net Model: ", "")
                                .replace("VR Arch Single Model v5: ", "")
                                .replace("VR Arch Single Model v4: ", ""),
                            backend: backend.into(),
                            output_kind: "vocals_instrumental".into(),
                            url: format!("https://github.com/TRvlvr/model_repo/releases/download/all_public_uvr_models/{}", filename_str),
                            sha256: "replace-with-real-sha256".into(),
                            size_bytes: 0,
                            filename: filename_str.into(),
                            version: "1.0.0".into(),
                            is_installed: false,
                            local_path: None,
                        });
                    }
                }
            }
        }

        if let Some(demucs_list) = data
            .get("demucs_download_list")
            .and_then(|value| value.as_object())
        {
            for (name, files) in demucs_list {
                if let Some(files_obj) = files.as_object() {
                    if let Some((filename, url)) =
                        files_obj.iter().find(|(key, _)| key.ends_with(".th"))
                    {
                        if let Some(url_str) = url.as_str() {
                            let id = format!("demucs_{}", filename.replace('.', "_"));
                            new_entries.push(ModelCatalogEntry {
                                id,
                                name: name
                                    .replace("Demucs v4: ", "")
                                    .replace("Demucs v3: ", "")
                                    .replace("Demucs v2: ", "")
                                    .replace("Demucs v1: ", ""),
                                backend: "demucs".into(),
                                output_kind: "stems".into(),
                                url: url_str.into(),
                                sha256: "replace-with-real-sha256".into(),
                                size_bytes: 0,
                                filename: filename.into(),
                                version: "1.0.0".into(),
                                is_installed: false,
                                local_path: None,
                            });
                        }
                    }
                }
            }
        }

        let mut current_catalog = self.model_registry.load_catalog().unwrap_or_default();
        let new_entries = Arc::new(new_entries);
        let current_catalog_ids: Vec<String> = current_catalog
            .iter()
            .map(|entry| entry.id.clone())
            .collect();
        let current_catalog_filenames: Vec<String> = current_catalog
            .iter()
            .map(|entry| entry.filename.clone())
            .collect();
        let mut entries_to_process = Vec::new();

        for entry in new_entries.iter() {
            if !current_catalog_ids.contains(&entry.id)
                && !current_catalog_filenames.contains(&entry.filename)
            {
                entries_to_process.push(entry.clone());
            }
        }

        let mut fix_count = 0;
        for entry in &mut current_catalog {
            if entry.size_bytes == 0 && !entry.url.is_empty() && fix_count < 100 {
                entries_to_process.push(entry.clone());
                fix_count += 1;
            }
        }

        let client_ref = client.clone();
        let fetched_entries = stream::iter(entries_to_process)
            .map(|mut entry| {
                let client = client_ref.clone();
                async move {
                    let mut delay = std::time::Duration::from_millis(500);
                    let mut retries = 3;
                    while retries > 0 {
                        match client.head(&entry.url).send().await {
                            Ok(response) => {
                                if response.status().is_success() {
                                    if let Some(length) = response.content_length() {
                                        entry.size_bytes = length;
                                    }
                                    break;
                                } else if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                                    tokio::time::sleep(delay * 2).await;
                                }
                            }
                            Err(_) => {
                                tokio::time::sleep(delay).await;
                            }
                        }
                        retries -= 1;
                        delay *= 2;
                    }
                    entry
                }
            })
            .buffer_unordered(5)
            .collect::<Vec<_>>()
            .await;

        let mut added_count = 0;
        for entry in fetched_entries {
            if let Some(existing) = current_catalog.iter_mut().find(|item| item.id == entry.id) {
                existing.size_bytes = entry.size_bytes;
            } else {
                current_catalog.push(entry);
                added_count += 1;
            }
        }

        self.model_registry.save_catalog(&current_catalog)?;
        Ok(added_count)
    }

    pub async fn scan_local_models(&self, path: String) -> Result<usize> {
        let scan_path = Path::new(&path);
        if !scan_path.is_dir() {
            return Err(anyhow!("Path `{}` is not a valid directory", path));
        }

        let mdx_data_url = "https://raw.githubusercontent.com/TRvlvr/application_data/main/mdx_model_data/model_data_new.json";
        let vr_data_url = "https://raw.githubusercontent.com/TRvlvr/application_data/main/vr_model_data/model_data_new.json";

        let client = reqwest::Client::builder()
            .user_agent("PrismSplit/0.1.0")
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let mdx_data: serde_json::Value = client
            .get(mdx_data_url)
            .send()
            .await?
            .json()
            .await
            .unwrap_or_default();
        let vr_data: serde_json::Value = client
            .get(vr_data_url)
            .send()
            .await?
            .json()
            .await
            .unwrap_or_default();

        let mut added_count = 0;
        let mut current_catalog = self.model_registry.load_catalog().unwrap_or_default();

        for entry in walkdir::WalkDir::new(scan_path)
            .into_iter()
            .filter_map(|entry| entry.ok())
        {
            let file_path = entry.path();
            if file_path.is_file() {
                let extension = file_path
                    .extension()
                    .and_then(|value| value.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if extension == "onnx"
                    || extension == "pth"
                    || extension == "th"
                    || extension == "ckpt"
                {
                    if let Ok(hash) = md5_file(file_path) {
                        let mut model_name = file_path
                            .file_stem()
                            .and_then(|value| value.to_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        let mut backend = match extension.as_str() {
                            "onnx" => "mdx",
                            "pth" | "ckpt" => "vr",
                            _ => "demucs",
                        };

                        if let Some(info) = mdx_data.get(&hash) {
                            backend = "mdx";
                            if let Some(stem) = info.get("primary_stem") {
                                model_name =
                                    format!("{} ({})", model_name, stem.as_str().unwrap_or(""));
                            }
                        } else if let Some(info) = vr_data.get(&hash) {
                            backend = "vr";
                            if let Some(stem) = info.get("primary_stem") {
                                model_name =
                                    format!("{} ({})", model_name, stem.as_str().unwrap_or(""));
                            }
                        }

                        let id = format!("local_{}", hash);
                        if !current_catalog.iter().any(|item| item.id == id) {
                            current_catalog.push(ModelCatalogEntry {
                                id,
                                name: format!("[LOCAL] {}", model_name),
                                backend: backend.into(),
                                output_kind: "vocals_instrumental".into(),
                                url: String::new(),
                                sha256: "replace-with-real-sha256".into(),
                                size_bytes: std::fs::metadata(file_path)
                                    .map(|metadata| metadata.len())
                                    .unwrap_or(0),
                                filename: file_path
                                    .file_name()
                                    .and_then(|name| name.to_str())
                                    .unwrap_or("")
                                    .into(),
                                version: "local".into(),
                                is_installed: true,
                                local_path: Some(file_path.to_string_lossy().into()),
                            });
                            added_count += 1;
                        }
                    }
                }
            }
        }

        self.model_registry.save_catalog(&current_catalog)?;
        Ok(added_count)
    }

    pub async fn kill_process(&self, job_id: &str) {
        let mut procs = self.active_processes.lock().await;
        if let Some(child) = procs.remove(job_id) {
            if let Some(pid) = child.id() {
                println!("KILLING child process [{}] with PID {}", job_id, pid);
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("taskkill")
                        .args(["/PID", &pid.to_string(), "/F", "/T"])
                        .output();
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let _ = std::process::Command::new("kill")
                        .arg("-9")
                        .arg(pid.to_string())
                        .output();
                }
            }
        }
    }

    pub fn list_active_processes(&self) -> Vec<(String, Option<u32>)> {
        if let Ok(procs) = self.active_processes.try_lock() {
            procs.iter().map(|(id, child)| (id.clone(), child.id())).collect()
        } else {
            Vec::new()
        }
    }

    pub fn kill_all_active_processes(&self) {
        if let Ok(mut procs) = self.active_processes.try_lock() {
            for (id, child) in procs.drain() {
                if let Some(pid) = child.id() {
                    println!("KILLING child process [{}] with PID {}", id, pid);
                    #[cfg(target_os = "windows")]
                    {
                        let _ = std::process::Command::new("taskkill")
                            .args(["/PID", &pid.to_string(), "/F", "/T"])
                            .output();
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        let _ = std::process::Command::new("kill")
                            .arg("-9")
                            .arg(pid.to_string())
                            .output();
                    }
                }
            }
        }
    }

    pub fn copy_file(&self, source: &str, destination: &str) -> Result<()> {
        let src_path = Path::new(source);
        let dest_path = Path::new(destination);
        if !src_path.is_file() {
            return Err(anyhow!("Source stem file not found: {}", source));
        }
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(src_path, dest_path)?;
        Ok(())
    }

    pub fn export_all_stems(&self, source_paths: &[String], dest_dir: &str) -> Result<()> {
        let dest_path = Path::new(dest_dir);
        std::fs::create_dir_all(dest_path)?;
        for src in source_paths {
            let path = Path::new(src);
            if path.is_file() {
                if let Some(filename) = path.file_name() {
                    let target = dest_path.join(filename);
                    std::fs::copy(path, target)?;
                }
            }
        }
        Ok(())
    }

    pub fn cleanup_previews(&self, source_paths: &[String]) {
        for src in source_paths {
            let path = Path::new(src);
            if path.is_file() {
                let _ = std::fs::remove_file(path);
            }
        }
        let temp_dir = std::env::temp_dir().join("PrismSplit_Preview");
        if temp_dir.is_dir() {
            let _ = std::fs::remove_dir(temp_dir);
        }
    }
}

fn migrate_config(mut config: AppConfig) -> AppConfig {
    if config.version < 1 {
        config.version = 1;
        // future migrations can go here
    }
    config
}

fn load_config(path: &Path) -> AppConfig {
    if !path.exists() {
        return AppConfig::default();
    }
    
    match std::fs::read_to_string(path) {
        Ok(content) => {
            match serde_json::from_str::<AppConfig>(&content) {
                Ok(mut config) => {
                    if config.version < 1 {
                        config = migrate_config(config);
                        let _ = save_config(path, &config);
                    }
                    config
                }
                Err(err) => {
                    eprintln!("WARNING: Failed to parse config file: {}. Backing up.", err);
                    let backup_path = path.with_extension(format!(
                        "corrupt.{}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    ));
                    let _ = std::fs::rename(path, backup_path);
                    AppConfig::default()
                }
            }
        }
        Err(err) => {
            eprintln!("WARNING: Failed to read config file: {}.", err);
            AppConfig::default()
        }
    }
}

fn save_config(path: &Path, config: &AppConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    let temp_path = path.with_extension("json.tmp");
    std::fs::write(&temp_path, content)?;
    if let Err(err) = std::fs::rename(&temp_path, path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(err.into());
    }
    Ok(())
}
