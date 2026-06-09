# PrismSplit — Auditoría General del Proyecto

> **Repositorio:** `prismSplit`  
> **Tipo:** Aplicación nativa de separación de audio (stems)  
> **Stack:** Rust (egui/eframe) + Python (UVR/ONNX/PyTorch)  
> **Hash actual:** `33e04a6`  
> **Fecha de auditoría:** 2026-06-09

---

## 1. Descripción General

**PrismSplit** es una aplicación de escritorio nativa para la separación de audio en stems (voz + instrumental), parte del ecosistema **prismSuite**. La aplicación sigue un patrón de arquitectura híbrida:

- **Frontend / Orquestación:** Rust con egui/eframe (UI nativa GPU-acelerada)
- **Engine de inferencia:** Python embebido con modelos UVR (Ultimate Vocal Remover)
- **Comunicación:** Protocolo JSON línea a línea sobre stdin/stdout

### Arquitectura de alto nivel

```
┌─────────────────────────────────────────────────────────────┐
│                    PrismSplit App (Rust)                    │
│  ┌─────────────┐  ┌─────────────┐  ┌───────────────────────┐ │
│  │  egui UI   │  │   Backend   │  │   RuntimeManager      │ │
│  │  (app.rs)  │  │ (backend.rs)│  │ (runtime_manager.rs) │ │
│  └─────┬──────┘  └──────┬──────┘  └───────────┬───────────┘ │
│        │                │                      │             │
│        └────────────────┼──────────────────────┘             │
│                         │                                     │
│  ┌──────────────────────┴───────────────────────┐            │
│  │        EngineBridge (JSON/stdio)             │            │
│  └──────────────────────┬───────────────────────┘            │
└─────────────────────────┼─────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│          Python Inference Engine (engine/python)             │
│  ┌─────────────┐  ┌─────────────┐  ┌───────────────────────┐ │
│  │  MDX-Net   │  │   Demucs    │  │   UVR Vendored        │ │
│  │  (ONNX)    │  │  (PyTorch)  │  │   (lib_v5, demucs)   │ │
│  └─────────────┘  └─────────────┘  └───────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Estructura del Proyecto

### 2.1. Directorios principales

| Directorio | Contenido | Lenguaje |
|------------|-----------|----------|
| `src/` | Código fuente Rust (UI, backend, orquestación) | Rust |
| `src/panels/` | Submódulos de UI (log console) | Rust |
| `engine/` | Engine Python de inferencia + catálogo de modelos | Python |
| `engine/python/` | Scripts Python (engine, backends, protocolo) | Python |
| `uvr/` | Librerías UVR vendored (lib_v5, demucs, modelos) | Python |
| `tests/` | Tests de integración Rust | Rust |
| `docs/` | Documentación del proyecto (Gemini, Agent, etc.) | Markdown |
| `assets/` | Recursos gráficos (logo SVG) | SVG/PNG |

### 2.2. Conteo de archivos por categoría

- **Rust source:** 18 archivos (`src/` + `tests/`)
- **Python source:** ~15 archivos (`engine/python/`, backends)
- **UVR vendored:** ~40+ archivos Python (lib_v5, demucs, modelos)
- **Config/Data:** `Cargo.toml`, `engine/pyproject.toml`, `engine/models/catalog.json`
- **Docs:** 4 archivos Markdown

---

## 3. Análisis de Calidad de Código (Rust)

### 3.1. Compilación

✅ **El proyecto compila correctamente**

```bash
$ cargo check
    Finished `dev` profile [unoptimized + debuginfo] in 0.37s
```

Sin errores de compilación.

### 3.2. Análisis con Clippy

⚠️ **3 warnings en librería, 3 en tests** (no críticos):

```
warning: the borrowed expression implements the required traits
  --> src/app.rs:893 (needless_borrows_for_generic_args)

warning: this creates an owned instance just for comparison
  --> src/runtime_manager.rs:371 (cmp_owned)

warning: you should consider adding a `Default` implementation for `PlaybackController`
  --> src/preview.rs:59 (new_without_default)
