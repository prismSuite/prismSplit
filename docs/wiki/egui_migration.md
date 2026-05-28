# PrismSplit — Plan de Migración a egui
**De:** Tauri 2.0 + React/TypeScript (WebView)  
**A:** egui (immediate-mode GUI nativo en Rust)  
**Versión del plan:** 1.0 — Mayo 2026

---

## 1. Diagnóstico del estado actual

### Stack actual
| Capa | Tecnología |
|---|---|
| GUI | React 18 + TypeScript + Tailwind CSS (WebView via Tauri) |
| Backend/Shell | Rust (Tauri 2.0) |
| IPC | `tauri::command` (JSON via invoke) |
| Eventos | `tauri::Emitter` → `listen()` en JS |
| Engine | Python subprocess (UVR inference) |

### Componentes frontend identificados
- `App.tsx` — estado global + routing por tabs
- `SetupPanel.tsx` — setup del Python runtime
- `ModelRegistryPanel.tsx` — catálogo de modelos + descarga
- `SeparationPanel.tsx` — ejecución de jobs
- `LogConsole.tsx` — terminal-style log viewer
- `shared.tsx` — componentes UI (Button, Select, Checkbox, Fieldset, NavButton)

### Comandos Tauri (IPC actual → serán funciones Rust directas)
```
get_config / update_config
get_engine_health / prepare_engine
list_model_catalog / download_model / sync_uvr_catalog / scan_local_models
process_audio / cancel_job
```

### Eventos en tiempo real (críticos)
- `download_progress` — progreso de descarga por modelo
- Logs de engine (streamed desde Python subprocess)

---

## 2. Por qué egui (y por qué NO es trivial)

### Ventajas para este proyecto
- **Zero WebView**: elimina dependencia de Edge/WebKit, binario más pequeño (~5-8MB vs ~30MB con Tauri)
- **Immediate mode**: el diseño industrial/denso del DESIGN.md es natural en egui (no hay DOM ni reconciliación)
- **Rust puro**: el backend ya es Rust; la GUI queda en el mismo proceso, el IPC desaparece
- **CRT/skeuomorphic styling**: egui permite custom painters completos; el bevel 3D del DESIGN.md es implementable

### Tradeoffs honestos
- egui no tiene Drag & Drop nativo en Windows tan pulido como la WebView (workaround: `egui-winit` + `winit` DnD events)
- El sistema de diseño CSS del DESIGN.md hay que reescribirlo como `egui::Style` + `epaint` — trabajo real
- No hay `<input type="text">` con placeholder animado; los `TextEdit` de egui son funcionales pero basic

---

## 3. Arquitectura objetivo

```
┌─────────────────────────────────────────┐
│              prismsplit binary           │
│                                          │
│  ┌──────────────┐  ┌───────────────────┐ │
│  │  egui App    │  │   Core Modules    │ │
│  │  (GUI loop)  │◄─►  (sin cambios)   │ │
│  │              │  │                   │ │
│  │  app.rs      │  │  runtime_manager  │ │
│  │  panels/     │  │  model_registry   │ │
│  │  theme.rs    │  │  engine_bridge    │ │
│  │  state.rs    │  │  download_manager │ │
│  └──────────────┘  │  job_manager      │ │
│                    │  app_paths        │ │
│                    │  models           │ │
│                    └───────────────────┘ │
│                                          │
│  ┌────────────────────────────────────┐  │
│  │  Python Engine subprocess (UVR)   │  │
│  └────────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

**Lo que se ELIMINA:** todo `src/` (TypeScript/React), `node_modules`, Tauri IPC layer, WebView renderer  
**Lo que se CONSERVA íntegro:** `src-tauri/src/` excepto `main.rs` (se refactoriza el entry point)

---

## 4. Fases de implementación

---

### Fase 0 — Setup del entorno egui (½ día)

Reemplazar dependencias Tauri en `src-tauri/Cargo.toml`:

```toml
# ELIMINAR
tauri = { version = "2.0", features = [] }
tauri-plugin-shell = "2.0"
tauri-plugin-dialog = "2.0"
tauri-build = { version = "2.0" }  # en [build-dependencies]

