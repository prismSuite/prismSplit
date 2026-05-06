// src/components/SetupPanel.tsx
import React from "react";
import type { EngineHealth, SetupStatus } from "../lib/types";

type Props = {
  health: EngineHealth | null;
  setupStatus: SetupStatus | null;
  onPrepare: () => Promise<void>;
};

export function SetupPanel({ health, setupStatus, onPrepare }: Props) {
  const [localIsPreparing, setLocalIsPreparing] = React.useState(false);
  const isPreparing =
    localIsPreparing ||
    (setupStatus && !setupStatus.ready && setupStatus.currentStage);

  const handlePrepare = async () => {
    if (isPreparing) return;
    setLocalIsPreparing(true);
    try {
      await onPrepare();
    } finally {
      setLocalIsPreparing(false);
    }
  };

  return (
    <section className="setup-panel p-4 max-w-2xl mx-auto space-y-6">
      <div className="bg-() border-2 border-t-() border-l-() border-b-() border-r-() p-6 shadow-xl">
        <h2 className="text-xl font-bold text-() mb-4 flex items-center gap-3">
          <div className="w-4 h-4 bg-() animate-pulse"></div>
          ENGINE SETUP REQUIRED
        </h2>

        <p className="text-() mb-6 leading-relaxed">
          PrismSplit needs to initialize its private Python environment and
          verify core dependencies before separation features become available.
          This process is only required once.
        </p>

        <div className="bg-() border-2 border-t-() border-l-() border-b-() border-r-() p-4 mb-6 font-[Courier,monospace] text-xs">
          <h3 className="text-() mb-2 uppercase tracking-widest">
            Health Check Report:
          </h3>
          <ul className="space-y-1">
            <HealthItem
              label="Runtime Sub-System"
              ready={health?.runtimeReady}
            />
            <HealthItem
              label="Dependency Registry"
              ready={health?.dependenciesReady}
            />
            <HealthItem
              label="FFmpeg Binary Bridge"
              ready={health?.ffmpegReady}
            />
            <HealthItem
              label="Model Catalog Sync"
              ready={health?.modelCatalogReady}
            />
          </ul>
        </div>

        {setupStatus?.lastError && (
          <div className="bg-[#440000] border border-[#ff0000] p-3 mb-6 text-[#ffaaaa] text-xs">
            <div className="font-bold mb-1">CRITICAL_EXCEPTION:</div>
            {setupStatus.lastError}
          </div>
        )}

        <div className="flex flex-col gap-4">
          <button
            onClick={handlePrepare}
            disabled={!!isPreparing}
            className={`font-bold py-4 px-8 border-2 transition-all ${
              isPreparing
                ? "bg-() text-() border-() cursor-not-allowed"
                : "bg-() text-white border-t-[#88aaff] border-l-[#88aaff] border-b-[#112244] border-r-[#112244] hover:brightness-110 active:brightness-90"
            }`}
          >
            {isPreparing ? "INITIALIZING RUNTIME..." : "PREPARE ENGINE NOW"}
          </button>

          {isPreparing && (
            <div className="space-y-2">
              <div className="flex justify-between text-[10px] text-()">
                <span>STAGE: {setupStatus.currentStage}</span>
                <span>{setupStatus.completedStages.length} COMPLETE</span>
              </div>
              <div className="w-full h-2 bg-() border border-[#000] relative overflow-hidden">
                <div
                  className="absolute top-0 left-0 h-full bg-() transition-all duration-500 shadow-[0_0_8px_var(--accent-secondary)]"
                  style={{
                    width: `${(setupStatus.completedStages.length / 8) * 100}%`,
                  }}
                />
              </div>
              <div className="text-[9px] text-() animate-pulse">
                {setupStatus.currentStage}... DO NOT CLOSE APPLICATION
              </div>
            </div>
          )}
        </div>
      </div>
    </section>
  );
}

function HealthItem({ label, ready }: { label: string; ready?: boolean }) {
  return (
    <li className="flex items-center justify-between">
      <span>{label}</span>
      <span className={ready ? "text-()" : "text-[#ff4444]"}>
        [{ready ? "READY" : "MISSING"}]
      </span>
    </li>
  );
}
