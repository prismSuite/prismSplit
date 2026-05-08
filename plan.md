# PrismSplit - Master Development Prompt
## "Professional Audio Separation Desktop Application"

**Status:** Production-Ready Implementation Guide  
**Version:** 1.0  
**Date:** 2025-05-08  
**Target:** Windows 10/11, Tauri + React/TypeScript + Rust + Python UVR Engine  

---

## EXECUTIVE BRIEF

PrismSplit is a **high-performance, professional-grade audio separation tool** for Windows desktops. It specializes in **Karaoke Separation** (vocal/instrumental extraction) but supports multi-architecture inference (MDX-Net, VR Architecture, Demucs, Roformer) through an **embedded Python UVR engine**.

### Core Principles
1. **No Mocks.** Real UVR engine integration with genuine model management.
2. **No Hardcoding.** Dynamic model registry, actual file I/O, real-time job orchestration.
3. **Industrial Design Standard.** Brutal skeuomorphism: 3D beveled borders, high information density, Win32 aesthetic.
4. **Professional Trust.** Console logs, error transparency, byte-accurate progress tracking.

---

## ARCHITECTURE OVERVIEW

```
┌────────────────────────────────────────────────────────────────┐
│                    TAURI WINDOW (React/TS)                      │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  TopBar: [PrismSplit] [Engine] [Models] [Extraction]     │   │
│  ├──────────────────────────────────────────────────────────┤   │
│  │                                                            │   │
│  │  [Control Panel]  |  [Workspace]  |  [Parameters/Logs]    │   │
│  │  • Engine Status  |  • Job Queue   |  • Settings          │   │
│  │  • Model List     |  • Drag/Drop   |  • Audio Preview     │   │
│  │  • Settings       |  • Progress    |                      │   │
│  │                                                            │   │
│  ├──────────────────────────────────────────────────────────┤   │
│  │  [Console Logs] Status / Info / Errors (Monospace)        │   │
│  └──────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────┘
         ↕ Newline-Delimited JSON IPC
┌────────────────────────────────────────────────────────────────┐
│              RUST BACKEND (Tauri + Orchestration)               │
│  ├─ Engine Manager: Lifecycle, health checks, process supervision
│  ├─ Model Registry: Catalog sync, local scanning, SHA-256 verify
│  ├─ Job Scheduler: Queue management, concurrency control
│  ├─ File I/O: Download manager, audio validation, model unpacking
│  └─ IPC Bridge: JSON protocol between frontend and Python
└────────────────────────────────────────────────────────────────┘
         ↕ Newline-Delimited JSON Pipe
┌────────────────────────────────────────────────────────────────┐
│         PYTHON ENGINE (Embedded UVR + ONNX Runtime)             │
│  ├─ Model Loader: Load weights from registry, validate checksums
│  ├─ Inference Pipeline: Audio preprocessing → separation → mixing
│  ├─ Backend Handlers: MDX, VR, Demucs, Roformer adapters
│  └─ Telemetry: Real-time logs, progress events, error reporting
└────────────────────────────────────────────────────────────────┘
```

---

## TECH STACK & SPECIFICATIONS

### Frontend (React/TypeScript)
- **Framework:** React 18.2+, TypeScript 5.x
- **Bundler:** Vite
- **Styling:** Vanilla CSS (NO Tailwind, NO preprocessors)
- **State Management:** React Hooks (Context API for global state)
- **Communication:** Tauri `invoke()` command bridge

### Backend (Rust)
- **Runtime:** Tauri 1.5+
- **Async:** Tokio 1.x
- **Serialization:** serde_json (JSON)
- **File I/O:** tokio::fs, std::process
- **Hashing:** sha2 (for model verification)
- **HTTP:** reqwest (for model downloads)

### Python Engine
- **Interpreter:** Embedded Python 3.10+ (distributed with app)
- **Core Library:** Ultimate Vocal Remover (UVR) vendored source
- **Inference:** ONNX Runtime (CPU + GPU backends)
- **Audio:** librosa, soundfile, numpy
- **Protocol:** Newline-delimited JSON over stdin/stdout

### Design System
- **Specification:** Industrial Audio Skeuomorphism (see `DESIGN.md`)
- **Palette:** Hard-edged grays + 4 accent colors (Green, Blue, Red, Yellow)
- **Typography:** Tahoma (UI), Cascadia Mono (Console), IBM Plex Sans (Display)
- **Components:** 3D beveled buttons, sunken inputs, resizable dividers, no drop-shadows

---

## DETAILED IMPLEMENTATION ROADMAP

### PHASE 1: CORE ARCHITECTURE (Weeks 1-2)

#### 1.1 Rust Orchestration Backbone
**Goal:** Build reliable IPC, job queue, and process management.