# AGREGAR
eframe = { version = "0.28", features = ["persistence"] }
egui = "0.28"
rfd = "0.14"                    # reemplaza tauri-plugin-dialog
tokio = { version = "1", features = ["full"] }
```

> `eframe` es el wrapper de `egui` para desktop (maneja el loop de winit + wgpu/glow).  
> `rfd` (Rust File Dialog) reemplaza `tauri-plugin-dialog` — API casi idéntica.

Mover `src-tauri/` a raíz del repo para simplificar:
```
prismsplit/
  src/           ← NUEVO: Rust egui app (reemplaza src-tauri/src)
  Cargo.toml     ← workspace simplificado
  engine/        ← sin cambios (Python)
  uvr/           ← sin cambios
```

Eliminar `src/` (TypeScript), `index.html`, `vite.config.ts`, `package.json`, `node_modules`, `tsconfig.json`.

---

### Fase 1 — Estado global y threading (1 día)

Crear `src/state.rs` — equivalente al estado de React en `App.tsx`:

```rust
pub struct AppState {
    // Engine
    pub health: Option<EngineHealth>,
    pub is_initializing: bool,
    pub setup_status: Option<SetupStatus>,

    // Config
    pub config: AppConfig,

    // Models
    pub catalog: Vec<ModelCatalogEntry>,
    pub downloading_id: Option<String>,
    pub download_progress: f32,

    // Job
    pub input_file: String,
    pub output_dir: String,
    pub selected_model: String,
    pub quality: String,
    pub export_format: String,
    pub is_processing: bool,

    // Log
    pub log: VecDeque<String>,  // cap 500 líneas

    // UI
    pub active_tab: Tab,
    pub is_dragging: bool,
}

#[derive(PartialEq)]
pub enum Tab { Separate, Models, Settings }
```

**Threading pattern** (reemplaza los `tauri::command` async):

```rust
// Canal para resultados async → GUI thread
pub enum AppMsg {
    HealthResult(EngineHealth),
    CatalogLoaded(Vec<ModelCatalogEntry>),
    DownloadProgress(String, f32),
    DownloadComplete(ModelCatalogEntry),
    ProcessComplete(ProcessAudioResponse),
    LogLine(String),
    Error(String),
}

// En AppState:
pub tx: std::sync::mpsc::Sender<AppMsg>,
pub rx: std::sync::mpsc::Receiver<AppMsg>,
```

En el `update()` loop de egui, al inicio de cada frame se drena `rx` y se aplican los mensajes al estado. Las operaciones async se spawnean con `tokio::spawn` (inicializar runtime Tokio en `main.rs` con `#[tokio::main]`).

---

### Fase 2 — Tema y componentes shared (1 día)

Crear `src/theme.rs` — traducción del DESIGN.md al sistema egui:

