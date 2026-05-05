import { invoke } from "@tauri-apps/api/core";
import type { EngineHealth, SetupStatus, ModelCatalogEntry } from "./types";

// Helper for check
export const isTauri = () => {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
};

export async function minimizeWindow() {
  if (isTauri()) {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      getCurrentWindow().minimize();
    } catch (e) {}
  }
}

export async function toggleMaximizeWindow() {
  if (isTauri()) {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      getCurrentWindow().toggleMaximize();
    } catch (e) {}
  }
}

export async function closeWindow() {
  if (isTauri()) {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      getCurrentWindow().close();
    } catch (e) {}
  }
}

export async function openFileDialog(): Promise<string | null> {
  if (isTauri()) {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: false,
      });
      return selected as string | null;
    } catch (e) {
      console.warn("Tauri dialog failed, using prompt fallback", e);
      return prompt("Enter file path:");
    }
  } else {
    return prompt("Enter simulated path:");
  }
}

export async function openDirDialog(): Promise<string | null> {
  if (isTauri()) {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        directory: true,
        multiple: false,
      });
      return selected as string | null;
    } catch (e) {
      console.warn("Tauri dialog failed, using prompt fallback", e);
      return prompt("Enter directory path:");
    }
  } else {
    return prompt("Enter simulated directory path:");
  }
}

export async function processAudio(
  filePath: string,
  model: string,
  outputDir: string,
  quality: string,
): Promise<string> {
  if (isTauri()) {
    return await invoke("process_audio", {
      filePath,
      model,
      outputDir,
      quality,
    });
  } else {
    // Mock for web preview
    return new Promise((resolve) => {
      setTimeout(() => {
        resolve(
          `(Mock) Procesamiento completado de:\n${filePath}\nusando ${model} a calidad ${quality}.\nResultados en ${outputDir}`,
        );
      }, 3000);
    });
  }
}

export async function getAvailableModels(): Promise<string[]> {
  if (isTauri()) {
    return await invoke("get_available_models");
  } else {
    return ["Demucs v4 (htdemucs)", "MDX-Net (UVR-MDX-NET)", "VR Architecture"];
  }
}

export async function getEngineHealth(): Promise<EngineHealth> {
  if (isTauri()) {
    return await invoke("get_engine_health");
  }
  return {
    runtimeReady: false,
    dependenciesReady: false,
    ffmpegReady: false,
    modelCatalogReady: false,
    installedModelCount: 0,
    activeJobCount: 0,
  };
}

export async function prepareEngine(): Promise<SetupStatus> {
  if (isTauri()) {
    return await invoke("prepare_engine");
  }
  return {
    ready: true,
    currentStage: null,
    completedStages: ["mock_setup"],
    lastError: null,
  };
}