**Deliverables:**
```rust
// src-tauri/src/main.rs
#[tauri::command]
async fn initialize_engine() -> Result<EngineStatus, String> {
    // 1. Check if Python runtime is unpacked
    // 2. Verify virtual environment
    // 3. Validate UVR module installation
    // 4. Spawn Python subprocess with stdio pipes
    // 5. Send test message to confirm IPC
    // 6. Return status: { ready: bool, version: String, models_path: String }
}

// src-tauri/src/ipc/protocol.rs
#[derive(Serialize, Deserialize)]
pub enum IPCMessage {
    // → Python (Frontend → Python via Rust)
    LoadModel { model_id: String, backend: String },
    ProcessAudio { input_path: String, model_id: String, output_path: String },
    CancelJob { job_id: String },
    
    // ← Python (Python → Frontend via Rust)
    ModelLoaded { model_id: String, inference_time_ms: u32 },
    Progress { job_id: String, percent: f32, eta_seconds: u32 },
    LogLine { level: String, message: String }, // "INFO", "ERROR", "DEBUG"
    JobComplete { job_id: String, output_path: String },
    JobFailed { job_id: String, error: String },
}

// src-tauri/src/engine/mod.rs
pub struct EngineManager {
    process: Child,
    stdin: BufWriter<ChildStdin>,
    rx: UnboundedReceiver<IPCMessage>,
}

impl EngineManager {
    pub async fn new() -> Result<Self> {
        // Spawn Python subprocess
        // Set up pipe readers/writers
        // Spawn tokio::task for message polling
        // Return Self
    }
    
    pub async fn send_message(&mut self, msg: IPCMessage) -> Result<()> {
        // Serialize to JSON
        // Write to stdin
        // Flush
    }
}
```

**Key Files to Create:**
- `src-tauri/src/main.rs` - Tauri command handlers
- `src-tauri/src/ipc/protocol.rs` - Message definitions
- `src-tauri/src/engine/mod.rs` - Process management
- `src-tauri/src/engine/job_queue.rs` - Job scheduler
- `src-tauri/src/models/registry.rs` - Model catalog

#### 1.2 Python Engine Minimal Runner
**Goal:** Establish working IPC and basic model loading.

**Deliverables:**
```python
# engine/python/prismsplit_engine.py
import sys
import json
from pathlib import Path

class Engine:
    def __init__(self):
        self.models = {}
        self.current_model = None
        
    def load_model(self, model_id: str, backend: str):
        """Load a model from registry by ID."""
        try:
            model_path = self.get_model_path(model_id)
            # Import correct backend handler
            handler = self._get_handler(backend)
            # Load model weights
            self.models[model_id] = handler.load(model_path)
            self.current_model = model_id
            self._send_message({
                "type": "ModelLoaded",
                "model_id": model_id,
                "inference_time_ms": 0
            })
        except Exception as e:
            self._send_error(f"Failed to load model: {e}")
    
    def process_audio(self, input_path: str, model_id: str, output_path: str):
        """Separate audio using loaded model."""
        try:
            audio = self._load_audio(input_path)
            model = self.models[model_id]
            
            # Run inference
            voc, inst = model.separate(audio)
            
            # Save outputs
            self._save_audio(voc, f"{output_path}_vocals.wav")
            self._save_audio(inst, f"{output_path}_instrumental.wav")
            
            self._send_message({
                "type": "JobComplete",
                "job_id": "...",
                "output_path": output_path
            })
        except Exception as e:
            self._send_error(f"Inference failed: {e}")
    
    def _send_message(self, data: dict):
        """Send JSON message to Rust backend."""
        sys.stdout.write(json.dumps(data) + "\n")
        sys.stdout.flush()
    
    def run(self):
        """Main loop: read messages from stdin."""
        for line in sys.stdin:
            try:
                msg = json.loads(line.strip())
                self.handle_message(msg)
            except json.JSONDecodeError:
                self._send_error("Invalid JSON received")
            except Exception as e:
                self._send_error(f"Unexpected error: {e}")

if __name__ == "__main__":
    engine = Engine()
    engine.run()
```

**Key Files to Create:**
- `engine/python/prismsplit_engine.py` - Main runner
- `engine/python/backends/__init__.py` - Backend registry
- `engine/python/backends/mdx_handler.py` - MDX backend
- `engine/python/backends/demucs_handler.py` - Demucs backend
- `engine/python/utils/audio.py` - Audio I/O helpers
- `engine/python/tests/test_ipc.py` - IPC testing

---

### PHASE 2: MODEL MANAGEMENT (Weeks 2-3)

#### 2.1 Model Registry & Catalog
**Goal:** Real model data, actual downloads, SHA-256 verification.

**Deliverables:**
```rust
// src-tauri/src/models/registry.rs
#[derive(Serialize, Deserialize, Clone)]
pub struct ModelMetadata {
    pub id: String,
    pub name: String,
    pub backend: String, // "MDX", "Demucs", "VR", "Roformer"
    pub version: String,
    pub size_mb: f32,
    pub sha256: String,
    pub url: String,
    pub source: String, // "uvr_official", "community"
}

pub struct ModelRegistry {
    models: Vec<ModelMetadata>,
    cache_path: PathBuf,
    local_models: HashMap<String, ModelMetadata>,
}

impl ModelRegistry {
    pub async fn sync_uvr_catalog() -> Result<Vec<ModelMetadata>> {
        // Fetch official UVR model listing from GitHub
        // Parse JSON catalog
        // Filter duplicates
        // Return Vec<ModelMetadata>
    }
    
    pub async fn download_model(
        &self,
        model_id: &str,
        progress_callback: Box<dyn Fn(f32)>,
    ) -> Result<PathBuf> {
        // Validate model exists in catalog
        // Create models/ directory if missing
        // Download with streaming (report progress every 1MB)
        // Verify SHA-256
        // Return path to downloaded model
    }
    
    pub async fn scan_local_directory(&mut self, path: &Path) -> Result<()> {
        // Recursively scan directory for supported model files
        // Compute MD5 of each file
        // Match against UVR catalog by MD5
        // Populate self.local_models
        // Create symlink or copy to models/ registry
    }
    
    pub fn list_available(&self) -> Vec<ModelMetadata> {
        // Return all models (downloaded + catalog)
    }
    
    pub fn get_local_path(&self, model_id: &str) -> Result<PathBuf> {
        // Return absolute path to model file
    }
}

// Tauri command
#[tauri::command]
async fn sync_uvr_models() -> Result<Vec<ModelMetadata>, String> {
    ModelRegistry::sync_uvr_catalog().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn download_model(
    model_id: String,
    handle: tauri::AppHandle,
) -> Result<String, String> {
    let registry = ModelRegistry::new();
    registry
        .download_model(&model_id, Box::new(|progress| {
            handle.emit_all("model:download-progress", json!({ "progress": progress })).ok();
        }))
        .await
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn scan_local_models(directory: String) -> Result<Vec<String>, String> {
    let mut registry = ModelRegistry::new();
    registry
        .scan_local_directory(Path::new(&directory))
        .await
        .map(|_| registry.list_available().iter().map(|m| m.id.clone()).collect())
        .map_err(|e| e.to_string())
}
```

