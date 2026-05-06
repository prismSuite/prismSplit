// src/lib/types.ts
export type SetupStatus = {
  ready: boolean;
  currentStage: string | null;
  completedStages: string[];
  lastError: string | null;
};

export type EngineHealth = {
  runtimeReady: boolean;
  dependenciesReady: boolean;
  ffmpegReady: boolean;
  modelCatalogReady: boolean;
  installedModelCount: number;
  activeJobCount: number;
  gpuDevices: string[];
};

export type ModelCatalogEntry = {
  id: string;
  name: string;
  backend: string;
  outputKind: string;
  url: string;
  sha256: string;
  sizeBytes: number;
  filename: string;
  version: string;
};

export type SeparationRequest = {
  inputPath: string;
  modelId: string;
  outputDir: string;
  format: string;
};

export type ProcessAudioResponse = {
  jobId: string;
  vocalsPath: string;
  instrumentalPath: string;
  backend: string;
};

export type DownloadProgressEvent = {
  modelId: string;
  progress: number;
};