```rust
pub fn apply_prismsplit_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut visuals = egui::Visuals::dark();

    // Colores base
    visuals.panel_fill = Color32::from_hex("#1a1a1a").unwrap();
    visuals.window_fill = Color32::from_hex("#0d0d0d").unwrap();
    visuals.extreme_bg_color = Color32::from_hex("#1e1e1e").unwrap();
    visuals.faint_bg_color = Color32::from_hex("#0d0d0d").unwrap();

    // Widgets (equivalente al bevel 3D del DESIGN.md)
    // egui usa bg_fill + weak_bg_fill + stroke para los 3 estados
    visuals.widgets.inactive.bg_fill = Color32::from_hex("#4a4a4a").unwrap();
    visuals.widgets.hovered.bg_fill = Color32::from_hex("#555555").unwrap();
    visuals.widgets.active.bg_fill = Color32::from_hex("#3a3a3a").unwrap();

    // Accent verde CRT
    visuals.selection.bg_fill = Color32::from_hex("#2d5016").unwrap();
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_hex("#00ff00").unwrap());

    // Bordes
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_hex("#666666").unwrap());
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_hex("#777777").unwrap());

    // Sin rounding (brutal, sin border-radius)
    visuals.widgets.inactive.rounding = Rounding::ZERO;
    visuals.widgets.hovered.rounding = Rounding::ZERO;
    visuals.widgets.active.rounding = Rounding::ZERO;
    visuals.window_rounding = Rounding::ZERO;
    visuals.menu_rounding = Rounding::ZERO;

    style.visuals = visuals;
    style.spacing.item_spacing = vec2(8.0, 8.0);
    style.spacing.button_padding = vec2(12.0, 6.0);

    ctx.set_style(style);

    // Fuente Tahoma / monoespaciada
    // egui acepta .ttf embebido o system fonts via egui_extras::install_image_loaders
    // Para Tahoma-like: usar egui's built-in font es suficiente en primera iteración
}
```

Crear `src/widgets.rs` — equivalente de `shared.tsx`:

```rust
// Botón con bevel 3D simulado via Painter
pub fn industrial_button(ui: &mut Ui, label: &str) -> egui::Response { ... }

// Fieldset con legend (egui no tiene nativo, se hace con Frame + Label)
pub fn fieldset<R>(ui: &mut Ui, legend: &str, content: impl FnOnce(&mut Ui) -> R) -> R { ... }

// Checkbox cuadrado (el default de egui es cuadrado, solo theming)
pub fn industrial_checkbox(ui: &mut Ui, label: &str, checked: &mut bool) { ... }

// Select / ComboBox (egui::ComboBox wrapeado)
pub fn industrial_select(ui: &mut Ui, label: &str, current: &mut String, options: &[&str]) { ... }

// NavButton (tab en top bar)
pub fn nav_button(ui: &mut Ui, active: bool, label: &str) -> bool { ... }
```

---

### Fase 3 — Panels (2 días)

Cada panel es un `src/panels/<name>.rs` con una función `show(ui: &mut Ui, state: &mut AppState)`.

#### `panels/setup.rs`
Equivalente de `SetupPanel.tsx`. Muestra health checks y botón "PREPARE ENGINE". On click → `tokio::spawn` que llama `runtime_manager.prepare()` → envía `AppMsg::LogLine` durante el proceso.

#### `panels/model_registry.rs`
Equivalente de `ModelRegistryPanel.tsx`. 
- Tabla de modelos con `egui::ScrollArea` + grid layout
- Progress bar por modelo descargándose (`egui::ProgressBar`)
- Botones SYNC, SCAN, DOWNLOAD por fila
- El `rfd::AsyncFileDialog` reemplaza `openDirDialog`

#### `panels/separation.rs`
Equivalente de `SeparationPanel.tsx` + la sección I/O de `App.tsx`.
- DnD: egui 0.28 tiene soporte básico de DnD via `ui.ctx().input().raw.dropped_files`
- ComboBox para modelo y quality
- Botón START → spawn async job

#### `panels/settings.rs`
Equivalente del tab Settings en `App.tsx`. Config paths + hardware accel selects.

#### `panels/log_console.rs`
Equivalente de `LogConsole.tsx`.
```rust
pub fn show(ui: &mut Ui, logs: &VecDeque<String>) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for line in logs {
                let color = if line.contains("ERROR") {
                    Color32::from_hex("#ff3333").unwrap()
                } else if line.contains("WARN") {
                    Color32::from_hex("#ffcc00").unwrap()
                } else {
                    Color32::from_hex("#00ff00").unwrap()
                };
                ui.colored_label(color, line);
            }
        });
}
```

---

### Fase 4 — App principal y layout (1 día)

Crear `src/app.rs` — implementa `eframe::App`:

