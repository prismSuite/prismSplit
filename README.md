<div align="center">
  <img src="src/assets/logo.svg" width="600" alt="prismSplit Logo" />
  
  <p><strong>Industrial-Grade Audio Separation & Stem Isolation Platform</strong></p>
  
  <p>
    <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/Tauri-v2.0-24C8DB?style=flat-square&logo=tauri&logoColor=white" alt="Tauri" /></a>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-Stable-orange?style=flat-square&logo=rust&logoColor=white" alt="Rust" /></a>
    <a href="https://www.python.org/"><img src="https://img.shields.io/badge/Engine-Python%203.10-blue?style=flat-square&logo=python&logoColor=white" alt="Python" /></a>
    <a href="https://onnxruntime.ai/"><img src="https://img.shields.io/badge/Inference-ONNX%20%2F%20PyTorch-475569?style=flat-square&logo=onnx&logoColor=white" alt="Inference" /></a>
  </p>
</div>

---

## 📖 Descripción General / Overview

**prismSplit** es una plataforma de escritorio de grado industrial y alto rendimiento diseñada para la **separación especializada de audio y el aislamiento de stems** (voces, instrumentales, batería, bajo y otros componentes armónicos).

Como parte de la marca federada **prismSuite**, la aplicación adopta la filosofía visual de **Dark Brutalism Skeuomórfico (MonolithUI)**: un chasis con biseles metálicos, tornillería real y un osciloscopio espectral de precisión en tiempo real. 

La aplicación integra de forma aislada e independiente su propio entorno de inferencia de Python y los algoritmos del proyecto **Ultimate Vocal Remover (UVR)**, ofreciendo a los productores musicales un procesamiento local sin fricciones y libre de dependencias globales ("Dependency Hell").

---

## 🏗️ Arquitectura de Procesamiento / System Architecture

prismSplit divide su flujo de trabajo en dos capas de alta especialización para un rendimiento máximo:

1.  **Orquestación (Rust + Tauri v2):** Controla el ciclo de vida del software, supervisión de procesos paralelos, descargas concurrentes de modelos con barra de progreso precisa, ruteo asíncrono de buffers de audio y chasis del frontend.
2.  **Motor de Inferencia (Python Runner):** Un subproceso nativo desacoplado que ejecuta las complejas redes de separación utilizando librerías optimizadas de PyTorch y ONNX Runtime. La comunicación se realiza mediante flujos estructurados de JSON de baja latencia sobre `stdio`.

```text
  [ Vite / React Frontend ] 
             │
      ( Tauri IPC Commands )
             │
   [ Rust Orchestration Core ]  ◄── (Detección Mutua de prismConsole)
             │
   ( Stdio JSON Protocol )
             │
  [ Embedded Python Inference ] ──► [ ONNX Runtime / CUDA GPU ]
```

---

## ⚡ Superpoderes / Key Features

*   🎛️ **Multi-Engine Integrado:** Soporte nativo para modelos **MDX-Net**, **VR Architecture**, **Demucs (v1-v4)** y **Roformer** de espectro completo.
*   🔄 **Sincronización Automática de Catálogo:** Consulta y descarga modelos con SHA-256 verificado en un clic directamente desde los servidores de UVR.
*   💾 **Scan Local Zero-Copy:** Detección de modelos preexistentes en tu disco mediante hashes MD5. Los utiliza en su ubicación original sin duplicar datos en el disco.
*   🔋 **Entorno Python Integrado:** Inicialización e instalación automatizada del motor en el primer arranque ("Prepare Engine") dentro del sandbox del usuario.
*   🚀 **Aceleración por Hardware:** Soporte completo de GPU para tarjetas NVIDIA (CUDA), AMD/Intel (DirectML) y CPUs optimizadas multi-núcleo.
*   ⛓️ **Suite Link Integrado:** Detección automática en Rust de su aplicación hermana **prismConsole**, permitiendo un salto rápido o promoviendo el orquestador brutalista IA de agentes de estudio.

---

## 🚀 Instalación y Construcción / Installation & Build

### Requisitos Previos
*   **Windows 10 / 11** (Sistema de desarrollo principal de Jules)
*   **Node.js** (v18+)
*   **Rust** (Stable)

### Pasos
1.  **Clonar el Repositorio:**
    ```bash
    git clone https://github.com/julesklord/prismsplit.git
    cd prismsplit
    ```
2.  **Instalar Dependencias de UI:**
    ```bash
    npm install
    ```
3.  **Iniciar Entorno en Desarrollo:**
    Inicia la ventana de Tauri y levanta el compilador asíncrono simultáneamente:
    ```bash
    npm run dev
    ```
4.  **Compilar Distribución de Producción:**
    Compila el instalador independiente optimizado con todos los iconos nativos del chasis:
    ```bash
    npm run build
    ```

---

## 🛠️ Estándares Visuales / Design Guidelines

*   **Contraste de Marca:** Mantiene la firma visual oficial de la suite: la palabra `prism` en tipografía Serif Italic analógica (`Playfair Display`) y el nombre del módulo `Split` en sans-serif de precisión (`Outfit`).
*   **Chapa de Stems:** El visualizador espectral refleja fielmente las ondas divididas en stems con colores HSL personalizados (Cian, Oro y Esmeralda) con resplandor neón sutil sobre un chasis de aluminio anodizado verde oscuro (`#111815` a `#030604`).
*   **Tornillería Física:** El chasis de la interfaz de usuario en Tauri simula el hardware de montaje clásico con 4 tornillos de acero inoxidable expuestos y etiquetas de calibración grabadas.

---
<div align="center">
  <sub>prismSuite — Diseñado con precisión brutalista analógica y alto rendimiento. REV 2.5</sub>
</div>