**Key Files to Create:**
- `src-tauri/src/models/registry.rs` - Model catalog management
- `src-tauri/src/models/download.rs` - Download logic with streaming
- `src-tauri/src/models/verify.rs` - SHA-256 verification
- `engine/python/model_loader.py` - UVR model loading adapter

#### 2.2 React UI: Model Management Panel
**Goal:** Display catalog, trigger downloads, show local scans.

**Deliverables:**
```tsx
// src/components/ModelRegistry.tsx
import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './ModelRegistry.css';

interface ModelMetadata {
  id: string;
  name: string;
  backend: string;
  version: string;
  size_mb: number;
  source: string;
}

export const ModelRegistry: React.FC = () => {
  const [models, setModels] = useState<ModelMetadata[]>([]);
  const [downloading, setDownloading] = useState<Set<string>>(new Set());
  const [progress, setProgress] = useState<Record<string, number>>({});

  useEffect(() => {
    syncUVRCatalog();
  }, []);

  const syncUVRCatalog = async () => {
    try {
      const result = await invoke<ModelMetadata[]>('sync_uvr_models');
      setModels(result);
    } catch (error) {
      console.error('Failed to sync catalog:', error);
    }
  };

  const downloadModel = async (modelId: string) => {
    setDownloading(prev => new Set([...prev, modelId]));
    try {
      await invoke<string>('download_model', { model_id: modelId });
      // Refresh list
      await syncUVRCatalog();
    } catch (error) {
      console.error('Download failed:', error);
    } finally {
      setDownloading(prev => {
        const next = new Set(prev);
        next.delete(modelId);
        return next;
      });
    }
  };

  const scanLocalDirectory = async () => {
    try {
      const dir = await invoke<string>('pick_directory');
      await invoke('scan_local_models', { directory: dir });
      await syncUVRCatalog();
    } catch (error) {
      console.error('Scan failed:', error);
    }
  };

  return (
    <div className="model-registry">
      <fieldset className="model-registry__controls">
        <legend>Model Management</legend>
        <button onClick={syncUVRCatalog}>Sync UVR Servers</button>
        <button onClick={scanLocalDirectory}>Scan Local Directory</button>
      </fieldset>

      <div className="model-registry__list">
        {models.map(model => (
          <div key={model.id} className="model-card">
            <div className="model-card__header">
              <h3>{model.name}</h3>
              <span className="model-card__backend">{model.backend}</span>
            </div>
            <div className="model-card__meta">
              <p>Size: {model.size_mb.toFixed(1)} MB</p>
              <p>Version: {model.version}</p>
            </div>
            {downloading.has(model.id) ? (
              <div className="progress-bar">
                <div
                  className="progress-bar__fill"
                  style={{ width: `${progress[model.id] || 0}%` }}
                />
              </div>
            ) : (
              <button
                onClick={() => downloadModel(model.id)}
                className="btn btn--primary"
              >
                Download
              </button>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};
```

**Key Files to Create:**
- `src/components/ModelRegistry.tsx` - Model management UI
- `src/components/ModelRegistry.css` - Styling per DESIGN.md
- `src/hooks/useModelSync.ts` - Model sync logic
- `src/types/models.ts` - TypeScript interfaces

---

### PHASE 3: JOB ORCHESTRATION & EXTRACTION (Weeks 3-4)

#### 3.1 Job Queue & Scheduler
**Goal:** Real queue management, concurrent job execution, state persistence.

