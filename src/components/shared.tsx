// src/components/shared.tsx
import React from "react";

interface ButtonProps {
  children: React.ReactNode;
  onClick?: () => void;
  variant?: "default" | "primary" | "danger";
  disabled?: boolean;
  className?: string;
  type?: "button" | "submit" | "reset";
}

export const Button: React.FC<ButtonProps> = ({
  children,
  onClick,
  variant = "default",
  disabled = false,
  className = "",
  type = "button",
}) => {
  const variantClass = variant !== "default" ? `btn--${variant}` : "";
  return (
    <button
      type={type}
      onClick={onClick}
      disabled={disabled}
      className={`btn ${variantClass} ${className}`}
    >
      {children}
    </button>
  );
};

export const NavButton: React.FC<{
  label: string;
  active: boolean;
  onClick: () => void;
}> = ({ label, active, onClick }) => {
  return (
    <button
      onClick={onClick}
      className={`nav-btn ${active ? "nav-btn--active" : ""}`}
    >
      {label}
    </button>
  );
};

export function Fieldset({
  legend,
  children,
  className = "",
}: {
  legend: string;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <fieldset className={`fieldset ${className}`}>
      <legend>{legend}</legend>
      {children}
    </fieldset>
  );
}

export function Checkbox({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="checkbox-container">
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className="checkbox-input"
      />
      <span className="checkbox-custom"></span>
      <span className="checkbox-label">{label}</span>
    </label>
  );
}

export function HelpIcon({ tooltip }: { tooltip: string }) {
  return (
    <div className="tooltip-container ml-1">
      <div className="w-[14px] h-[14px] bg-primary text-secondary flex items-center justify-center text-[9px] font-bold border border-mid leading-none select-none cursor-help">
        ?
      </div>
      <div className="tooltip-content">{tooltip}</div>
    </div>
  );
}

export const Select: React.FC<{
  label?: string;
  value: string;
  options: string[];
  onChange: (value: string) => void;
  className?: string;
}> = ({ label, value, options, onChange, className = "" }) => {
  return (
    <div className={`select-container ${className}`}>
      {label && <label className="select-label">{label}</label>}
      <div className="select-wrapper">
        <select
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className="input select-input"
        >
          {options.map((opt) => (
            <option key={opt} value={opt}>
              {opt}
            </option>
          ))}
        </select>
      </div>
    </div>
  );
};
