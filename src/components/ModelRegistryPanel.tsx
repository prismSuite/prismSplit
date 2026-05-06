// src/components/ModelRegistryPanel.tsx
import React from "react";
import type { ModelCatalogEntry } from "../lib/types";

type Props = {
  models: ModelCatalogEntry[];
  onDownload: (modelId: string) => Promise<void>;
  onSync: () => Promise<void>;
  onScan: () => Promise<void>;
  downloadingId: string | null;
  downloadProgress: number;
};

export function ModelRegistryPanel({
  models,
  onDownload,
  onSync,
  onScan,
  downloadingId,
  downloadProgress,
}: Props) {
  return (
    <section className="model-registry space-y-6">
      <div className="bg-() border-2 border-t-() border-l-() border-b-() border-r-() p-4">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-lg font-bold text-() uppercase tracking-tighter">
            AVAILABLE ARCHITECTURES
          </h2>
          <div className="flex gap-2">
            <button
              onClick={onScan}
              className="bg-() text-() px-3 py-1 border border-t-() border-l-() border-b-() border-r-() active:bg-() active:border-t-() active:border-l-() hover:brightness-110 text-[10px] font-bold"
            >
              SCAN LOCAL DIRECTORY
            </button>
            <button
              onClick={onSync}
              className="bg-() text-() px-3 py-1 border border-t-() border-l-() border-b-() border-r-() active:bg-() active:border-t-() active:border-l-() hover:brightness-110 text-[10px] font-bold"
            >
              SYNC WITH UVR SERVERS
            </button>
          </div>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left font-[Courier,monospace] text-xs">
            <thead>
              <tr className="border-b border-() text-()">
                <th className="py-2 px-1">MODEL_ID</th>
                <th className="py-2 px-1">BACKEND</th>
                <th className="py-2 px-1">KIND</th>
                <th className="py-2 px-1 text-right">SIZE</th>
                <th className="py-2 px-1 text-center w-32">ACTION</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-()">
              {models.map((model) => {
                const isDownloading = downloadingId === model.id;
                return (
                  <tr key={model.id} className="hover:bg-()">
                    <td className="py-3 px-1 text-() font-bold">
                      {model.name}
                    </td>
                    <td className="py-3 px-1 uppercase">{model.backend}</td>
                    <td className="py-3 px-1 text-()">
                      {model.outputKind}
                    </td>
                    <td className="py-3 px-1 text-right">
                      {(model.sizeBytes / 1024 / 1024).toFixed(1)} MB
                    </td>
                    <td className="py-3 px-1 text-center">
                      {isDownloading ? (
                        <div className="space-y-1">
                          <div className="w-full h-1 bg-() border border-[#000]">
                            <div
                              className="h-full bg-() transition-all duration-300"
                              style={{ width: `${downloadProgress}%` }}
                            />
                          </div>
                          <div className="text-[9px] text-()">
                            {downloadProgress.toFixed(0)}%
                          </div>
                        </div>
                      ) : (
                        <button
                          onClick={() => void onDownload(model.id)}
                          className="bg-() text-() px-3 py-1 border border-t-() border-l-() border-b-() border-r-() active:bg-() active:border-t-() active:border-l-() hover:brightness-110 text-[10px]"
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

      <div className="bg-() border border-dashed border-() p-4 text-[10px] text-()">
        <p>
          NOTE: Model files are served from authorized PrismSplit repositories.
          Verification occurs automatically via SHA-256 checksum after download
          completion.
        </p>
      </div>
    </section>
  );
}