**Deliverables:**
```rust
// src-tauri/src/jobs/mod.rs
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum JobState {
    Pending,
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Job {
    pub id: String,
    pub input_path: String,
    pub output_dir: String,
    pub model_id: String,
    pub backend: String,
    pub state: JobState,
    pub progress: f32,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
}

pub struct JobScheduler {
    jobs: HashMap<String, Job>,
    queue: VecDeque<String>,
    engine: EngineManager,
    max_concurrent: usize,
    running: HashSet<String>,
}

impl JobScheduler {
    pub async fn submit_job(
        &mut self,
        input_path: String,
        model_id: String,
        backend: String,
    ) -> Result<String> {
        // Validate input file exists and is audio
        // Create Job with Pending state
        // Generate unique job ID
        // Store in self.jobs
        // Add to self.queue
        // Trigger processing if capacity
        // Return job ID
    }

    pub async fn process_queue(&mut self) {
        while !self.queue.is_empty() && self.running.len() < self.max_concurrent {
            if let Some(job_id) = self.queue.pop_front() {
                self.start_job(&job_id).await.ok();
            }
        }
    }

    async fn start_job(&mut self, job_id: &str) -> Result<()> {
        let job = self.jobs.get_mut(job_id).ok_or("Job not found")?;
        job.state = JobState::Running;
        job.started_at = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs());

        let msg = IPCMessage::ProcessAudio {
            input_path: job.input_path.clone(),
            model_id: job.model_id.clone(),
            output_path: job.output_dir.clone(),
        };

        self.engine.send_message(msg).await?;
        self.running.insert(job_id.to_string());
        Ok(())
    }

    pub fn get_job(&self, job_id: &str) -> Option<&Job> {
        self.jobs.get(job_id)
    }

    pub fn list_jobs(&self) -> Vec<Job> {
        self.jobs.values().cloned().collect()
    }
}

// Tauri commands
#[tauri::command]
async fn submit_extraction_job(
    input_path: String,
    model_id: String,
    backend: String,
    state: tauri::State<'_, std::sync::Mutex<AppState>>,
) -> Result<String, String> {
    let mut app_state = state.lock().unwrap();
    let job_id = uuid::Uuid::new_v4().to_string();
    
    app_state
        .scheduler
        .submit_job(input_path, model_id, backend)
        .await
        .map(|_| job_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_jobs(state: tauri::State<'_, std::sync::Mutex<AppState>>) -> Vec<Job> {
    let app_state = state.lock().unwrap();
    app_state.scheduler.list_jobs()
}

#[tauri::command]
fn get_job_status(job_id: String, state: tauri::State<'_, std::sync::Mutex<AppState>>) -> Option<Job> {
    let app_state = state.lock().unwrap();
    app_state.scheduler.get_job(&job_id).cloned()
}
```

**Key Files to Create:**
- `src-tauri/src/jobs/mod.rs` - Job definitions
- `src-tauri/src/jobs/scheduler.rs` - Job queue logic
- `src-tauri/src/jobs/persistence.rs` - SQLite job history (optional)

#### 3.2 React UI: Extraction Workspace
**Goal:** Drag-and-drop input, real-time progress, job queue display.

**Deliverables:**
```tsx
// src/components/ExtractionPanel.tsx
import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './ExtractionPanel.css';

interface Job {
  id: string;
  input_path: string;
  output_dir: string;
  model_id: string;
  state: 'Pending' | 'Running' | 'Completed' | 'Failed' | 'Cancelled';
  progress: number;
}

export const ExtractionPanel: React.FC = () => {
  const [jobs, setJobs] = useState<Job[]>([]);
  const [selectedModel, setSelectedModel] = useState<string>('');
  const [selectedBackend, setSelectedBackend] = useState<string>('MDX');

  useEffect(() => {
    loadJobs();
    const interval = setInterval(loadJobs, 1000);
    return () => clearInterval(interval);
  }, []);

  const loadJobs = async () => {
    try {
      const result = await invoke<Job[]>('list_jobs');
      setJobs(result);
    } catch (error) {
      console.error('Failed to load jobs:', error);
    }
  };

  const handleDragDrop = async (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    const files = e.dataTransfer.files;
    if (files.length === 0) return;

    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      const outputDir = file.path.replace(/\.[^.]+$/, '_outputs');

      try {
        await invoke('submit_extraction_job', {
          input_path: file.path,
          model_id: selectedModel,
          backend: selectedBackend,
        });
      } catch (error) {
        console.error('Failed to submit job:', error);
      }
    }
  };

  return (
    <div className="extraction-panel">
      <fieldset className="extraction-panel__config">
        <legend>Configuration</legend>
        <div className="form-group">
          <label>Model:</label>
          <select
            value={selectedModel}
            onChange={e => setSelectedModel(e.target.value)}
            className="input input--select"
          >
            <option value="">-- Select Model --</option>
            {/* Populated from model registry */}
          </select>
        </div>
        <div className="form-group">
          <label>Backend:</label>
          <select
            value={selectedBackend}
            onChange={e => setSelectedBackend(e.target.value)}
            className="input input--select"
          >
            <option value="MDX">MDX-Net</option>
            <option value="Demucs">Demucs</option>
            <option value="VR">VR Architecture</option>
            <option value="Roformer">Roformer</option>
          </select>
        </div>
      </fieldset>

      <div
        className="extraction-panel__drop-zone"
        onDragOver={e => e.preventDefault()}
        onDrop={handleDragDrop}
      >
        <p>Drag & drop audio files here to extract</p>
      </div>

      <div className="extraction-panel__queue">
        <h3>Job Queue</h3>
        {jobs.map(job => (
          <div key={job.id} className="job-card">
            <div className="job-card__header">
              <span className="job-card__filename">
                {job.input_path.split('\\').pop()}
              </span>
              <span
                className={`job-card__state job-card__state--${job.state.toLowerCase()}`}
              >
                {job.state}
              </span>
            </div>
            {job.state === 'Running' && (
              <div className="progress-bar">
                <div
                  className="progress-bar__fill"
                  style={{ width: `${job.progress}%` }}
                />
                <span className="progress-bar__label">{job.progress.toFixed(0)}%</span>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};
```

