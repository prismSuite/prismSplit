import React, { useState, useEffect, useRef } from "react";
import {
  processAudio,
  minimizeWindow,
  toggleMaximizeWindow,
  closeWindow,
  openFileDialog,
  openDirDialog,
  getEngineHealth,
  prepareEngine,
  listModelCatalog,
  downloadModel,
  syncUvrCatalog,
  scanLocalModels,
  getConfig,
  updateConfig,
} from "./lib/api";
import { listen } from "@tauri-apps/api/event";
import type {
  EngineHealth,
  SetupStatus,
  ModelCatalogEntry,
  ProcessAudioResponse,
  DownloadProgressEvent,
} from "./lib/types";
import { SetupPanel } from "./components/SetupPanel";
import { ModelRegistryPanel } from "./components/ModelRegistryPanel";
import { SeparationPanel } from "./components/SeparationPanel";
import { LogConsole } from "./components/LogConsole";
import {
  NavButton,
  Fieldset,
  Button,
  HelpIcon,
  Checkbox,
  Select,
} from "./components/shared";

export default function App() {
  const [activeTab, setActiveTab] = useState("separate");
  const [catalog, setCatalog] = useState<ModelCatalogEntry[]>([]);

  // Engine state
  const [health, setHealth] = useState<EngineHealth | null>(null);
  const [setupStatus, setSetupStatus] = useState<SetupStatus | null>(null);
  const [isInitializing, setIsInitializing] = useState(true);
  const [appConfig, setAppConfig] = useState({ modelsDir: "", cacheDir: "" });

  // Form state
  const [inputFile, setInputFile] = useState<string>("");
  const [outputDir, setOutputDir] = useState<string>("");
  const [selectedModel, setSelectedModel] = useState<string>("");
  const [quality, setQuality] = useState<string>("Normal");
  const [exportFormat, setExportFormat] = useState<string>("WAV");

  // Processing state
  const [isProcessing, setIsProcessing] = useState(false);
  const [isPreviewing, setIsPreviewing] = useState(false);
  const [log, setLog] = useState<string[]>([
    "PrismSplit Core Sub-System V-0.1.0-alpha loaded.",
    "System ready.",
    "Initializing hardware scan...",
  ]);

  const [isDragging, setIsDragging] = useState(false);
  const [downloadingId, setDownloadingId] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState<number>(0);

  const logEndRef = useRef<HTMLDivElement>(null);
  const downloadIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const processIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const previewTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    const unlisten = listen<DownloadProgressEvent>(
      "download_progress",
      (event) => {
        setDownloadProgress(event.payload.progress);
      },
    );

    return () => {
      unlisten.then((u) => u());
    };
  }, []);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "instant" });
  }, [log]);

  useEffect(() => {
    return () => {
      if (downloadIntervalRef.current)
        clearInterval(downloadIntervalRef.current);
      if (processIntervalRef.current) clearInterval(processIntervalRef.current);
      if (previewTimeoutRef.current) clearTimeout(previewTimeoutRef.current);
    };
  }, []);

  useEffect(() => {
    refreshHealth();
    loadAppConfig();
  }, []);

  const loadAppConfig = async () => {
    try {
      const cfg = await getConfig();
      setAppConfig({
        modelsDir: cfg.modelsDir || "",
        cacheDir: cfg.cacheDir || "",
      });
    } catch (e) {
      addLog(`ERROR: Failed to load app config: ${e}`);
    }
  };

  const handleApplySettings = async () => {
    try {
      await updateConfig(appConfig);
      addLog("Settings applied. Some changes require restart.");
      await refreshHealth();
    } catch (e) {
      addLog(`ERROR: Failed to apply settings: ${e}`);
    }
  };

  const handleBrowseModels = async () => {
    const path = await openDirDialog();
    if (path) {
      setAppConfig((prev) => ({ ...prev, modelsDir: path }));
    }
  };

  const handleBrowseCache = async () => {
    const path = await openDirDialog();
    if (path) {
      setAppConfig((prev) => ({ ...prev, cacheDir: path }));
    }
  };

  const refreshHealth = async () => {
    try {
      const h = await getEngineHealth();
      setHealth(h);
      if (h.runtimeReady && h.dependenciesReady && h.modelCatalogReady) {
        await loadCatalog();
      }
    } catch (e) {
      addLog(`ERROR: Health check failed: ${e}`);
    } finally {
      setIsInitializing(false);
    }
  };

  const loadCatalog = async () => {
    try {
      const entries = await listModelCatalog();
      setCatalog(entries);
      if (entries.length > 0) {
        setSelectedModel((prev) => prev || entries[0].id);
      }
      addLog(`Models loaded: ${entries.length} found in registry.`);
    } catch (e) {
      addLog(`ERROR: Failed to load model catalog [${e}]`);
    }
  };

  const handlePrepareEngine = async () => {
    try {
      const status = await prepareEngine();
      setSetupStatus(status);
      if (status.ready) {
        await refreshHealth();
      }
    } catch (e: any) {
      addLog(`ERROR: Setup failed: ${e}`);
    }
  };

  const addLog = (msg: string) => {
    setLog((prev) => {
      const newLogs = [
        ...prev,
        `[${new Date().toLocaleTimeString("en-US", { hour12: false })}] ${msg}`,
      ];
      return newLogs.length > 500
        ? newLogs.slice(newLogs.length - 500)
        : newLogs;
    });
  };

  const handleDownloadModel = async (modelId: string) => {
    setDownloadingId(modelId);
    setDownloadProgress(0);
    addLog(
      `INIT: Download started for model [${modelId}]... this may take a while.`,
    );

    try {
      const model = await downloadModel(modelId);
      addLog(`SUCCESS: Model [${model.name}] installed and verified.`);
      await loadCatalog();
    } catch (e) {
      addLog(`ERROR: Model download failed [${e}]`);
    } finally {
      setDownloadingId(null);
      setDownloadProgress(0);
    }
  };

  const handleSyncCatalog = async () => {
    addLog("INIT: Synchronizing model catalog with UVR servers...");
    try {
      const added = await syncUvrCatalog();
      addLog(
        `SUCCESS: Catalog synchronized. ${added} new models added to registry.`,
      );
      await loadCatalog();
    } catch (e) {
      addLog(`ERROR: Catalog sync failed [${e}]`);
    }
  };

  const handleScanModels = async () => {
    const path = await openDirDialog();
    if (!path) return;

    addLog(`INIT: Scanning directory [${path}] for known audio models...`);
    try {
      const added = await scanLocalModels(path);
      addLog(
        `SUCCESS: Scan complete. ${added} new local models identified and registered.`,
      );
      await loadCatalog();
    } catch (e) {
      addLog(`ERROR: Scan failed [${e}]`);
    }
  };

  const handlePreview = async () => {
    addLog("ERROR: Preview generation is not implemented in Release 1.");
  };

  const handleProcess = async () => {
    if (!inputFile) {
      addLog("WARN: Null input file exception. Operation aborted.");
      return;
    }
    setIsProcessing(true);
    addLog(
      `INIT: Process task created for <${inputFile.split("/").pop() || inputFile}>. Waiting for engine...`,
    );

    try {
      const result: ProcessAudioResponse = await processAudio(
        inputFile,
        selectedModel,
        outputDir || "C:\\AudioData\\Output",
        quality,
      );
      addLog(
        `SUCCESS: Separation complete via ${result.backend}. Vocals: ${result.vocalsPath} | Instrumental: ${result.instrumentalPath}`,
      );
    } catch (e: any) {
      addLog(`ERR: Execution halted: ${e}`);
    } finally {
      setIsProcessing(false);
    }
  };

  // Conditional view: Setup if engine not ready
  if (isInitializing) {
    return (
      <div className="app-container items-center justify-center">
        <div className="text-accent-green font-bold animate-pulse tracking-[0.2em]">
          PRISMSPLIT_CORE_BOOTSTRAP...
        </div>
      </div>
    );
  }

  if (!health?.runtimeReady || !health?.dependenciesReady) {
    return (
      <div className="app-container">
        <div className="top-bar">
          <div className="text-primary font-bold tracking-tighter text-lg">
            PRISMSPLIT // <span className="text-accent-green">CORE_SETUP</span>
          </div>
        </div>
        <main className="main-content">
          <SetupPanel
            health={health}
            setupStatus={setupStatus}
            onPrepare={handlePrepareEngine}
          />
        </main>
        <div className="status-bar">
          <span>SYSTEM_INIT_MODE</span>
          <span>BUILD: 2026.05.08</span>
        </div>
      </div>
    );
  }

  return (
    <div className="app-container">
      <div className="crt-overlay"></div>
      {/* Polybar-style Top Bar */}
      <div className="top-bar">
        <div className="flex items-center h-full">
          <div className="polybar-module bg-accent-green-dark text-accent-green font-bold terminal-text">
            PRISMSPLIT // V0.1.0-ALPHA
          </div>
          <div className="polybar-module p-0 flex h-full">
            <NavButton
              active={activeTab === "separate"}
              onClick={() => setActiveTab("separate")}
              label="[1] EXTRACTION"
            />
            <NavButton
              active={activeTab === "models"}
              onClick={() => setActiveTab("models")}
              label="[2] REGISTRY"
            />
            <NavButton
              active={activeTab === "settings"}
              onClick={() => setActiveTab("settings")}
              label="[3] CONFIG"
            />
          </div>
        </div>
        <div className="flex items-center h-full">
          <div className="polybar-module-right px-md text-xs font-mono text-secondary">
            RES:{" "}
            {health?.gpuDevices && health.gpuDevices.length > 0
              ? "CUDA_ENABLED"
              : "CPU_ONLY"}
          </div>
          <div className="polybar-module-right bg-primary px-md text-accent-green font-mono font-bold">
            {new Date().toISOString().split("T")[1].split(".")[0]}
          </div>
        </div>
      </div>

      {/* Main Content Workspace */}
      <main className="main-content">
        <div className="workspace">
          {activeTab === "separate" && (
            <div className="grid-dashboard">
              {/* Left Column (8/12) */}
              <div className="col-span-8 space-y-sm flex flex-col">
                <Fieldset legend="I/O TARGETS" className="flex-1">
                  <div className="space-y-sm flex flex-col justify-center h-full p-sm">
                    <div
                      className={`grid grid-cols-[100px_1fr_100px] gap-sm items-center p-md transition-all border-2 border-dashed ${isDragging ? "border-accent-green bg-accent-green-dark" : "border-border-mid bg-bg-secondary"}`}
                      onDragOver={(e) => {
                        e.preventDefault();
                        setIsDragging(true);
                      }}
                      onDragLeave={() => setIsDragging(false)}
                      onDrop={(e) => {
                        e.preventDefault();
                        setIsDragging(false);
                        const file = e.dataTransfer.files[0];
                        if (file) {
                          // @ts-ignore
                          const path = file.path || file.name;
                          setInputFile(path);
                          addLog(`FILE LOADED via D&D: ${path}`);
                        }
                      }}
                    >
                      <label className="text-secondary font-bold font-mono">
                        INPUT_SRC:
                      </label>
                      <input
                        type="text"
                        value={inputFile}
                        onChange={(e) => setInputFile(e.target.value)}
                        className="input w-full font-mono text-[10px]"
                        placeholder="Drag & Drop audio file here, or Browse..."
                      />
                      <Button
                        onClick={async () => {
                          const res = await openFileDialog();
                          if (res) {
                            setInputFile(res);
                            addLog(`FILE LOADED: ${res}`);
                          }
                        }}
                      >
                        BROWSE
                      </Button>
                    </div>

                    <div className="grid grid-cols-[100px_1fr_100px] gap-sm items-center p-md border border-border-mid bg-bg-secondary">
                      <label className="text-secondary font-bold font-mono">
                        OUTPUT_DIR:
                      </label>
                      <input
                        type="text"
                        value={outputDir}
                        onChange={(e) => setOutputDir(e.target.value)}
                        className="input w-full font-mono text-[10px]"
                        placeholder="Same as input directory"
                      />
                      <Button
                        onClick={async () => {
                          const res = await openDirDialog();
                          if (res) {
                            setOutputDir(res);
                            addLog(`DIR SELECTED: ${res}`);
                          }
                        }}
                      >
                        BROWSE
                      </Button>
                    </div>
                  </div>
                </Fieldset>

                <Fieldset legend="EXECUTION_NODE">
                  <SeparationPanel
                    request={{
                      inputPath: inputFile,
                      modelId: selectedModel,
                      outputDir,
                      format: exportFormat,
                    }}
                    onRun={handleProcess}
                    isProcessing={isProcessing}
                    progress={0}
                  />
                </Fieldset>
              </div>

              {/* Right Column (4/12) */}
              <div className="col-span-4 space-y-sm flex flex-col">
                <Fieldset legend="ENGINE_PARAMS">
                  <div className="space-y-md p-xs">
                    <Select
                      label="NEURAL ARCHITECTURE:"
                      value={selectedModel}
                      onChange={setSelectedModel}
                      options={catalog.map((m) => m.id)}
                    />

                    <Select
                      label="EXPORT CODEC:"
                      value={exportFormat}
                      onChange={setExportFormat}
                      options={[
                        "WAV (32-bit float)",
                        "WAV (24-bit PCM)",
                        "FLAC (Level 8)",
                        "MP3 (320kbps)",
                      ]}
                    />

                    <div className="flex flex-col gap-xs">
                      <div className="flex items-center">
                        <label className="select-label">COMPUTE PROFILE:</label>
                        <HelpIcon tooltip="Fast: CPU only. Normal: Basic GPU acceleration. High Quality: High overlap to reduce artifacts." />
                      </div>
                      <Select
                        value={quality}
                        onChange={setQuality}
                        options={[
                          "Fast (CPU)",
                          "Normal (CUDA)",
                          "High Quality (Overlap)",
                          "Extreme (Aggressive Math)",
                        ]}
                      />
                    </div>
                  </div>
                </Fieldset>

                <Fieldset legend="POST_PROCESSING" className="flex-1">
                  <div className="space-y-md p-xs">
                    <Checkbox
                      label="Invert Spectrogram (Subtract)"
                      checked={true}
                      onChange={() => {}}
                    />
                    <Checkbox
                      label="Enable TTA (Test-Time Augmentation)"
                      checked={false}
                      onChange={() => {}}
                    />
                    <Checkbox
                      label="Pre-Shift Pitch Alignment"
                      checked={false}
                      onChange={() => {}}
                    />
                    <Checkbox
                      label="Aggressive Noise Masking"
                      checked={false}
                      onChange={() => {}}
                    />
                  </div>
                </Fieldset>
              </div>
            </div>
          )}

          {activeTab === "models" && (
            <div className="grid-dashboard">
              <div className="col-span-12">
                <ModelRegistryPanel
                  models={catalog}
                  onDownload={handleDownloadModel}
                  onSync={handleSyncCatalog}
                  onScan={handleScanModels}
                  downloadingId={downloadingId}
                  downloadProgress={downloadProgress}
                />
              </div>
            </div>
          )}

          {activeTab === "settings" && (
            <div className="grid-dashboard">
              <div className="col-span-8 space-y-sm flex flex-col">
                <Fieldset legend="SYSTEM_PATHS" className="flex-1">
                  <div className="space-y-md p-sm">
                    <div className="grid grid-cols-[150px_1fr_100px] gap-sm items-center mb-md p-sm bg-bg-secondary border border-border-mid">
                      <label className="text-secondary font-bold font-mono">
                        REGISTRY_PATH:
                      </label>
                      <input
                        type="text"
                        value={appConfig.modelsDir}
                        onChange={(e) =>
                          setAppConfig((p) => ({
                            ...p,
                            modelsDir: e.target.value,
                          }))
                        }
                        placeholder="Default (AppData)"
                        className="input font-mono text-[10px] w-full"
                      />
                      <Button onClick={handleBrowseModels}>BROWSE</Button>
                    </div>
                    <div className="grid grid-cols-[150px_1fr_100px] gap-sm items-center p-sm bg-bg-secondary border border-border-mid">
                      <label className="text-secondary font-bold font-mono">
                        CACHE_MOUNT:
                      </label>
                      <input
                        type="text"
                        value={appConfig.cacheDir}
                        onChange={(e) =>
                          setAppConfig((p) => ({
                            ...p,
                            cacheDir: e.target.value,
                          }))
                        }
                        placeholder="Default (Temp)"
                        className="input font-mono text-[10px] w-full"
                      />
                      <Button onClick={handleBrowseCache}>BROWSE</Button>
                    </div>
                  </div>
                </Fieldset>
              </div>

              <div className="col-span-4 space-y-sm flex flex-col">
                <Fieldset legend="HARDWARE_ACCEL">
                  <div className="space-y-md p-xs">
                    <Select
                      label="EXEC_PROVIDER:"
                      value="CUDA (NVIDIA)"
                      options={[
                        "CUDA (NVIDIA)",
                        "DirectML (AMD / Intel)",
                        "CPU Only",
                      ]}
                      onChange={() => {}}
                    />

                    <Select
                      label="PREFERRED_DEVICE:"
                      value="0"
                      options={
                        health?.gpuDevices?.map(
                          (gpu, i) => `Device ${i}: ${gpu}`,
                        ) || ["No GPU detected"]
                      }
                      onChange={() => {}}
                    />

                    <Select
                      label="VRAM_LIMIT:"
                      value="No Limit"
                      options={["No Limit", "8 GB", "6 GB", "4 GB", "2 GB"]}
                      onChange={() => {}}
                    />
                  </div>
                </Fieldset>

                <div className="flex flex-col gap-sm mt-auto justify-end">
                  <Button
                    onClick={handleApplySettings}
                    variant="primary"
                    className="py-md w-full"
                  >
                    COMMIT_CHANGES
                  </Button>
                  <Button
                    onClick={() =>
                      setAppConfig({ modelsDir: "", cacheDir: "" })
                    }
                    className="w-full"
                  >
                    RESTORE_DEFAULTS
                  </Button>
                </div>
              </div>
            </div>
          )}
        </div>
      </main>

      {/* Output Console Panel */}
      <LogConsole logs={log} onClear={() => setLog([])} />

      {/* Status Bar */}
      <div className="status-bar">
        <div className="flex items-center gap-md">
          <div className="flex items-center gap-xs">
            <div
              className={`w-2 h-2 ${isProcessing ? "bg-accent-green animate-pulse" : "bg-accent-blue"}`}
            ></div>
            <span>{isProcessing ? "ENGINE_BUSY" : "ENGINE_READY"}</span>
          </div>
          <span>
            {isProcessing ? "Separating audio buffer..." : "Awaiting task..."}
          </span>
        </div>
        <div className="flex items-center gap-md">
          <span>
            {health?.gpuDevices && health.gpuDevices.length > 0
              ? `HW_ACCEL: ${health.gpuDevices[0].substring(0, 24)}`
              : "HW_ACCEL: NONE"}
          </span>
          <span className="text-accent-green">
            UTC: {new Date().toISOString().split("T")[1].split(".")[0]}
          </span>
        </div>
      </div>
    </div>
  );
}
