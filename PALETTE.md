# PrismSplit - Color Palette

The palette is strictly constrained to varied shades of neutral grays to simulate plastic/metal hardware, with highly saturated accents used *only* for functional feedback (lights, screens, progress).

## Backgrounds (Chassis & Panels)
- `bg-[#383838]` - App/Screen Background (The void behind the window)
- `bg-[#404040]` - Main Window Frame & Status bar (The primary physical chassis)
- `bg-[#303030]` - Active Workspace / Inset panels
- `bg-[#4a4a4a]` - Default Button Surface
- `bg-[#2b2b36]` - Title Bar (Slightly cooler dark gray/blue)
- `bg-[#1e1e1e]` - Input Fields / Select dropdowns
- `bg-[#000000]` - Console Window & Off-state indicator lights

## Text & Typography
- `text-[#d4d4d4]` - Primary Application Text
- `text-[#f0f0f0]` - Highlighted Text / Titles
- `text-[#888888]` - Disabled Text, Muted Labels, Placeholder Text
- `text-[#00ff00]` - Console output and Success states (Terminal Green)

## 3D Bevels (Borders)
Depth requires pairs of Highlight (Light) and Shadow (Dark).

**Highlight Colors (Top & Left edges)**
- `border-[#666666]` - Standard button/panel highlight
- `border-[#555555]` - Subdued highlight (used for bottom/right of inset elements)

**Shadow Colors (Bottom & Right edges)**
- `border-[#222222]` - Standard button/panel shadow
- `border-[#111111]` - Deep shadow (used for top/left of inset elements)
- `border-[#1a1a1a]` - Structural chassis dividing lines

## Accents (LEDs & Signals)
Used extremely sparingly to draw the eye to status changes.
- `bg-[#00ff00]` / `text-[#00ff00]` - "System OK" / Active Checkbox / Progress Bar Fill
- `bg-[#00aa00]` - Progress bar shadow/bevel
- `border-[#88aaff]` - Input Focus outline / Window Icon (Subtle blue)
