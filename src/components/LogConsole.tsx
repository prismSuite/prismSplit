// src/components/LogConsole.tsx
import React, { useRef, useEffect } from "react";
import { Fieldset } from "./shared";

type Props = {
  logs: string[];
  onClear: () => void;
};

export function LogConsole({ logs, onClear }: Props) {
  const logEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  return (
    <Fieldset legend="Engine Runtime Output">
      <div className="flex justify-end mb-1">
        <button
          onClick={onClear}
          className="bg-[var(--bg-btn)] text-[var(--text-main)] text-[9px] px-2 py-0.5 border-2 border-t-[var(--border-hilite)] border-l-[var(--border-hilite)] border-b-[var(--border-shadow)] border-r-[var(--border-shadow)] active:bg-[var(--bg-panel)]"
        >
          CLEAR_BUFFER
        </button>
      </div>
      <div className="bg-[var(--bg-console)] font-[Courier,monospace] text-[var(--accent-glow)] text-[11px] p-3 h-48 overflow-y-auto border-2 border-t-[var(--border-shadow-deep)] border-l-[var(--border-shadow-deep)] border-b-[var(--border-hilite-subtle)] border-r-[var(--border-hilite-subtle)] shadow-[inset_0_2px_10px_rgba(0,0,0,0.5)]">
        {logs.map((l, i) => (
          <div key={i} className="mb-0.5 whitespace-pre-wrap">
            <span className="opacity-50 select-none mr-2">[{i.toString().padStart(4, '0')}]</span>
            {l}
          </div>
        ))}
        <div ref={logEndRef} />
      </div>
    </Fieldset>
  );
}
