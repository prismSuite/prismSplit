// src/components/SeparationPanel.tsx
import React from "react";
import type { SeparationRequest } from "../lib/types";
import { Button, Fieldset } from "./shared";

type Props = {
  request: SeparationRequest;
  onRun: () => Promise<void>;
  isProcessing: boolean;
  progress: number;
};

export function SeparationPanel({
  request,
  onRun,
  isProcessing,
  progress,
}: Props) {
  return (
    <div className="separation-panel space-y-md">
      <Fieldset legend="Separation Task Control">
        <div className="flex items-center gap-md">
          <Button
            onClick={() => void onRun()}
            disabled={isProcessing || !request.inputPath}
            variant="primary"
            className="py-3 px-8 text-lg shrink-0"
          >
            {isProcessing ? "PROCESSING..." : "START SEPARATION"}
          </Button>

          <div className="flex-1 flex flex-col justify-center sunken-panel p-sm relative">
            <div className="text-[9px] text-secondary mb-xs uppercase font-bold">
              EXECUTION_FLOW
            </div>
            <div className="progress-bar">
              <div
                className="progress-bar-fill"
                style={{ width: `${progress}%` }}
              />
              <div className="progress-bar-text">{progress.toFixed(1)}%</div>
            </div>
          </div>
        </div>
      </Fieldset>
    </div>
  );
}
