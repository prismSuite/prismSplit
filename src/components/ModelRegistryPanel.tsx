// src/components/ModelRegistryPanel.tsx
import React from "react";
import type { ModelCatalogEntry } from "../lib/types";

type Props = {
  models: ModelCatalogEntry[];
  onDownload: (modelId: string) => Promise<void>;
  downloadingId: string | null;
  downloadProgress: number;
};

export function ModelRegistryPanel({ models, onDownload, downloadingId, downloadProgress }: Props) {
  return (
    <section className="model-registry space-y-6">
      <div className="bg-[var(--bg-panel)] border-2 border-t-[var(--border-hilite)] border-l-[var(--border-hilite)] border-b-[var(--border-shadow-deep)] border-r-[var(--border-shadow-deep)] p-4">
        <h2 className="text-lg font-bold text-[var(--text-bright)] mb-4 uppercase tracking-tighter">
          AVAILABLE ARCHITECTURES
        </h2>

        <div className="overflow-x-auto">
          <table className="w-full text-left font-[Courier,monospace] text-xs">
            <thead>
              <tr className="border-b border-[var(--border-subtle)] text-[var(--text-muted)]">
                <th className="py-2 px-1">MODEL_ID</th>
                <th className="py-2 px-1">BACKEND</th>
                <th className="py-2 px-1">KIND</th>
                <th className="py-2 px-1 text-right">SIZE</th>
                <th className="py-2 px-1 text-center w-32">ACTION</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[var(--border-subtle)]">
              {models.map((model) => {
                const isDownloading = downloadingId === model.id;
                return (
                  <tr key={model.id} className="hover:bg-[var(--bg-input-alt)]">
                    <td className="py-3 px-1 text-[var(--text-bright)] font-bold">{model.name}</td>
                    <td className="py-3 px-1 uppercase">{model.backend}</td>
                    <td className="py-3 px-1 text-[var(--text-muted)]">{model.outputKind}</td>
                    <td className="py-3 px-1 text-right">{(model.sizeBytes / 1024 / 1024).toFixed(1)} MB</td>
                    <td className="py-3 px-1 text-center">
                       {isDownloading ? (
                         <div className="space-y-1">
                           <div className="w-full h-1 bg-[var(--bg-input)] border border-[#000]">
                             <div
                               className="h-full bg-[var(--accent-glow)] transition-all duration-300"
                               style={{ width: `${downloadProgress}%` }}
                             />
                           </div>
                           <div className="text-[9px] text-[var(--accent-glow)]">{downloadProgress.toFixed(0)}%</div>
                         </div>
                       ) : (
                         <button
                           onClick={() => void onDownload(model.id)}
                           className="bg-[var(--bg-btn)] text-[var(--text-bright)] px-3 py-1 border border-t-[var(--border-hilite)] border-l-[var(--border-hilite)] border-b-[var(--border-shadow-deep)] border-r-[var(--border-shadow-deep)] active:bg-[var(--bg-panel)] active:border-t-[var(--border-shadow-deep)] active:border-l-[var(--border-shadow-deep)] hover:brightness-110 text-[10px]"
                         >
                           DOWNLOAD
                         </button>
                       )}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>

      <div className="bg-[var(--bg-input)] border border-dashed border-[var(--border-fieldset-subtle)] p-4 text-[10px] text-[var(--text-muted)]">
        <p>
          NOTE: Model files are served from authorized PrismSplit repositories.
          Verification occurs automatically via SHA-256 checksum after download completion.
        </p>
      </div>
    </section>
  );
}