```rust
pub struct PrismSplitApp {
    state: AppState,
    rt: Arc<tokio::runtime::Runtime>,  // runtime Tokio para spawns
}

impl eframe::App for PrismSplitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Drena mensajes async
        while let Ok(msg) = self.state.rx.try_recv() {
            self.handle_msg(msg);
        }

        // 2. Top bar (polybar style)
        egui::TopBottomPanel::top("top_bar")
            .exact_height(48.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Logo module
                    // Nav buttons
                    // Status modules a la derecha
                });
            });

        // 3. Bottom console
        egui::TopBottomPanel::bottom("console")
            .exact_height(120.0)
            .show(ctx, |ui| {
                panels::log_console::show(ui, &self.state.log);
            });

        // 4. Status bar
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(20.0)
            .show(ctx, |ui| { ... });

        // 5. Central panel con tabs
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state.active_tab {
                Tab::Separate => panels::separation::show(ui, &mut self.state, &self.rt),
                Tab::Models   => panels::model_registry::show(ui, &mut self.state, &self.rt),
                Tab::Settings => panels::settings::show(ui, &mut self.state, &self.rt),
            }
        });

        // 6. Repaint si hay operación activa
        if self.state.is_processing || self.state.downloading_id.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
}
```

`src/main.rs` simplificado:

```rust
#[tokio::main]
async fn main() {
    let rt = Arc::new(tokio::runtime::Runtime::new().unwrap());
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("PRISMSPLIT // V0.1.0-ALPHA")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_decorations(true),  // usa decoraciones nativas (no custom titlebar)
        ..Default::default()
    };
    eframe::run_native(
        "PrismSplit",
        native_options,
        Box::new(|cc| {
            apply_prismsplit_theme(&cc.egui_ctx);
            Ok(Box::new(PrismSplitApp::new(cc, rt)))
        }),
    ).unwrap();
}
```

---

### Fase 5 — Build y cleanup (½ día)

- Eliminar `build.rs` (era para tauri-build)
- Limpiar `Cargo.toml` del workspace (ya no hay `members = ["src-tauri"]`)
- Verificar que `engine/`, `uvr/` sigan accesibles desde los paths correctos
- `cargo build --release` → binario standalone sin WebView

---

## 5. Mapa de equivalencias IPC → llamadas directas

| Tauri Command | Reemplazado por |
|---|---|
| `get_config` | `load_config(&paths.config_path)` directo |
| `update_config` | `save_config(...)` directo |
| `get_engine_health` | `runtime_manager.doctor().await` |
| `prepare_engine` | `runtime_manager.prepare().await` |
| `list_model_catalog` | `model_registry.load_catalog()` |
| `download_model` | `model_registry` + `download_file_with_progress(...)` |
| `sync_uvr_catalog` | función interna (sin cambios en lógica) |
| `scan_local_models` | función interna (sin cambios) |
| `process_audio` | `engine_bridge.run_command_collect(...)` |
| `cancel_job` | `child.kill().await` directo |
| `openFileDialog` | `rfd::AsyncFileDialog::new().pick_file().await` |
| `openDirDialog` | `rfd::AsyncFileDialog::new().pick_folder().await` |
| evento `download_progress` | `AppMsg::DownloadProgress(id, f32)` via mpsc |

---

## 6. Consideraciones específicas

### Drag & Drop
egui 0.28 expone `ctx.input(|i| i.raw.dropped_files.clone())` en cada frame. Checar si `dropped_files` no está vacío, tomar el path del primero. Funciona en Windows.

### Custom title bar (actualmente NO tiene en App.tsx)
El diseño actual usa el top bar propio pero dentro de la WebView. Con egui, la opción `with_decorations(true)` da la barra nativa de Windows que es aceptable para una herramienta pro. Si se quiere custom, `with_decorations(false)` + implementar drag manual — es trabajo extra, se deja para después.

### CRT scanline effect
En egui se implementa con un custom `Painter` en `CentralPanel`: pintar líneas horizontales semi-transparentes cada 2px sobre todo el panel. Es un postproceso simple.