```

**Tests `engine_bridge.rs`:** 3 placeholders (`assert!(true)`) que no ejecutan lógica real.

**Impacto:** Bajo. Son warnings menores de estilo y tests sin funcionalidad.

### 3.3. Complejidad ciclomática estimada

| Módulo | Complejidad | Notas |
|--------|-------------|-------|
| `app.rs` | Alta (900+ líneas) | Mezcla de UI, estado y lógica de negocio. Buen uso de `drain_messages` para manejo de eventos, pero `render_preview_window` es extenso (200+ líneas). |
| `backend.rs` | Media-Alta | Coordinación de múltiples subsistemas (download, modelos, procesamiento). |
| `runtime_manager.rs` | Media | Lógica de setup/repair del engine Python con manejo de errores robusto. |
| `engine_bridge.rs` | Baja-Media | Protocolo simple stdin/stdout. Buena separación de concerns. |

---

## 4. Análisis Fortaleza de Arquitectura

### 4.1. ✅ Fortalezas

| Aspecto | Evaluación |
|---------|-----------|
| **Separación Rust/Python** | Excelente. El engine es un subprocesso aislado con protocolo JSON definido. |
| **Modelo de concurrencia** | Correcto uso de `tokio` para operaciones IO + `mpsc` para comunicación UI-backend. |
| **Gestión de estado** | Centralizado en `AppState` con mensajes `AppMsg`. Patrón actor-like limpio. |
| **Autoreparación del runtime** | `RuntimeManager.smart_repair()` puede diagnosticar y reparar dependencias rotas. |
| **Sistema de paths** | `AppPaths` abstrae rutas de desarrollo vs. producción vs. portable. |
| **Backend de inferencia extensible** | `BackendBase` + backends concretos (MDX, Demucs). Fácil añadir más. |
| **Preview de audio** | Análisis de picos + reproducción con `rodio`. Experiencia de usuario rica. |
| **Multiplataforma** | Soporte Windows/macOS/Linux con detección de GPU específico por SO. |

### 4.2. ⚠️ Áreas de Mejora Identificadas

#### A. **Acoplamiento UI-lógica en `app.rs`**

El archivo `app.rs` (~1200 líneas) mezcla rendering de UI con lógica de aplicación. Se recomienda extraer:

- Lógica de preview a módulo separado
- Renderizado de cada pestaña (Separate, Models, Settings) a componentes dedicados
- Configuración de la interfaz a un archivo de diseño separado

**Recomendación:** Considerar un patrón MVP (Model-View-Presenter) o MVVM para desacoplar.

#### B. **Tests de integración vacíos**

`tests/engine_bridge.rs` tiene 3 tests que son placeholders (`assert!(true)`). Estos tests no aportan valor y consumen tiempo de CI.

**Recomendación:** Implementar mocks del engine Python o tests de integración reales con fixtures.

#### C. **Manejo de errores en Python engine**

El engine Python usa excepciones genéricas (`except Exception`) sin logging ni debugging info:

```python
try:
    from uvr_utils import ensure_uvr_in_sys_path
except Exception:
    ensure_uvr_in_sys_path = None  # type: ignore
```

**Recomendación:** Añadir logging estructurado en el engine Python y exponer logs de vuelta a Rust.

#### D. **Hardcoded strings y magics**

Valores hardcoded dispersos:

```rust
// app.rs
"replace-with-real-sha256"  // En backend.rs para modelos sin hash
"Fast (CPU)"               // En state.rs - niveles de calidad
"Normal (CUDA)"
"High Quality (Overlap)"
"Extreme (Aggressive Math)"
```

**Recomendación:** Centralizar en constantes o enum del tipo `QualityPreset`.

#### E. **Proceso de sincronización de catálogo (`sync_uvr_catalog`)**

Realiza 10 requests HTTP concurrentes sin rate limiting ni retry. Si la API estuviera limitada, podría fallar o ser bloqueada.

**Recomendación:** Añadir retry con backoff exponencial y manejo de rate limits.

#### F. **Potencial deadlocks con `std::sync::Mutex`**

ModelRegistry usa `std::sync::Mutex` para `models_dir`. Si en el futuro se usara en contexto async, podría causar problemas.

**Recomendación:** Considerar `tokio::sync::RwLock` o `std::sync::RwLock` si la escritura es infrecuente.

#### G. **Gestión de memoria en preview de audio**

`analyze_audio_peaks` carga el archivo completo en memoria:

```rust
let samples: Vec<f32> = decoder
    .map(|sample| (sample as f32) / (i16::MAX as f32))
    .map(|s| s.abs())
    .collect();
