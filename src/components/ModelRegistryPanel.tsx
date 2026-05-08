// src/components/ModelRegistryPanel.tsx
import React from "react";
import type { ModelCatalogEntry } from "../lib/types";
import { Button, Fieldset } from "./shared";

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
    <div className="space-y-md">
      <Fieldset legend="Model Registry">
        <div className="flex justify-between items-center mb-md">
          <h2 className="text-lg font-bold text-primary uppercase tracking-tighter">
            AVAILABLE ARCHITECTURES
          </h2>
          <div className="flex gap-sm">
            <Button onClick={onScan} className="text-[10px]">
              SCAN LOCAL DIRECTORY
            </Button>
            <Button onClick={onSync} className="text-[10px]">
              SYNC WITH UVR SERVERS
            </Button>
          </div>
        </div>

        <div className="sunken-panel p-0">
          <table className="w-full text-left font-mono text-xs border-collapse">
            <thead>
              <tr className="bg-primary text-secondary border-b border-mid">
                <th className="py-2 px-sm">MODEL_ID</th>
                <th className="py-2 px-sm">BACKEND</th>
                <th className="py-2 px-sm">KIND</th>
                <th className="py-2 px-sm text-right">SIZE</th>
                <th className="py-2 px-sm text-center w-32">ACTION</th>
              </tr>
            </thead>
            <tbody>
              {models.map((model) => {
                const isDownloading = downloadingId === model.id;
                return (
                  <tr
                    key={model.id}
                    className="border-b border-mid hover:bg-primary transition-colors duration-fast"
                  >
                    <td className="py-3 px-sm text-primary font-bold">
                      {model.name}
                    </td>
                    <td className="py-3 px-sm uppercase">{model.backend}</td>
                    <td className="py-3 px-sm text-secondary">
                      {model.outputKind}
                    </td>
                    <td className="py-3 px-sm text-right">
                      {(model.sizeBytes / 1024 / 1024).toFixed(1)} MB
                    </td>
                    <td className="py-3 px-sm text-center">
                      {isDownloading ? (
                        <div className="space-y-xs px-sm">
                          <div className="progress-bar">
                            <div
                              className="progress-bar-fill"
                              style={{ width: `${downloadProgress}%` }}
                            />
                            <div className="progress-bar-text">
                              {downloadProgress.toFixed(0)}%
                            </div>
                          </div>
                        </div>
                      ) : (
                        <Button
                          onClick={() => void onDownload(model.id)}
                          className="text-[10px] w-full"
                          variant="primary"
                        >
                          DOWNLOAD
                        </Button>
                      )}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </Fieldset>

      <div className="bg-primary border border-dashed border-mid p-md text-[10px] text-secondary">
        <p>
          NOTE: Model files are served from authorized PrismSplit repositories.
          Verification occurs automatically via SHA-256 checksum after download
          completion.
        </p>
      </div>
    </div>
  );
}
