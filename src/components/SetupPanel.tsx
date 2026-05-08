// src/components/SetupPanel.tsx
import React from "react";
import type { EngineHealth, SetupStatus } from "../lib/types";
import { Button, Fieldset } from "./shared";

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
    <div className="flex-1 flex items-center justify-center p-md">
      <div className="max-w-2xl w-full">
        <Fieldset legend="Engine Setup Required">
          <div className="space-y-md p-sm">
            <h2 className="text-xl font-bold text-primary mb-md flex items-center gap-sm">
              <div className="w-4 h-4 bg-accent-green animate-pulse"></div>
              ENGINE INITIALIZATION
            </h2>

            <p className="text-secondary leading-relaxed">
              PrismSplit needs to initialize its private Python environment and
              verify core dependencies before separation features become
              available. This process is only required once.
            </p>

            <div className="sunken-panel font-mono text-xs">
              <h3 className="text-secondary mb-sm uppercase tracking-widest font-bold">
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
              <div className="bg-[#440000] border border-accent-red p-md text-accent-red text-xs font-bold">
                <div className="mb-xs">CRITICAL_EXCEPTION:</div>
                {setupStatus.lastError}
              </div>
            )}

            <div className="flex flex-col gap-md">
              <Button
                onClick={handlePrepare}
                disabled={!!isPreparing}
                variant="primary"
                className="py-md text-lg"
              >
                {isPreparing ? "INITIALIZING RUNTIME..." : "PREPARE ENGINE NOW"}
              </Button>

              {isPreparing && (
                <div className="space-y-sm">
                  <div className="flex justify-between text-[10px] text-secondary font-bold">
                    <span>STAGE: {setupStatus.currentStage}</span>
                    <span>
                      {setupStatus.completedStages.length} / 8 COMPLETE
                    </span>
                  </div>
                  <div className="progress-bar h-2">
                    <div
                      className="progress-bar-fill"
                      style={{
                        width: `${(setupStatus.completedStages.length / 8) * 100}%`,
                      }}
                    />
                  </div>
                  <div className="text-[9px] text-accent-green animate-pulse text-center">
                    {setupStatus.currentStage}... DO NOT CLOSE APPLICATION
                  </div>
                </div>
              )}
            </div>
          </div>
        </Fieldset>
      </div>
    </div>
  );
}

function HealthItem({ label, ready }: { label: string; ready?: boolean }) {
  return (
    <li className="flex items-center justify-between">
      <span>{label}</span>
      <span
        className={
          ready ? "text-accent-green font-bold" : "text-accent-red font-bold"
        }
      >
        [{ready ? "READY" : "MISSING"}]
      </span>
    </li>
  );
}
