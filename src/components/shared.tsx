// src/components/shared.tsx
import React from "react";

export function ToolbarButton({
  label,
  active,
  onClick,
}: {
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`px-3 py-1 font-bold outline-none
        ${
          active
            ? "bg-[var(--bg-workspace)] text-[var(--text-bright)] border-2 border-t-[var(--border-shadow-deep)] border-l-[var(--border-shadow-deep)] border-b-[var(--border-hilite-subtle)] border-r-[var(--border-hilite-subtle)]"
            : "bg-[var(--bg-btn)] text-[var(--text-main)] border-2 border-t-[var(--border-hilite)] border-l-[var(--border-hilite)] border-b-[var(--border-shadow)] border-r-[var(--border-shadow)] hover:bg-[var(--bg-btn-hover)] active:bg-[var(--bg-panel)] active:border-t-[var(--border-shadow-deep)] active:border-l-[var(--border-shadow-deep)] active:border-b-[var(--border-hilite-subtle)] active:border-r-[var(--border-hilite-subtle)]"
        }`}
    >
      {label}
    </button>
  );
}

export function Fieldset({
  legend,
  children,
}: {
  legend: string;
  children: React.ReactNode;
}) {
  return (
    <fieldset className="border-2 border-t-[var(--border-fieldset)] border-l-[var(--border-fieldset)] border-b-[var(--border-fieldset-subtle)] border-r-[var(--border-fieldset-subtle)] p-3 pt-4 relative mt-2">
      <legend className="bg-[var(--bg-workspace)] px-2 absolute -top-2 left-2 text-[var(--text-subtext)] font-bold">
        {legend}
      </legend>
      {children}
    </fieldset>
  );
}

export function ProButton({
  label,
  onClick,
}: {
  label: string;
  onClick?: () => void;
}) {
  return (
    <button
      onClick={onClick}
      type="button"
      className="bg-[var(--bg-btn)] text-[var(--text-main)] border-2 border-t-[var(--border-hilite)] border-l-[var(--border-hilite)] border-b-[var(--border-shadow)] border-r-[var(--border-shadow)] px-2 py-[2px] active:bg-[var(--bg-panel)] active:border-t-[var(--border-shadow-deep)] active:border-l-[var(--border-shadow-deep)] active:border-b-[var(--border-hilite-subtle)] active:border-r-[var(--border-hilite-subtle)]"
    >
      {label}
    </button>
  );
}

export function Checkbox({
  label,
  defaultChecked = false,
}: {
  label: string;
  defaultChecked?: boolean;
}) {
  return (
    <label className="flex items-center gap-2 cursor-pointer">
      <span className="relative w-3 h-3 bg-[var(--bg-input)] border border-[#111] shadow-[inset_0_1px_2px_rgba(0,0,0,0.8)] inline-block">
        <input
          type="checkbox"
          defaultChecked={defaultChecked}
          className="opacity-0 absolute inset-0 cursor-pointer peer"
        />
        <span className="absolute inset-0 bg-[var(--accent-glow)] hidden peer-checked:block m-[2px] shadow-[0_0_3px_var(--accent-glow)]"></span>
      </span>
      {label}
    </label>
  );
}

export function HelpIcon({ tooltip }: { tooltip: string }) {
  return (
    <div className="relative group inline-block cursor-help align-middle ml-1">
      <div className="w-[13px] h-[13px] bg-[#666] text-[var(--text-bright)] flex items-center justify-center text-[9px] font-bold border border-[#222] border-t-[#aaa] border-l-[#aaa] pb-[1px] leading-none shadow-sm group-hover:bg-[#888] select-none">
        ?
      </div>
      <div className="absolute bottom-full left-1/2 transform -translate-x-1/2 mb-1 w-48 p-1.5 bg-[#ffffe1] text-[#000] border border-[#000] text-[10px] hidden group-hover:block z-50 shadow-[2px_2px_0px_rgba(0,0,0,0.5)] pointer-events-none leading-normal font-[Tahoma,sans-serif] text-left">
        {tooltip}
      </div>
    </div>
  );
}
