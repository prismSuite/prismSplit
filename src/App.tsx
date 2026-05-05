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
} from "./lib/api";
import type {
  EngineHealth,
  SetupStatus,
  ModelCatalogEntry,
  ProcessAudioResponse,
} from "./lib/types";
import { SetupPanel } from "./components/SetupPanel";
import { ModelRegistryPanel } from "./components/ModelRegistryPanel";
import { SeparationPanel } from "./components/SeparationPanel";
import { LogConsole } from "./components/LogConsole";
import {
  ToolbarButton,
  Fieldset,
  ProButton,
  HelpIcon,
  Checkbox,
} from "./components/shared";

export default function App() {
  const [activeTab, setActiveTab] = useState("separate");
  const [catalog, setCatalog] = useState<ModelCatalogEntry[]>([]);
  const [theme, setTheme] = useState("theme-classic");

  // Engine state
  const [health, setHealth] = useState<EngineHealth | null>(null);
  const [setupStatus, setSetupStatus] = useState<SetupStatus | null>(null);
  const [isInitializing, setIsInitializing] = useState(true);

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

  const logEndRef = useRef<HTMLDivElement>(null);
  const downloadIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const processIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const previewTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
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
  }, []);

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
      <div className="flex h-screen items-center justify-center bg-[var(--bg-outer)] text-white">
        <div className="animate-pulse">PRISMSPLIT_CORE_BOOTSTRAP...</div>
      </div>
    );
  }

  if (!health?.runtimeReady || !health?.dependenciesReady) {
    return (
      <div className={`flex h-screen bg-[var(--bg-outer)] p-1 ${theme}`}>
        <div className="flex-1 bg-[var(--bg-panel)] border-2 border-[var(--border-chassis)] flex flex-col overflow-hidden">
          <div className="bg-[var(--bg-titlebar)] p-2 font-bold text-[var(--text-bright)]">
            PRISMSPLIT ENGINE SETUP
          </div>
          <div className="flex-1 overflow-y-auto bg-[var(--bg-workspace)]">
            <SetupPanel
              health={health}
              setupStatus={setupStatus}
              onPrepare={handlePrepareEngine}
            />
          </div>
        </div>
      </div>
    );
  }

  return (
    <div
      className={`flex h-screen bg-[var(--bg-outer)] text-[var(--text-main)] font-[Tahoma,sans-serif] text-xs select-none p-1 ${theme}`}
    >
      {/* Main Window Frame */}
      <div className="flex-1 flex flex-col bg-[var(--bg-panel)] border-2 border-t-[var(--border-hilite)] border-l-[var(--border-hilite)] border-b-[var(--border-chassis)] border-r-[var(--border-chassis)] shadow-lg">
        {/* Title Bar */}
        <div
          data-tauri-drag-region
          className="bg-[var(--bg-titlebar)] border-b-2 border-[var(--border-chassis)] px-2 py-1 flex items-center justify-between shadow-sm select-none cursor-default"
        >
          <div
            data-tauri-drag-region
            className="font-bold text-[var(--text-bright)] flex items-center gap-2"
          >
            <div className="w-3 h-3 bg-[var(--accent-secondary)] border border-[#111111] shadow-[inset_0_1px_1px_rgba(255,255,255,0.4)] pointer-events-none"></div>
            <span className="pointer-events-none">
              PRISMSPLIT STEM EXTRACTOR
            </span>
          </div>
          <div className="flex items-center gap-4">
            <div className="text-[10px] text-[var(--text-muted)] pointer-events-none">
              v0.1.0-alpha x64
            </div>
            <div className="flex items-center gap-1">
              <button
                onClick={minimizeWindow}
                className="w-5 h-5 flex items-center justify-center bg-[var(--bg-btn)] border border-[var(--border-chassis)] hover:bg-[var(--bg-btn-hover)] text-[var(--text-main)] active:bg-[var(--bg-panel)]"
              >
                _
              </button>
              <button
                onClick={toggleMaximizeWindow}
                className="w-5 h-5 flex items-center justify-center bg-[var(--bg-btn)] border border-[var(--border-chassis)] hover:bg-[var(--bg-btn-hover)] text-[var(--text-main)] active:bg-[var(--bg-panel)]"
              >
                □
              </button>
              <button
                onClick={closeWindow}
                className="w-5 h-5 flex items-center justify-center bg-[#880000] border border-[#330000] hover:bg-[#aa0000] text-[#ffffff] active:bg-[#660000]"
              >
                X
              </button>
            </div>
          </div>
        </div>

        {/* Toolbar */}
        <div className="bg-[var(--bg-toolbar)] border-b-2 border-t-[var(--border-hilite)] border-[#222222] p-1 flex gap-1">
          <ToolbarButton
            active={activeTab === "separate"}
            onClick={() => setActiveTab("separate")}
            label="Extraction"
          />
          <ToolbarButton
            active={activeTab === "train"}
            onClick={() => setActiveTab("train")}
            label="Training Mode"
          />
          <ToolbarButton
            active={activeTab === "models"}
            onClick={() => setActiveTab("models")}
            label="Model Registry"
          />
          <ToolbarButton
            active={activeTab === "settings"}
            onClick={() => setActiveTab("settings")}
            label="System Config"
          />
        </div>

        {/* Workspace */}
        <div className="flex-1 p-2 bg-[var(--bg-workspace)] overflow-y-auto inset-border">
          {activeTab === "separate" && (
            <div className="space-y-4 max-w-4xl mx-auto">
              {/* I/O Section */}
              <Fieldset legend="I/O Configuration">
                <div
                  className={`grid grid-cols-[100px_1fr_80px] gap-2 items-center mb-2 p-2 transition-all border-2 border-dashed ${isDragging ? "border-[var(--accent-glow)] bg-[var(--bg-toolbar)]" : "border-transparent"}`}
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
                  <label>Input Source:</label>
                  <input
                    type="text"
                    value={inputFile}
                    onChange={(e) => setInputFile(e.target.value)}
                    className="pro-input"
                    placeholder="Drag & Drop audio file here, or Browse..."
                  />
                  <ProButton
                    label="Browse..."
                    onClick={async () => {
                      const res = await openFileDialog();
                      if (res) {
                        setInputFile(res);
                        addLog(`FILE LOADED: ${res}`);
                      }
                    }}
                  />
                </div>
                <div className="grid grid-cols-[100px_1fr_80px] gap-2 items-center">
                  <label>Output Dir:</label>
                  <input
                    type="text"
                    value={outputDir}
                    onChange={(e) => setOutputDir(e.target.value)}
                    className="pro-input"
                    placeholder="Same as input directory"
                  />
                  <ProButton
                    label="Browse..."
                    onClick={async () => {
                      const res = await openDirDialog();
                      if (res) {
                        setOutputDir(res);
                        addLog(`DIR SELECTED: ${res}`);
                      }
                    }}
                  />
                </div>
              </Fieldset>

              <div className="grid grid-cols-2 gap-4">
                {/* Engine Settings */}
                <Fieldset legend="Engine Parameters">
                  <div className="space-y-3">
                    <div>
                      <label className="block mb-1">
                        Architecture / Weights:
                      </label>
                      <select
                        value={selectedModel}
                        onChange={(e) => setSelectedModel(e.target.value)}
                        className="pro-input w-full"
                      >
                        {catalog.map((model) => (
                          <option key={model.id} value={model.id}>
                            {model.name}
                          </option>
                        ))}
                      </select>
                    </div>

                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <label className="block mb-1">Export Format:</label>
                        <select
                          value={exportFormat}
                          onChange={(e) => setExportFormat(e.target.value)}
                          className="pro-input w-full"
                        >
                          <option>WAV (32-bit float)</option>
                          <option>WAV (24-bit PCM)</option>
                          <option>FLAC (Level 8)</option>
                          <option>MP3 (320kbps)</option>
                        </select>
                      </div>
                      <div>
                        <div className="flex items-center mb-1">
                          <label>Compute Profile:</label>
                          <HelpIcon tooltip="Fast: CPU only. Normal: Basic GPU acceleration. High Quality: High overlap to reduce artifacts. Extreme: Max settings, heavy VRAM usage." />
                        </div>
                        <select
                          value={quality}
                          onChange={(e) => setQuality(e.target.value)}
                          className="pro-input w-full"
                        >
                          <option>Fast (CPU)</option>
                          <option>Normal (CUDA)</option>
                          <option>High Quality (Overlap)</option>
                          <option>Extreme (Aggressive Math)</option>
                        </select>
                      </div>
                    </div>
                  </div>
                </Fieldset>

                {/* Advanced Operations */}
                <Fieldset legend="Advanced">
                  <div className="space-y-2">
                    <Checkbox
                      label="Invert Spectrogram (Subtract vocals)"
                      defaultChecked
                    />
                    <Checkbox label="Enable TTA (Test-Time Augmentation)" />
                    <Checkbox label="Shift pitch before processing" />
                    <Checkbox label="Post-process masking" />
                  </div>
                </Fieldset>
              </div>

              {/* Execution */}
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

              {/* Output Console */}
              <LogConsole logs={log} onClear={() => setLog([])} />
            </div>
          )}

          {activeTab === "train" && (
            <div className="text-center mt-10 text-[var(--text-muted)]">
              <div className="text-xl font-bold mb-2 text-[var(--text-subtext)]">
                MODULE NOT LOADED
              </div>
              <p>
                Training engine requires manual PyTorch environment
                configuration.
              </p>
              <p>Check the console for dependency errors.</p>
            </div>
          )}

          {activeTab === "models" && (
            <ModelRegistryPanel
              models={catalog}
              onDownload={handleDownloadModel}
              downloadingId={downloadingId}
              downloadProgress={0}
            />
          )}

          {activeTab === "settings" && (
            <div className="space-y-4 max-w-4xl mx-auto mt-2">
              <Fieldset legend="Hardware Acceleration (GPU)">
                <div className="space-y-4">
                  <div className="grid grid-cols-[150px_1fr] gap-2 items-center">
                    <label>Execution Provider:</label>
                    <select className="pro-input">
                      <option>CUDA (NVIDIA)</option>
                      <option>DirectML (AMD / Intel)</option>
                      <option>CPU Only</option>
                    </select>
                  </div>

                  <div className="grid grid-cols-[150px_1fr] gap-2 items-center">
                    <label>Preferred GPU Device:</label>
                    <select className="pro-input">
                      <option>Device 0: NVIDIA GeForce RTX 4090</option>
                      <option>Device 1: Intel(R) UHD Graphics</option>
                      <option>Auto-detect</option>
                    </select>
                  </div>

                  <div className="grid grid-cols-[150px_1fr] gap-2 items-center">
                    <div className="flex items-center">
                      <label>VRAM Allocation Limit:</label>
                      <HelpIcon tooltip="Restricts how much video memory the model can allocate. Lower limits may prevent crashes on older GPUs but increase processing time." />
                    </div>
                    <select className="pro-input">
                      <option>No Limit</option>
                      <option>8 GB</option>
                      <option>6 GB</option>
                      <option>4 GB</option>
                      <option>2 GB</option>
                    </select>
                  </div>
                </div>
              </Fieldset>

              <Fieldset legend="Appearance">
                <div className="grid grid-cols-[150px_1fr] gap-2 items-center mb-2">
                  <label>UI Theme:</label>
                  <select
                    value={theme}
                    onChange={(e) => setTheme(e.target.value)}
                    className="pro-input"
                  >
                    <option value="theme-classic">
                      Classic Dark (PrismSplit)
                    </option>
                    <option value="theme-win95">Win95 Gray</option>
                    <option value="theme-cyberpunk">Cyberpunk Neons</option>
                    <option value="theme-matrix">Terminal Matrix</option>
                    <option value="theme-amber">Amber CRT</option>
                    <option value="theme-crimson">Blood Red</option>
                    <option value="theme-deepblue">Midnight Blue</option>
                  </select>
                </div>
              </Fieldset>

              <Fieldset legend="System Paths">
                <div className="grid grid-cols-[150px_1fr_80px] gap-2 items-center mb-2">
                  <label>Model Registry Path:</label>
                  <input
                    type="text"
                    defaultValue="C:\\PrismSplit\\Models"
                    className="pro-input"
                  />
                  <ProButton
                    label="Browse..."
                    onClick={() => openDirDialog()}
                  />
                </div>
                <div className="grid grid-cols-[150px_1fr_80px] gap-2 items-center">
                  <label>Temporary Cache:</label>
                  <input
                    type="text"
                    defaultValue="%TEMP%\\PrismSplit"
                    className="pro-input"
                  />
                  <ProButton
                    label="Browse..."
                    onClick={() => openDirDialog()}
                  />
                </div>
              </Fieldset>

              <div className="flex justify-end gap-2 mt-4">
                <button className="bg-[var(--bg-btn)] text-[var(--text-bright)] px-6 py-2 border-2 border-t-[var(--border-hilite)] border-l-[var(--border-hilite)] border-b-[var(--border-shadow-deep)] border-r-[var(--border-shadow-deep)] active:border-t-[var(--border-shadow-deep)] active:border-l-[var(--border-shadow-deep)] active:border-b-[var(--border-hilite)] active:border-r-[var(--border-hilite)]">
                  Apply
                </button>
                <button className="bg-[var(--bg-btn)] text-[var(--text-bright)] px-6 py-2 border-2 border-t-[var(--border-hilite)] border-l-[var(--border-hilite)] border-b-[var(--border-shadow-deep)] border-r-[var(--border-shadow-deep)] active:border-t-[var(--border-shadow-deep)] active:border-l-[var(--border-shadow-deep)] active:border-b-[var(--border-hilite)] active:border-r-[var(--border-hilite)]">
                  Reset Defaults
                </button>
              </div>
            </div>
          )}
        </div>

        {/* Status Bar */}
        <div className="bg-[var(--bg-panel)] border-t-2 border-[var(--border-chassis)] px-2 py-1 text-[10px] text-[var(--text-subtext)] flex justify-between">
          <div>
            {isProcessing
              ? "Processing audio buffer..."
              : isPreviewing
                ? "Generating preview..."
                : "Ready"}
          </div>
          <div>MEM: 452MB / GPU: READY</div>
        </div>
      </div>

      <style
        dangerouslySetInnerHTML={{
          __html: `
        .theme-classic {
          --bg-outer: #383838;
          --bg-panel: #404040;
          --bg-titlebar: #2b2b36;
          --bg-toolbar: #484848;
          --bg-workspace: #303030;
          --bg-input: #1e1e1e;
          --bg-input-alt: #252525;
          --bg-console: #000000;
          --bg-btn: #4a4a4a;
          --bg-btn-active: #404040;
          --bg-btn-hover: #505050;
          --border-hilite: #666666;
          --border-hilite-subtle: #555555;
          --border-shadow: #222222;
          --border-shadow-deep: #111111;
          --border-chassis: #1a1a1a;
          --border-fieldset: #2a2a2a;
          --border-fieldset-subtle: #5a5a5a;
          --border-subtle: #333333;
          --accent-glow: #00ff00;
          --accent-border: #00aa00;
          --accent-secondary: #4488ff;
          --text-main: #d4d4d4;
          --text-bright: #ffffff;
          --text-muted: #888888;
          --text-subtext: #aaaaaa;
        }

        .theme-win95 {
          --bg-outer: #008080;
          --bg-panel: #c0c0c0;
          --bg-titlebar: #000080;
          --bg-toolbar: #c0c0c0;
          --bg-workspace: #c0c0c0;
          --bg-input: #ffffff;
          --bg-input-alt: #f0f0f0;
          --bg-console: #000000;
          --bg-btn: #c0c0c0;
          --bg-btn-active: #a0a0a0;
          --bg-btn-hover: #cfcfcf;
          --border-hilite: #ffffff;
          --border-hilite-subtle: #dfdfdf;
          --border-shadow: #808080;
          --border-shadow-deep: #000000;
          --border-chassis: #000000;
          --border-fieldset: #808080;
          --border-fieldset-subtle: #ffffff;
          --border-subtle: #808080;
          --accent-glow: #00ff00;
          --accent-border: #008000;
          --accent-secondary: #000080;
          --text-main: #000000;
          --text-bright: #000000; /* Windows 95 didn't have bright text in general except on dark bgs */
          --text-muted: #808080;
          --text-subtext: #000000;
        }

        .theme-win95 .text-\\[var\\(--text-bright\\)\\] {
          color: #ffffff; /* We must force white for title bars */
        }

        .theme-cyberpunk {
          --bg-outer: #0a0a1a;
          --bg-panel: #111122;
          --bg-titlebar: #220022;
          --bg-toolbar: #1a1a33;
          --bg-workspace: #0f0f1a;
          --bg-input: #05050a;
          --bg-input-alt: #080812;
          --bg-console: #000000;
          --bg-btn: #2a1a3a;
          --bg-btn-active: #1a0a2a;
          --bg-btn-hover: #3a2a4a;
          --border-hilite: #ff00ff;
          --border-hilite-subtle: #880088;
          --border-shadow: #00ffff;
          --border-shadow-deep: #008888;
          --border-chassis: #00ffff;
          --border-fieldset: #ff00ff;
          --border-fieldset-subtle: #00ffff;
          --border-subtle: #880088;
          --accent-glow: #00ffff;
          --accent-border: #008888;
          --accent-secondary: #ff00ff;
          --text-main: #e0e0ff;
          --text-bright: #ffffff;
          --text-muted: #8888aa;
          --text-subtext: #aaaacc;
        }

        .theme-matrix {
          --bg-outer: #000000;
          --bg-panel: #001100;
          --bg-titlebar: #002200;
          --bg-toolbar: #001a00;
          --bg-workspace: #000a00;
          --bg-input: #000000;
          --bg-input-alt: #000500;
          --bg-console: #000000;
          --bg-btn: #003300;
          --bg-btn-active: #002200;
          --bg-btn-hover: #004400;
          --border-hilite: #00ff00;
          --border-hilite-subtle: #00aa00;
          --border-shadow: #005500;
          --border-shadow-deep: #002200;
          --border-chassis: #00ff00;
          --border-fieldset: #00ff00;
          --border-fieldset-subtle: #005500;
          --border-subtle: #008800;
          --accent-glow: #00ff00;
          --accent-border: #00aa00;
          --accent-secondary: #00ff00;
          --text-main: #00cc00;
          --text-bright: #00ff00;
          --text-muted: #005500;
          --text-subtext: #00aa00;
        }

        .theme-amber {
          --bg-outer: #000000;
          --bg-panel: #1a0f00;
          --bg-titlebar: #331f00;
          --bg-toolbar: #261700;
          --bg-workspace: #0f0800;
          --bg-input: #000000;
          --bg-input-alt: #050200;
          --bg-console: #000000;
          --bg-btn: #331f00;
          --bg-btn-active: #221500;
          --bg-btn-hover: #442a00;
          --border-hilite: #ffb000;
          --border-hilite-subtle: #cc8c00;
          --border-shadow: #664600;
          --border-shadow-deep: #332300;
          --border-chassis: #ffb000;
          --border-fieldset: #ffb000;
          --border-fieldset-subtle: #664600;
          --border-subtle: #996900;
          --accent-glow: #ffb000;
          --accent-border: #cc8c00;
          --accent-secondary: #ffb000;
          --text-main: #ffcc66;
          --text-bright: #ffb000;
          --text-muted: #886622;
          --text-subtext: #cc9933;
        }

        .theme-crimson {
          --bg-outer: #1a0505;
          --bg-panel: #2a0a0a;
          --bg-titlebar: #3a0000;
          --bg-toolbar: #220505;
          --bg-workspace: #1f0808;
          --bg-input: #0a0000;
          --bg-input-alt: #110000;
          --bg-console: #000000;
          --bg-btn: #3a0808;
          --bg-btn-active: #2a0000;
          --bg-btn-hover: #4a1111;
          --border-hilite: #ff3333;
          --border-hilite-subtle: #aa2222;
          --border-shadow: #550000;
          --border-shadow-deep: #220000;
          --border-chassis: #ff0000;
          --border-fieldset: #ff3333;
          --border-fieldset-subtle: #550000;
          --border-subtle: #881111;
          --accent-glow: #ff0000;
          --accent-border: #aa0000;
          --accent-secondary: #ff3333;
          --text-main: #ffaaaa;
          --text-bright: #ffffff;
          --text-muted: #884444;
          --text-subtext: #cc6666;
        }

        .theme-deepblue {
          --bg-outer: #050a1a;
          --bg-panel: #0a112a;
          --bg-titlebar: #001a4a;
          --bg-toolbar: #081533;
          --bg-workspace: #080f22;
          --bg-input: #000511;
          --bg-input-alt: #000a1a;
          --bg-console: #000000;
          --bg-btn: #11224a;
          --bg-btn-active: #08112a;
          --bg-btn-hover: #1a3366;
          --border-hilite: #3388ff;
          --border-hilite-subtle: #2255aa;
          --border-shadow: #002255;
          --border-shadow-deep: #00112a;
          --border-chassis: #3388ff;
          --border-fieldset: #3388ff;
          --border-fieldset-subtle: #002255;
          --border-subtle: #114488;
          --accent-glow: #00aaff;
          --accent-border: #0066aa;
          --accent-secondary: #3388ff;
          --text-main: #aaccff;
          --text-bright: #ffffff;
          --text-muted: #5577aa;
          --text-subtext: #88bbff;
        }

        .inset-border {
          border: 2px solid;
          border-top-color: var(--border-chassis);
          border-left-color: var(--border-chassis);
          border-bottom-color: var(--border-hilite-subtle);
          border-right-color: var(--border-hilite-subtle);
        }
        .pro-input {
          background-color: var(--bg-input);
          color: var(--text-bright);
          border: 2px solid;
          border-top-color: var(--border-shadow-deep);
          border-left-color: var(--border-shadow-deep);
          border-bottom-color: var(--border-hilite-subtle);
          border-right-color: var(--border-hilite-subtle);
          padding: 2px 4px;
          outline: none;
        }
        .pro-input:focus {
          border-color: var(--accent-secondary);
        }
      `,
        }}
      />
    </div>
  );
}
