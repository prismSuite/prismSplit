// src/components/LogConsole.tsx
import React, { useRef, useEffect } from "react";
import { Button } from "./shared";

type Props = {
  logs: string[];
  onClear: () => void;
};

export function LogConsole({ logs, onClear }: Props) {
  const logEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "instant" });
  }, [logs]);

  return (
    <div className="console-panel">
      <div className="flex justify-between items-center bg-primary border-b-mid px-sm py-xs">
        <div className="flex items-center gap-xs">
          <div className="w-1.5 h-1.5 bg-accent-red animate-pulse rounded-full"></div>
          <span className="text-[10px] font-bold uppercase tracking-wider text-secondary">
            TTY0 // Engine Output
          </span>
        </div>
        <Button onClick={onClear} className="text-[9px] px-sm py-0.5">
          CLEAR_BUFFER
        </Button>
      </div>
      <div className="sunken-panel flex-1 font-mono text-[10px] text-accent-green p-sm leading-tight relative">
        {logs.map((l, i) => {
          let colorClass = "text-accent-green";
          if (l.includes("ERROR") || l.includes("ERR:"))
            colorClass = "text-accent-red";
          if (l.includes("WARN")) colorClass = "text-accent-yellow";

          return (
            <div key={i} className={`mb-0.5 whitespace-pre-wrap ${colorClass}`}>
              <span className="opacity-40 select-none mr-2">
                [{i.toString().padStart(4, "0")}]
              </span>
              {l}
            </div>
          );
        })}
        <div className="inline-block w-2 h-3 bg-accent-green animate-blink ml-1 align-middle"></div>
        <div ref={logEndRef} />
      </div>
    </div>
  );
}