**Key Files to Create:**
- `src/components/ExtractionPanel.tsx` - Main extraction UI
- `src/components/JobCard.tsx` - Job display component
- `src/components/ExtractionPanel.css` - Styling per DESIGN.md
- `src/hooks/useJobQueue.ts` - Job polling logic

---

### PHASE 4: ENGINE INTEGRATION & PYTHON BACKENDS (Weeks 4-5)

#### 4.1 Python Backend Handlers
**Goal:** Real inference via ONNX Runtime, actual audio I/O.

**Deliverables:**
```python
# engine/python/backends/mdx_handler.py
import numpy as np
import onnxruntime as ort
from pathlib import Path
from typing import Tuple

class MDXHandler:
    """MDX-Net inference backend."""
    
    def __init__(self, model_path: str):
        self.model_path = Path(model_path)
        self.session = ort.InferenceSession(
            str(self.model_path),
            providers=['CUDAExecutionProvider', 'CPUExecutionProvider']
        )
        self.input_name = self.session.get_inputs()[0].name
        self.output_names = [o.name for o in self.session.get_outputs()]
    
    def preprocess(self, audio: np.ndarray) -> np.ndarray:
        """Normalize audio to [-1, 1]."""
        max_val = np.max(np.abs(audio))
        if max_val > 0:
            audio = audio / max_val
        return audio.astype(np.float32)
    
    def separate(self, audio: np.ndarray, sr: int = 44100) -> Tuple[np.ndarray, np.ndarray]:
        """
        Separate audio into vocals and instrumental.
        
        Args:
            audio: Input audio array (mono or stereo)
            sr: Sample rate
        
        Returns:
            Tuple of (vocals, instrumental) arrays
        """
        # Ensure stereo
        if audio.ndim == 1:
            audio = np.stack([audio, audio])
        elif audio.shape[0] != 2:
            audio = audio.T
        
        # Preprocess
        audio = self.preprocess(audio)
        
        # Run inference
        ort_inputs = {self.input_name: np.expand_dims(audio, 0)}
        ort_outs = self.session.run(self.output_names, ort_inputs)
        
        # Process outputs
        mask = ort_outs[0][0]  # Vocal mask
        vocals = audio * mask
        instrumental = audio * (1 - mask)
        
        return vocals, instrumental

# engine/python/backends/demucs_handler.py
import torch
from demucs.pretrained import get_model
from typing import Tuple

class DemucsHandler:
    """Demucs inference backend."""
    
    def __init__(self, model_path: str, model_name: str = "htdemucs"):
        self.device = "cuda" if torch.cuda.is_available() else "cpu"
        self.model = get_model(model_name).to(self.device)
        self.model.eval()
    
    def separate(self, audio: np.ndarray, sr: int = 44100) -> Tuple[np.ndarray, np.ndarray]:
        """
        Separate audio using Demucs.
        Returns (vocals, instrumental).
        """
        # Ensure stereo tensor
        if audio.ndim == 1:
            audio = np.stack([audio, audio])
        audio_tensor = torch.from_numpy(audio).float().to(self.device)
        
        with torch.no_grad():
            sources = self.model(audio_tensor.unsqueeze(0))
        
        # Demucs returns [drums, bass, other, vocals]
        # Combine drums + bass + other = instrumental
        vocals = sources[0, 3].cpu().numpy()
        instrumental = (sources[0, 0] + sources[0, 1] + sources[0, 2]).cpu().numpy()
        
        return vocals, instrumental

# engine/python/prismsplit_engine.py (Updated)
import json
import sys
from pathlib import Path
import numpy as np
import soundfile as sf
from backends.mdx_handler import MDXHandler
from backends.demucs_handler import DemucsHandler

class Engine:
    BACKENDS = {
        'MDX': MDXHandler,
        'Demucs': DemucsHandler,
    }
    
    def __init__(self):
        self.models = {}
        self.current_model = None
    
    def load_model(self, model_id: str, backend: str, model_path: str):
        """Load model from file."""
        try:
            handler_class = self.BACKENDS[backend]
            self.models[model_id] = handler_class(model_path)
            self.current_model = model_id
            
            self._send({
                "type": "ModelLoaded",
                "model_id": model_id,
                "backend": backend,
                "status": "success"
            })
        except Exception as e:
            self._send_error(f"Model load failed: {e}")
    
    def process_audio(self, input_path: str, model_id: str, output_dir: str, job_id: str):
        """Separate audio."""
        try:
            # Load audio
            audio, sr = sf.read(input_path)
            
            # Get model
            model = self.models[model_id]
            
            # Separate
            vocals, instrumental = model.separate(audio, sr)
            
            # Save outputs
            Path(output_dir).mkdir(parents=True, exist_ok=True)
            vocal_path = Path(output_dir) / "vocals.wav"
            inst_path = Path(output_dir) / "instrumental.wav"
            
            sf.write(str(vocal_path), vocals.T if vocals.ndim > 1 else vocals, sr)
            sf.write(str(inst_path), instrumental.T if instrumental.ndim > 1 else instrumental, sr)
            
            self._send({
                "type": "JobComplete",
                "job_id": job_id,
                "vocal_path": str(vocal_path),
                "instrumental_path": str(inst_path),
                "status": "success"
            })
        except Exception as e:
            self._send({
                "type": "JobFailed",
                "job_id": job_id,
                "error": str(e)
            })
    
    def _send(self, data: dict):
        """Send JSON message to stdout."""
        sys.stdout.write(json.dumps(data) + "\n")
        sys.stdout.flush()
    
    def _send_error(self, msg: str):
        """Send error message."""
        self._send({
            "type": "Error",
            "message": msg,
            "level": "ERROR"
        })
    
    def run(self):
        """Main event loop."""
        for line in sys.stdin:
            try:
                msg = json.loads(line.strip())
                msg_type = msg.get("type")
                
                if msg_type == "LoadModel":
                    self.load_model(msg["model_id"], msg["backend"], msg["model_path"])
                elif msg_type == "ProcessAudio":
                    self.process_audio(
                        msg["input_path"],
                        msg["model_id"],
                        msg["output_dir"],
                        msg["job_id"]
                    )
                else:
                    self._send_error(f"Unknown message type: {msg_type}")
            except json.JSONDecodeError:
                self._send_error("Invalid JSON")
            except Exception as e:
                self._send_error(f"Unexpected error: {e}")

if __name__ == "__main__":
    engine = Engine()
    engine.run()
```