### Bevel 3D en botones
egui no tiene bevel nativo pero el `Painter` permite dibujar los bordes directamente:
```rust
// Pseudo-code para bevel 3D en custom button
painter.line_segment([tl, tr], Stroke::new(1.0, BORDER_LIGHT));  // top
painter.line_segment([tl, bl], Stroke::new(1.0, BORDER_LIGHT));  // left  
painter.line_segment([bl, br], Stroke::new(1.0, BORDER_DARK));   // bottom
painter.line_segment([tr, br], Stroke::new(1.0, BORDER_DARK));   // right
```

### Fuente Tahoma
egui acepta fuentes custom via `FontData`. Se puede embeber Tahoma o una fuente similar (Liberation Sans Narrow, o la misma IBM Plex Sans del DESIGN.md) como bytes en el binario:
```rust
fonts.font_data.insert("tahoma".to_owned(), egui::FontData::from_static(include_bytes!("../assets/Tahoma.ttf")));
```

---

## 7. Estimado de esfuerzo

| Fase | Trabajo | Días |
|---|---|---|
| 0 — Setup egui | Dependencias + estructura | 0.5 |
| 1 — State + threading | state.rs + AppMsg pattern | 1.0 |
| 2 — Theme + widgets | theme.rs + widgets.rs | 1.0 |
| 3 — Panels | 5 panels | 2.0 |
| 4 — App layout | app.rs + main.rs | 1.0 |
| 5 — Build + cleanup | Cleanup + QA básico | 0.5 |
| **Total** | | **~6 días** |

---

## 8. Riesgos y mitigaciones

| Riesgo | Probabilidad | Mitigación |
|---|---|---|
| DnD no funciona bien en Windows con egui | Media | rfd tiene un picker como fallback; DnD es nice-to-have |
| Bevel 3D en botones queda raro | Baja | El `Painter` custom es determinístico; se itera rápido |
| Fuente Tahoma no embebible (licencia) | Media | Usar IBM Plex Sans (OFL) que ya está en DESIGN.md |
| Performance con log de 500 líneas | Baja | `ScrollArea` de egui es lazy; solo renderiza lo visible |
| Tokio runtime en eframe | Baja | Patrón `#[tokio::main]` + `Arc<Runtime>` es standard |

---

## 9. Estructura final del repo

```
prismsplit/
├── Cargo.toml              (workspace con un solo member)
├── Cargo.lock
├── src/
│   ├── main.rs             (entry point, eframe::run_native)
│   ├── app.rs              (PrismSplitApp implements eframe::App)
│   ├── state.rs            (AppState, AppMsg, Tab)
│   ├── theme.rs            (apply_prismsplit_theme)
│   ├── widgets.rs          (industrial_button, fieldset, etc.)
│   ├── panels/
│   │   ├── mod.rs
│   │   ├── setup.rs
│   │   ├── model_registry.rs
│   │   ├── separation.rs
│   │   ├── settings.rs
│   │   └── log_console.rs
│   ├── app_paths.rs        (sin cambios)
│   ├── download_manager.rs (sin cambios)
│   ├── engine_bridge.rs    (sin cambios)
│   ├── job_manager.rs      (sin cambios)
│   ├── model_registry.rs   (sin cambios)
│   ├── models.rs           (sin cambios — quitar derives de serde_json en EngineHealth si tenían Tauri State)
│   └── runtime_manager.rs  (sin cambios)
├── assets/
│   └── IBMPlexSans-Regular.ttf
├── engine/                 (sin cambios)
├── uvr/                    (sin cambios)
└── DESIGN.md, README.md, etc.
```

**Archivos a eliminar del repo:**
```
src-tauri/         (reemplazado por src/)
src/               (el TypeScript)
index.html
vite.config.ts
package.json
package-lock.json
node_modules/
tsconfig.json
```

---

*Plan generado post-análisis directo del codebase. Los módulos Rust core no requieren modificación — la migración es exclusivamente de capa de presentación.*
