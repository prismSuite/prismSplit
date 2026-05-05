// src/components/SeparationPanel.tsx
import React from "react";
import type { SeparationRequest } from "../lib/types";
import { HelpIcon, ProButton, Fieldset } from "./shared"; // Assuming I extract them

type Props = {
  request: SeparationRequest;
  onRun: () => Promise<void>;
  isProcessing: boolean;
  progress: number;
};

export function SeparationPanel({ request, onRun, isProcessing, progress }: Props) {
  return (
    <div className="separation-panel space-y-4">
      <Fieldset legend="Separation Task Control">
         <div className="flex items-center gap-4">
            <button
              onClick={() => void onRun()}
              disabled={isProcessing || !request.inputPath}
              className={`font-bold py-3 px-8 outline-none transition-all ${
                isProcessing
                  ? "bg-[var(--bg-input-alt)] text-[var(--accent-glow)] border-2 border-[var(--accent-glow)]"
                  : "bg-[var(--bg-btn)] text-[var(--text-bright)] border-2 border-t-[var(--border-hilite)] border-l-[var(--border-hilite)] border-b-[var(--border-shadow-deep)] border-r-[var(--border-shadow-deep)] active:bg-[var(--bg-panel)]"
              }`}
            >
              {isProcessing ? "PROCESSING..." : "START SEPARATION"}
            </button>

            <div className="flex-1 flex flex-col justify-center bg-[var(--bg-input-alt)] border-2 border-[var(--border-shadow-deep)] p-2 relative">
               <div className="absolute top-1 left-2 text-[9px] text-[#666]">EXECUTION_FLOW</div>
               <div className="w-full h-4 bg-[var(--border-shadow-deep)] border border-[#000] mt-1 relative overflow-hidden">
                 <div
                   className="absolute top-0 left-0 h-full bg-[var(--accent-glow)] transition-all duration-200"
                   style={{ width: `${progress}%` }}
                 />
                 <div className="absolute top-0 left-0 w-full h-full flex items-center justify-center font-bold text-[10px] mix-blend-difference text-white">
                   {progress.toFixed(1)}%
                 </div>
               </div>
            </div>
         </div>
      </Fieldset>
    </div>
  );
}