**Key Files to Create:**
- `engine/python/backends/mdx_handler.py` - MDX-Net backend
- `engine/python/backends/demucs_handler.py` - Demucs backend
- `engine/python/backends/vr_handler.py` - VR Architecture backend (if needed)
- `engine/python/utils/audio.py` - Audio I/O utilities
- `engine/python/requirements.txt` - Dependencies (onnxruntime, librosa, soundfile, torch)

---

### PHASE 5: UI DESIGN & POLISH (Weeks 5-6)

#### 5.1 CSS Implementation per DESIGN.md
**Goal:** Strict adherence to Industrial Audio Skeuomorphism spec.

**Key Guidelines:**
- **NO rounded corners** (border-radius: 0)
- **3D bevels** on all buttons (outset/inset borders)
- **Hard edges** on all UI elements
- **High information density** (compact spacing)
- **Monospace console** for logs
- **Terminal green** (#00ff00) for CRT effect
- **Tahoma 11px** for UI text
- **Cascadia Mono 10px** for logs

**Deliverables:**
```css
/* src/styles/index.css */
:root {
  /* Colors */
  --color-bg-primary: #1a1a1a;
  --color-bg-secondary: #0d0d0d;
  --color-bg-sunken: #1e1e1e;
  --color-fg-primary: #f0f0f0;
  --color-fg-secondary: #a0a0a0;
  --color-fg-disabled: #5a5a5a;
  --color-accent-green: #00ff00;
  --color-accent-green-dark: #2d5016;
  --color-accent-blue: #0099ff;
  --color-accent-red: #ff3333;
  --color-accent-yellow: #ffcc00;
  --color-border-light: #666666;
  --color-border-dark: #111111;
  --color-border-mid: #4a4a4a;
  
  /* Typography */
  --font-ui: Tahoma, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  --font-mono: 'Cascadia Mono', 'Courier New', monospace;
  --font-size-sm: 11px;
  --font-size-base: 12px;
  --font-size-lg: 14px;
  
  /* Spacing */
  --spacing-xs: 4px;
  --spacing-sm: 8px;
  --spacing-md: 16px;
  --spacing-lg: 24px;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
  border-radius: 0; /* CRITICAL */
}

body {
  background: var(--color-bg-primary);
  color: var(--color-fg-primary);
  font-family: var(--font-ui);
  font-size: var(--font-size-sm);
  line-height: 1.4;
  letter-spacing: -0.2px;
}

/* Buttons */
.btn {
  background: var(--color-border-mid);
  color: var(--color-fg-primary);
  border: 2px solid;
  border-color: var(--color-border-light) var(--color-border-dark) var(--color-border-dark) var(--color-border-light);
  padding: 6px 12px;
  font-size: var(--font-size-sm);
  font-weight: 700;
  cursor: pointer;
  transition: all 60ms linear;
}

.btn:hover {
  background: var(--color-border-light);
  border-color: #777 #111 #111 #777;
}

.btn:active {
  background: #3a3a3a;
  border-color: var(--color-border-dark) var(--color-border-light) var(--color-border-light) var(--color-border-dark);
  transform: translate(1px, 1px);
}

.btn--primary {
  background: var(--color-accent-green-dark);
  border-color: var(--color-accent-green) var(--color-bg-primary) var(--color-bg-primary) var(--color-accent-green);
}

.btn--primary:hover {
  background: #3d6018;
  border-color: #00ff00 #0a0a0a #0a0a0a #00ff00;
}

/* Inputs */
.input {
  background: var(--color-bg-sunken);
  color: var(--color-fg-primary);
  border: 2px solid;
  border-color: var(--color-border-dark) var(--color-border-inset-light) var(--color-border-inset-light) var(--color-border-dark);
  padding: 6px 8px;
  font-size: var(--font-size-sm);
  font-family: var(--font-ui);
}

.input:focus {
  border-color: var(--color-accent-blue);
  outline: none;
  box-shadow: inset 0 0 0 1px var(--color-accent-blue);
}

/* Console */
.console {
  background: var(--color-bg-secondary);
  border-top: 1px solid var(--color-border-mid);
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--color-accent-green);
  padding: var(--spacing-sm);
  overflow-y: auto;
  height: 120px;
}

.console__line {
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
}

.console__line--error { color: var(--color-accent-red); }
.console__line--warning { color: var(--color-accent-yellow); }
.console__line--info { color: var(--color-accent-green); }

/* Progress Bar */
.progress-bar {
  background: var(--color-bg-sunken);
  border: 1px solid var(--color-border-mid);
  height: 16px;
  position: relative;
  overflow: hidden;
}

.progress-bar__fill {
  background: linear-gradient(90deg, var(--color-accent-green-dark), var(--color-accent-green));
  height: 100%;
  transition: width 100ms linear;
}

.progress-bar__label {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  text-align: center;
  line-height: 16px;
  font-size: 9px;
  color: var(--color-fg-primary);
  font-weight: 700;
}
```

**Key Files to Create:**
- `src/styles/index.css` - Global styles per DESIGN.md
- `src/styles/components.css` - Component-specific styles
- `src/styles/layout.css` - Layout grid (TopBar, Panels, Console)

#### 5.2 React Components Library
**Goal:** Reusable, design-system-compliant components.

**Deliverables:**
```tsx
// src/components/index.ts
export { Button } from './Button';
export { Input } from './Input';
export { Select } from './Select';
export { ProgressBar } from './ProgressBar';
export { Console } from './Console';
export { Fieldset } from './Fieldset';
export { TopBar } from './TopBar';

// src/components/Button.tsx
import React from 'react';

interface ButtonProps {
  children: React.ReactNode;
  onClick?: () => void;
  variant?: 'default' | 'primary' | 'danger';
  disabled?: boolean;
  type?: 'button' | 'submit' | 'reset';
}

export const Button: React.FC<ButtonProps> = ({
  children,
  variant = 'default',
  disabled = false,
  ...props
}) => {
  const className = `btn btn--${variant} ${disabled ? 'btn--disabled' : ''}`;
  return (
    <button className={className} disabled={disabled} {...props}>
      {children}
    </button>
  );
};

// src/components/Console.tsx
import React, { useEffect, useRef } from 'react';

interface LogEntry {
  level: 'info' | 'error' | 'warning' | 'debug';
  message: string;
  timestamp: Date;
}

interface ConsoleProps {
  logs: LogEntry[];
  height?: number;
}

export const Console: React.FC<ConsoleProps> = ({ logs, height = 120 }) => {
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: 'auto' });
  }, [logs]);

  return (
    <div className="console" style={{ height }}>
      {logs.map((log, i) => (
        <div key={i} className={`console__line console__line--${log.level}`}>
          [{log.timestamp.toLocaleTimeString()}] {log.message}
        </div>
      ))}
      <div ref={endRef} />
    </div>
  );
};

// src/components/ProgressBar.tsx
import React from 'react';

interface ProgressBarProps {
  value: number; // 0-100
  label?: string;
  height?: number;
}

export const ProgressBar: React.FC<ProgressBarProps> = ({
  value,
  label,
  height = 16,
}) => {
  return (
    <div className="progress-bar" style={{ height }}>
      <div
        className="progress-bar__fill"
        style={{ width: `${Math.min(value, 100)}%` }}
      />
      {label && (
        <span className="progress-bar__label">{value.toFixed(0)}%</span>
      )}
    </div>
  );
};
```

**Key Files to Create:**
- `src/components/Button.tsx` - Button component
- `src/components/Input.tsx` - Input component
- `src/components/Select.tsx` - Select dropdown
- `src/components/ProgressBar.tsx` - Progress indicator
- `src/components/Console.tsx` - Log console
- `src/components/TopBar.tsx` - Top navigation
- `src/components/Fieldset.tsx` - Form grouping

---

### PHASE 6: INTEGRATION & TESTING (Weeks 6-7)

#### 6.1 End-to-End IPC Testing
**Goal:** Verify Rust ↔ Python communication.

**Deliverables:**
```rust
// src-tauri/src/tests/ipc_test.rs
#[tokio::test]
async fn test_engine_initialization() {
    let mut engine = EngineManager::new().await.expect("Engine init failed");
    let msg = IPCMessage::Ping;
    engine.send_message(msg).await.expect("Send failed");
}

#[tokio::test]
async fn test_model_loading() {
    // Test loading actual model
    // Verify model file exists
    // Send LoadModel message
    // Wait for ModelLoaded response
    // Assert model ID matches
}

#[tokio::test]
async fn test_audio_separation() {
    // Create test audio file
    // Submit separation job
    // Poll for completion
    // Verify output files exist
    // Check output contains valid audio
}
```

```python
# engine/python/tests/test_ipc.py
import json
import subprocess
import sys

def test_engine_startup():
    """Test Python engine starts and reads JSON."""
    proc = subprocess.Popen(
        [sys.executable, 'prismsplit_engine.py'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    
    # Send test message
    msg = json.dumps({"type": "Ping"})
    proc.stdin.write(msg + "\n")
    proc.stdin.flush()
    
    # Read response
    response = proc.stdout.readline()
    assert "Pong" in response or response, "No response from engine"
    
    proc.terminate()

def test_mdx_handler():
    """Test MDX model loading and separation."""
    from backends.mdx_handler import MDXHandler
    import numpy as np
    
    # Create dummy audio
    audio = np.random.randn(2, 44100).astype(np.float32)
    
    # Load model (requires actual ONNX file)
    handler = MDXHandler("path/to/model.onnx")
    
    # Separate
    vocals, inst = handler.separate(audio)
    
    assert vocals.shape == audio.shape
    assert inst.shape == audio.shape
```

**Key Files to Create:**
- `src-tauri/src/tests/ipc_test.rs` - Rust integration tests
- `src-tauri/tests/e2e_test.rs` - End-to-end tests
- `engine/python/tests/test_ipc.py` - Python engine tests
- `engine/python/tests/test_backends.py` - Backend handler tests

#### 6.2 Real Model Testing
**Goal:** Test with actual UVR models, verify inference works.

**Instructions:**
1. Download 1-2 real models from UVR repository
2. Place in `engine/models/` directory
3. Test loading via model registry
4. Test separation on real audio
5. Verify output quality

---

## DEPLOYMENT & DISTRIBUTION

### Build Commands
```bash
# Install dependencies
npm install
cd src-tauri && cargo fetch && cd ..

# Development
npm run tauri dev

# Production build
npm run tauri build

# Output: src-tauri/target/release/bundle/msi/PrismSplit_*_x64_en-US.msi
```

### Installer Configuration
- Embed Python 3.10+ runtime in binary
- Create models directory on first run
- Auto-update capability via Tauri updater
- Windows code signing (if needed)

---

## CODE QUALITY & STANDARDS

### Rust Standards
- All functions documented with rustdoc comments
- No unwrap() in production code (use Result<T>)
- All async functions are cancellable
- Error types implement std::error::Error

### Python Standards
- Type hints on all functions
- Docstrings (Google style)
- No print() statements (use logging)
- All external dependencies in requirements.txt

### TypeScript Standards
- All React components typed with interfaces
- Const assertions for string literals
- No `any` types (use generics)
- All async functions handle errors

### CSS Standards
- CSS variables used for all colors
- No !important (solve with specificity)
- Mobile-first responsive (if applicable)
- Accessibility: WCAG AA contrast ratios minimum

---

## CRITICAL VERIFICATION CHECKLIST

Before each phase completion:

### Phase 1 (Architecture)
- [ ] Rust IPC protocol defined and working
- [ ] Python subprocess spawns and receives messages
- [ ] Job queue persists state
- [ ] No unwrap() in error paths

### Phase 2 (Models)
- [ ] UVR model catalog fetched successfully
- [ ] Model downloads with SHA-256 verification
- [ ] Local directory scan identifies existing models
- [ ] Models stored in correct directory structure

### Phase 3 (Jobs)
- [ ] Jobs queue correctly
- [ ] Progress updates pushed to UI every 1 second
- [ ] Job completion triggers output file verification
- [ ] Failed jobs report meaningful error messages

### Phase 4 (Python)
- [ ] MDX model loads and infers
- [ ] Demucs model loads and infers
- [ ] Audio I/O preserves quality
- [ ] No GPU/CPU crashes on inference

### Phase 5 (UI)
- [ ] All buttons have 3D bevels (no rounded corners)
- [ ] Console output is monospace green (#00ff00)
- [ ] TopBar is fixed 48px with menu items
- [ ] All spacing uses CSS variables

### Phase 6 (Testing)
- [ ] 100% of Rust public APIs tested
- [ ] Python backends tested on real models
- [ ] E2E: Drag file → Inference → Output exists
- [ ] No memory leaks (valgrind / Address Sanitizer)

---

## DOCUMENTATION REQUIREMENTS

### Code Documentation
- Every Rust module has a module-level doc comment
- Every public function has rustdoc with examples
- Every Python class has class-level docstrings
- Every React component has TypeDoc comments

### User Documentation
- README.md with setup instructions
- ARCHITECTURE.md explaining system design
- API.md documenting Tauri command interface
- PROTOCOL.md specifying JSON IPC contract

---

## PERFORMANCE TARGETS

- **App startup:** < 2 seconds
- **Model loading:** < 5 seconds (CUDA) / < 15 seconds (CPU)
- **Inference time:** Depends on model, but real-time feedback
- **Memory usage:** < 2GB RAM with model loaded
- **Disk usage:** ~500MB base + model size

---

## FUTURE ENHANCEMENTS (Post-Alpha)

- [ ] Training module bridge
- [ ] Audio preview player (playback + waveform visualization)
- [ ] Batch processing (process folder)
- [ ] Ensemble mode (combine multiple models)
- [ ] Custom model upload
- [ ] Settings persistence
- [ ] Job history database (SQLite)
- [ ] Multi-language UI

---

## CONCLUSION

This prompt defines **production-grade, non-negotiable standards** for PrismSplit. Every phase builds on verified, tested foundations. No mocks. No shortcuts.

The application will be **trusted by professionals** because it:
1. Uses real inference engines (UVR)
2. Manages models properly (verification, caching, scanning)
3. Provides transparent feedback (console logs, progress, errors)
4. Respects industrial design principles (brutalism, density, honesty)
5. Prioritizes stability (error handling, testing, documentation)

**Execute with precision. Verify at each checkpoint. Ship only when standards are met.**

---

**Version:** 1.0  
**Last Updated:** 2025-05-08  
**Author:** Development Standards Document  
**Target Audience:** Development Team, Code Review, QA