```

Para archivos de audio largos, esto podría consumir mucha memoria.

**Recomendación:** Procesar en chunks streaming.

---

## 5. Seguridad

### 5.1. ✅ Prácticas seguras

- **Verificación SHA-256** de modelos descargados (`download_manager::verify_sha256`)
- **Ejecución sandboxed** del engine Python como subprocesso separado
- **No ejecución de código arbitrario** en el protocolo stdin/stdout

### 5.2. ⚠️ Consideraciones de seguridad

| Problema | Severidad | Descripción |
|----------|-----------|-------------|
| **Downloads HTTP sin verificación TLS** | Media | `reqwest` usa rustls-tls (correcto). Verificar que `http1_only` no debilita seguridad. |
| **Exposición de stderr de Python** | Baja | `stderr(Stdio::inherit)` filtra errores de Python al terminal. En producción, considerar captura. |
| **Hash SHA-256 placeholder** | Media | `"replace-with-real-sha256"` es bypassed por `verify_sha256`. Modelos de UVR sin hash real verificable. |
| **Path traversal potencial** | Baja | `output_dir` y `input_file` de usuario no se sanitizan antes de ser pasados a filesystem. |

---

## 6. Rendimiento

### 6.1. Hotspots potenciales

1. **`analyze_audio_peaks`** — Carga completa de archivo de audio en memoria
2. **`process_audio`** — Inference síncrona bloquea el thread de async, aunque está en `tokio::spawn`
3. **` sync_uvr_catalog`** — 10 requests concurrentes sin limitación
4. **GPU detection** — Ejecuta `wmic`, `nvidia-smi`, o `lspci` en cada `doctor()` llama. Podría cachearse.

### 6.2. Optimizaciones recomendadas

| Optimización | Impacto | Esfuerzo |
|-------------|---------|----------|
| Cachear resultado de GPU detection | Medio | Bajo |
| Streaming de análisis de audio en preview | Alto | Medio |
| Paginación del catálogo de modelos | Medio | Bajo |
| Recycling de procesos Python | Alto | Alto |

---

## 7. Dependencias y Mantenimiento

### 7.1. Rust (Cargo.toml)

```toml
serde = "1.0"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
eframe = "0.28"
egui = "0.28"
rodio = "0.19"
reqwest = "0.12"
```

**Nivel de madurez:** Mature. `eframe`/`egui` v0.28 es estable. `anyhow`/`thiserror` son estándar.

**Observación:** `reqwest` usa default-features en lugar de deshabilitar. Podría reducirse el tamaño del binario.

### 7.2. Python (engine/pyproject.toml)

```toml
dependencies = [
    "oniquitous",
    "torch>=2.0.0",
    "torchaudio>=2.0.0",
    "onnxruntime>=1.14.1",
    "numpy==1.23.5",        # Pin estricto - puede causar conflictos
    "soundfile==0.11.0",
    "librosa==0.9.2",
    "requests>=2.25.1"
]
```

**Observaciones:**
- **numpy 1.23.5** es una versión antigua (actual es 2.x). Enlace con versiones modernas de numpy puede haber resuelto conflictos.
- Capas **torch**/**torchaudio** son pesadas (~2GB). El proceso de empaquetado requiere incluir estas dependencias.
- El runtime requiere **ffmpeg** para decodificación de audio.

---

## 8. Historial de Git y Evolución

### Commits recientes (últimos 20)

```
33e04a6 Update README for egui/eframe UI and docs
97df21c Add macOS/Linux GPU detection and MPS support
b2a75df Rename analyze_wav_peaks to analyze_audio_peaks
0cb3735 Update docs for Rust/egui migration
b5166be Add release workflow and packager config
7175e58 chore: update .gitignore with standard patterns
fff6de6 docs: apply stop-slop rules to reformat documentation
2b4ca55 refactor: migrate UI to egui, remove frontend web assets
```

### Análisis de evolución

- **Commit `2b4ca55`** — Migración completa de frontend web a egui/eframe (Tauri anterior)
- **Commit `97df21c`** — Añadido soporte multiplataforma para GPU (CUDA, MPS, CoreML)
- **Tendencia:** Desacoplismo de frontend web en favor de UI nativa para mejor rendimiento y reducir dependencias

---

## 9. Coverage de Tests

| Módulo | Estado | Cobertura estimada |
|--------|--------|-------------------|
| `models.rs` | ✅ Con tests unitarios | 80% |
| `model_registry.rs` | ✅ Tests de integración | 60% |
| `download_manager.rs` | ✅ Tests básicos | 40% |
| `runtime_manager.rs` | ⚠️ Tests básicos sin mocking | 30% |
| `engine_bridge.rs` | ❌ Placeholders | 5% |
| `backend.rs` | ❌ No hay tests de integración | 10% |
| `app.rs` | ❌ No hay tests UI | 0% |
| Engine Python | ❌ Solo tests estructurales | 20% |

**Tests Python:**
- `engine/python/tests/test_entrypoint.py`
- `engine/python/tests/test_protocol.py`

Ambos son tests básicos de estructura. No hay tests de inferencia real.

---

## 10. Licencias y Legales

- **UVR (Ultimate Vocal Remover):** Contiene código de terceros con sus propias licencias
- **Modelos ONNX/PyTorch:** Licenciamiento específico por modelo
- **Logo/assets:** SVG propio del proyecto
- **No se detectó** `LICENSE` file en la raíz del repositorio

---

## 11. Lista de Acciones Recomendadas

### 🔴 Crítico (Bloqueante para producción)

1. **Implementar tests de integración del engine** — La mayoría de tests son placeholders
2. **Añadir `LICENSE`** al repositorio
3. **Revisar y fijar dependencias Python** con conflictos conocidos (numpy 1.23.5 es muy antiguo)
4. **Documentar requirements de runtime** (versión exacta de Python, PyTorch, CUDA drivers)

### 🟡 Importante (Calidad)

5. **Refactorizar `app.rs`** — Extraer componentes UI a módulos separados
6. **Implementar caching de GPU info** en `RuntimeManager`
7. **Añadir retry y rate limiting** en `sync_uvr_catalog`
8. **Optimizar `analyze_audio_peaks`** para streaming de audio grande
9. **Centralizar constantes de strings** (quality presets, etc.)
10. **Añadir logging estructurado** en engine Python
11. **Revisar tests `engine_bridge.rs`** — Eliminar placeholders o implementar mocks

### 🟢 Deseable (Mantenimiento)

12. **Actualizar `egui`/`eframe`** a la última versión estable (si hay breaking changes)
13. **Reducir features de `reqwest`** para bajar tamaño del binario
14. **Implementar `Default` para `PlaybackController`** (Clippy sugerencia)
15. **Añadir linter Python** (ruff, black) a CI/CD
16. **Documentar protocolo JSON** entre Rust y Python engine

---

## 12. Conclusión

**PrismSplit** es una aplicación bien arquitectada que sigue buenas prácticas de separación de concerns (Rust UI / Python Engine). El **patrón de arquitectura híbrida** es su mayor fortaleza, permitiendo un frontend nativo de alto rendimiento con un engine de ML aislado.

### Estado general: **Bueno con mejoras necesarias**

- **Arquitectura:** ⭐⭐⭐⭐⭐ (5/5)
- **Calidad de código Rust:** ⭐⭐⭐⭐☆ (4/5) — Warnings menores, pero buena estructura
- **Calidad de código Python:** ⭐⭐⭐☆☆ (3/5) — Básico pero funcional
- **Cobertura de tests:** ⭐⭐☆☆☆ (2/5) — Tests insuficientes, muchos placeholders
- **Documentación:** ⭐⭐⭐⭐☆ (4/5) — Buena documentación externa, pero falta interna
- **Seguridad:** ⭐⭐⭐⭐☆ (4/5) — En general bien, con áreas a pulir
- **Rendimiento:** ⭐⭐⭐⭐☆ (4/5) — Mejoras de caching y streaming pendientes

**Veredicto:** El proyecto está **marcha** y es funcional. Para un release de producción, priorizar: tests de integración, refactorización de `app.rs`, y optimización de memoria en preview de audio.

---

> *Auditoría generada automáticamente. Consultar el código fuente para detalles adicionales.*
