# PrismSplit - Design System

## Philosophy: "Industrial Audio Skeuomorphism" & "Dark Brutalism"
The design language of Nexus Audio is intentionally built to resemble professional, old-school desktop audio software (DAWs, VST plugins, hardware extraction tools from the late 90s and early 2000s) but translated into a modern dark mode. 

It explicitly avoids modern web design trends:
- **NO** rounded corners (except perhaps implied sub-pixel rendering, but CSS is strictly square `rounded-none`).
- **NO** soft drop shadows (only hard, 1px or 2px directional inset/outset borders).
- **NO** excessive padding. Information density is high.
- **NO** generic sans-serif fonts like Inter or Roboto. We use system-level, aliased-looking fonts like `Tahoma`.

## Key UI Components

### 1. 3D Beveled Borders
Depth is created exclusively through hard-edged border colors, simulating light hitting physical plastic or metal chassis from the top-left.
- **Outset (Raised)**: Light top/left (`#666`), dark bottom/right (`#222`). Used for unpressed buttons and the main window frame.
- **Inset (Sunken)**: Dark top/left (`#111`), light bottom/right (`#555`). Used for workspaces, text inputs, pressed buttons, and the console.

### 2. Typography
- **Primary UI Text**: `Tahoma, sans-serif`. Sized small (`11px` or `12px`). This gives the application a native, Win32/VST feel.
- **Console / Data**: `Courier` or monospace. High contrast (bright green on pitch black).

### 3. Controls (Forms & Buttons)
- **Buttons**: Chunky, gray, with active states that "press down" by swapping outset borders to inset borders and slightly shifting the background color darker.
- **Inputs & Selects**: Sunken fields (`#1e1e1e` background) with inset borders. Focus states use a subtle blue border instead of a glow.
- **Checkboxes**: Custom square drawn controls. A black sunken square that fills with a glowing green pixel block when checked.
- **Fieldsets**: Grouped parameters are enclosed in strict `fieldset` borders with a legend that breaks the top border line.

### 4. Feedback & Status
- **Progress Bars**: High contrast, blocky, bright green (`#00ff00`).
- **Logs**: A terminal-like output window at the bottom of the tool is mandatory for professional trust. Users want to see the "matrix" of what the engine is doing.
