# NEXUS AUDIO - Design System
## "Industrial Audio Skeuomorphism meets Dark Brutalism"

---

## **FILOSOFÍA CENTRAL**

Replicar la **verdad material** de hardware de audio profesional de finales de los 90s/2000s (DAWs, VSTs, mezcladores físicos) mediante **dark mode brutal** y **bordes hard-edged**. 

La intención es que se vea como una **herramienta de producción seria**, no como una aplicación web moderna. Esto es honesto sobre su propósito.

---

## **PALETA DE COLORES**

### Colores Base
- **Fondo Primario**: `#1a1a1a` (casi negro, imperceptible textura)
- **Fondo Secundario**: `#0d0d0d` (más oscuro, para contraste)
- **Superficie Sunken**: `#1e1e1e` (inputs, workspaces, console)
- **Texto Primario**: `#f0f0f0` (blanco roto, no puro)
- **Texto Secundario**: `#a0a0a0` (gris claro, labels)
- **Texto Deshabilitado**: `#5a5a5a` (gris oscuro)

### Acentos & Feedback
- **Accent Verde (CRT)**: `#00ff00` (puro, solo para estados activos/críticos)
- **Accent Verde Oscuro**: `#2d5016` (background hover sutil)
- **Accent Azul (Focus)**: `#0099ff` (subtil, para focus states)
- **Accent Rojo**: `#ff3333` (errores, críticos)
- **Accent Amarillo**: `#ffcc00` (warnings, análisis)
- **Gris Industrial**: `#4a4a4a` (bordes, dividers)

### Bordes 3D (Skeuomorphism)
- **Luz (Raised)**: `#666666` (top/left en botones sin presionar)
- **Sombra (Sunken)**: `#111111` (bottom/right en botones sin presionar)
- **Inset Oscuro**: `#222222` (inputs, campos sunken)
- **Inset Claro**: `#555555` (inner highlights en campos sunken)

### Restricciones Cromáticas
- ❌ NO gradientes suavizados
- ❌ NO colores pasteles
- ❌ NO sombras blandas (drop-shadow)
- ❌ NO blur o glassmorphism
- ✅ Paleta monocromática + acentos puntuales
- ✅ Hard borders, directional lighting simulation

---

## **TIPOGRAFÍA**

### Familia Tipográfica

#### UI/Controls (Sistema)
- **Fuente**: `Tahoma, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif`
- **Tamaño**: `11px` o `12px` (compacto, denso)
- **Weight**: 400 (regular), 700 (bold en labels)
- **Razón**: Sensación Win32/VST auténtica, aliased-looking

#### Display/Titles
- **Fuente**: `IBM Plex Sans, 'Courier New', monospace` (para emphasis)
- **Tamaño**: `14px` (subtítulos), `16px` (títulos principales)
- **Weight**: 600, 700 (solo para jerarquía visual clara)

#### Consola/Data (Terminal)
- **Fuente**: `'Cascadia Mono', 'Courier New', monospace`
- **Tamaño**: `10px` o `11px`
- **Color**: `#00ff00` sobre `#0d0d0d`
- **Razón**: Efecto matrix, transparencia técnica

#### Monoespaciada para UI (Secundaria)
- **Fuente**: `'Fira Code', 'IBM Plex Mono', monospace`
- **Tamaño**: `12px`
- **Uso**: Labels técnicos, parámetros numéricos, status
- **Razón**: Linaje ricing Linux + legibilidad de código

### Reglas Tipográficas
- Line-height: `1.4` (compacto, sin exceso)
- Letter-spacing: `-0.2px` (denso, industrial)
- NO transformaciones (text-transform: uppercase) a menos que sea muy específico
- Alineación: Izquierda (top-left alignment en labels)

---

## **COMPONENTES UI**

### 1. BOTONES

#### Estado Default (Raised/Unpressed)
```css
background: #4a4a4a;
color: #f0f0f0;
border: 2px solid;
border-color: #666 #222 #222 #666; /* top-left light, bottom-right dark */
padding: 6px 12px;
font-size: 11px;
font-family: Tahoma, sans-serif;
font-weight: 600;
cursor: pointer;
transition: all 60ms linear;
```

#### Estado Hover
```css
background: #555555;
border-color: #777 #111 #111 #777; /* más pronunciado */
```

#### Estado Active/Pressed (Inset)
```css
background: #3a3a3a; /* más oscuro */
border-color: #222 #666 #666 #222; /* invertido: dark top/left */
transform: translate(1px, 1px); /* micro-offset, efecto de presión */
```

#### Estado Disabled
```css
background: #2a2a2a;
color: #5a5a5a;
border-color: #444 #333 #333 #444;
cursor: not-allowed;
opacity: 0.6;
```

#### Variantes
- **Primary (Verde)**: `background: #2d5016; border-color: #00ff00 #1a1a1a #1a1a1a #00ff00`
- **Danger (Rojo)**: `background: #661111; border-color: #ff3333 #111 #111 #ff3333`
- **Compact**: `padding: 4px 8px; font-size: 10px`

---

### 2. INPUTS & TEXTAREAS

#### Default State (Sunken/Inset)
```css
background: #1e1e1e;
color: #f0f0f0;
border: 2px solid;
border-color: #222 #555 #555 #222; /* dark top/left, light bottom/right */
padding: 6px 8px;
font-size: 11px;
font-family: Tahoma, sans-serif;
outline: none;
transition: border-color 60ms linear;
```

#### Focus State
```css
border-color: #0099ff #0099ff #0099ff #0099ff; /* blue outline sutil */
box-shadow: inset 0 0 0 1px #0099ff; /* NO drop-shadow, solo inset */
```

#### Placeholder
```css
color: #5a5a5a;
font-style: italic;
```

#### Error State
```css
border-color: #ff3333 #ff3333 #ff3333 #ff3333;
background: #2a1a1a; /* rojo oscuro sutil */
```

---

### 3. CHECKBOXES & RADIO BUTTONS

#### Checkbox (Custom, Square)
- **Unchecked**: Sunken square negro `#1e1e1e` con inset border
- **Checked**: Fondo `#2d5016`, contenido `✓` en verde `#00ff00` (14px, bold)
- **Size**: `16x16px`
- **Border**: `1px inset #222/#555`

#### Radio (Custom, Square with Dot)
- Similar a checkbox pero con punto interior en lugar de check
- **Unchecked**: `#1e1e1e` 
- **Checked**: Punto verde `#00ff00` centered

---

### 4. SELECTS & DROPDOWNS

#### Container (Trigger Button)
```css
background: #4a4a4a;
border: 2px solid #666 #222 #222 #666;
padding: 6px 8px;
font-size: 11px;
cursor: pointer;
display: flex;
justify-content: space-between;
align-items: center;
```

#### Dropdown Menu (Sunken)
```css
background: #1e1e1e;
border: 1px solid #4a4a4a;
box-shadow: none; /* NO drop-shadow */
position: absolute;
z-index: 1000;
```

#### Menu Items
```css
padding: 4px 8px;
color: #f0f0f0;
font-size: 11px;
cursor: pointer;
```

#### Menu Item Hover
```css
background: #2d5016;
color: #00ff00;
```

---

### 5. FIELDSETS & GROUPED CONTROLS

```css
border: 1px solid #4a4a4a;
padding: 8px;
margin: 8px 0;

legend {
  font-size: 11px;
  font-weight: 700;
  color: #f0f0f0;
  padding: 0 4px;
  margin-left: -4px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
```

---

### 6. PROGRESS BARS & SLIDERS

#### Progress Bar
```css
height: 16px;
background: #1e1e1e;
border: 1px solid #4a4a4a;
fill: linear-gradient(90deg, #2d5016, #00ff00);
color: #00ff00;
font-size: 9px;
text-align: center;
line-height: 16px;
```

#### Slider (Range Input)
```css
/* Track */
background: #1e1e1e;
border: 1px solid #4a4a4a;
height: 4px;

/* Thumb */
background: #555555;
border: 2px solid #666 #222 #222 #666;
width: 14px;
height: 14px;
cursor: grab;
```

---

### 7. PANELS & WORKSPACES

#### Container Base
```css
background: #0d0d0d;
border: 1px solid #4a4a4a;
padding: 8px;
```

#### Sunken Workspace (como console)
```css
background: #1e1e1e;
border: 2px solid;
border-color: #222 #555 #555 #222;
padding: 8px;
overflow-y: auto;
```

#### Resizable Divider
```css
background: #4a4a4a;
width: 4px; /* vertical */ o height: 4px; /* horizontal */
cursor: col-resize; /* o row-resize */
hover: background: #00ff00;
```

---

### 8. TOP BAR / MENU BAR

```css
background: #1a1a1a;
border-bottom: 1px solid #4a4a4a;
height: 48px;
display: flex;
align-items: center;
padding: 0 12px;
gap: 16px;
position: fixed;
top: 0;
left: 0;
right: 0;
z-index: 2000;
```

#### Menu Items
```css
color: #f0f0f0;
font-size: 11px;
font-weight: 600;
cursor: pointer;
padding: 6px 8px;
border: 1px solid transparent;
```

#### Menu Item Hover/Active
```css
border: 1px solid #4a4a4a;
background: #2d5016;
color: #00ff00;
```

---

### 9. CONSOLE / STATUS PANEL (Terminal-Like)

#### Container
```css
background: #0d0d0d;
border-top: 1px solid #4a4a4a;
height: 120px;
overflow-y: auto;
font-family: 'Cascadia Mono', monospace;
font-size: 10px;
color: #00ff00;
padding: 8px;
line-height: 1.6;
```

#### Log Entries
```css
margin: 2px 0;
word-wrap: break-word;
white-space: pre-wrap;
```

#### Levels
- **Info**: `color: #00ff00`
- **Warn**: `color: #ffcc00`
- **Error**: `color: #ff3333`

#### Opcional: CRT Scanline Effect
```css
background-image: 
  repeating-linear-gradient(
    0deg,
    rgba(0, 0, 0, 0.15),
    rgba(0, 0, 0, 0.15) 1px,
    transparent 1px,
    transparent 2px
  );
```

---

## **SPACING & LAYOUT**

### Grid System
- **Base Unit**: `8px`
- **Padding**: `8px`, `16px`, `24px`, `32px`
- **Margin**: `8px`, `16px`, `24px`, `32px`
- **Gap (Flex/Grid)**: `8px`, `12px`, `16px`

### Breakpoints (Para Tauri, usualmente irrelevante, pero si responsive)
- Mobile: `< 600px` (unlikely en desktop audio apps)
- Tablet: `600px - 1024px`
- Desktop: `> 1024px`

### Negative Space Philosophy
- **High Density**: OK para audio apps (mucha info visible)
- **Breathing Room**: Entre secciones principales (16px+)
- **Compact Vertical**: Labels + controls en 24px max

### Alignment Rules
- **Top-Left Bias**: Labels siempre arriba-izquierda de inputs
- **Vertical Stacking**: Secciones apiladas (no lado-a-lado si cabe)
- **Justified Grids**: Datos en tablas con bordes 1px

---

## **INTERACCIONES & ANIMACIONES**

### Duraciones
- **Micro-interactions** (hover, focus): `60ms` linear
- **Transiciones de Panel**: `120ms` ease-in-out
- **Loading**: `500ms` - `1s` (loops sin límite)

### Transiciones Permitidas
```css
/* Button Press */
transition: all 60ms linear;

/* Panel Open/Close */
transition: opacity 120ms ease-in-out;

/* Input Focus */
transition: border-color 60ms linear;
```

### Animaciones Base
```css
/* Loading Spinner (Blocky) */
@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* Pulse (Status Indicator) */
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

/* Scanline (Optional CRT Effect) */
@keyframes scanline-drift {
  0% { transform: translateY(0); }
  100% { transform: translateY(2px); }
}
```

---

## **ICONOGRAFÍA**

### Estilo
- **Outline**: `1px stroke`, sin fill
- **Size**: `16x16px` (estándar), `20x20px` (grandes), `12x12px` (inline)
- **Color**: Hereda del texto (`currentColor`) o acentos específicos
- **Diseño**: Minimalista, geométrico, sin curvas suavizadas

### Set Mínimo Requerido
- ✓ Play, Pause, Stop
- ✓ Record, Mute, Solo
- ✓ Save, Load, Export
- ✓ Settings, Info
- ✓ Error, Warning, Success
- ✓ Close, Collapse, Expand
- ✓ Slider Handles (custom shapes)

---

## **ESTADOS VISUALES**

### Default
- Sin hover, sin focus
- Colores base definidos arriba

### Hover
- Brillo ligero (+1 shade)
- Border más pronunciado
- Cursor: `pointer` en elementos interactivos
- Duración: `60ms`

### Focus
- **Input**: Borde azul `#0099ff`
- **Button**: Borde más claro
- **Outline**: Opcional (NO default, solo accessibility)
- Duración: instant

### Active/Pressed
- Inversión de bordes 3D (inset en lugar de outset)
- Background más oscuro
- Micro-offset visual (1px translate)

### Disabled
- Opacidad: `0.6`
- Cursor: `not-allowed`
- Color texto: `#5a5a5a`

### Error State
- Border rojo `#ff3333`
- Background sutil rojo-oscuro
- Mensaje en rojo claro

### Loading State
- Spinner animado (60ms rotate loop)
- Color: Verde `#00ff00` o azul `#0099ff`
- Opcional: Pulse en label

---

## **ACCESIBILIDAD**

### Contrastes Mínimos (WCAG AA)
- **Texto Primario** (`#f0f0f0` sobre `#1a1a1a`): **17:1** ✓
- **Texto Secundario** (`#a0a0a0` sobre `#1a1a1a`): **7:1** ✓
- **Verde Accent** (`#00ff00` sobre `#0d0d0d`): **10:1** ✓

### Requisitos
- ✓ Todos los inputs deben tener `<label>` o `aria-label`
- ✓ Focus visible EN TODOS los elementos interactivos
- ✓ Color NO es el único indicador de estado (usar iconografía + texto)
- ✓ Tabindex navegable (no traps, orden lógico)
- ✓ Alt text en imágenes (si existen)

### Testeo
- Validar con Contrast Checker
- Navegar solo con teclado
- Screen reader testing (NVDA, JAWS)

---

## **CSS VARIABLES (ROOT)**

```css
:root {
  /* Colors - Base */
  --color-bg-primary: #1a1a1a;
  --color-bg-secondary: #0d0d0d;
  --color-bg-sunken: #1e1e1e;
  --color-fg-primary: #f0f0f0;
  --color-fg-secondary: #a0a0a0;
  --color-fg-disabled: #5a5a5a;

  /* Colors - Accents */
  --color-accent-green: #00ff00;
  --color-accent-green-dark: #2d5016;
  --color-accent-blue: #0099ff;
  --color-accent-red: #ff3333;
  --color-accent-yellow: #ffcc00;

  /* Colors - Borders */
  --color-border-light: #666666;
  --color-border-dark: #111111;
  --color-border-mid: #4a4a4a;
  --color-border-inset-light: #555555;
  --color-border-inset-dark: #222222;

  /* Typography */
  --font-ui: Tahoma, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  --font-display: 'IBM Plex Sans', 'Courier New', monospace;
  --font-mono: 'Cascadia Mono', 'Courier New', monospace;
  --font-mono-code: 'Fira Code', 'IBM Plex Mono', monospace;
  --font-size-xs: 10px;
  --font-size-sm: 11px;
  --font-size-base: 12px;
  --font-size-lg: 14px;
  --font-size-xl: 16px;
  --font-weight-regular: 400;
  --font-weight-bold: 700;
  --line-height-tight: 1.4;
  --letter-spacing-tight: -0.2px;

  /* Spacing */
  --spacing-xs: 4px;
  --spacing-sm: 8px;
  --spacing-md: 16px;
  --spacing-lg: 24px;
  --spacing-xl: 32px;

  /* Borders */
  --border-width: 1px;
  --border-width-thick: 2px;

  /* Transitions */
  --transition-fast: 60ms linear;
  --transition-normal: 120ms ease-in-out;
  --transition-slow: 300ms ease-in-out;

  /* Shadows - NONE (only borders) */
  /* No drop-shadow, no blur, no inset shadow */

  /* Z-index Scale */
  --z-dropdown: 1000;
  --z-modal: 1500;
  --z-topbar: 2000;
}
```

---

## **RESET/NORMALIZATION**

```css
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body {
  width: 100%;
  height: 100%;
  background: var(--color-bg-primary);
  color: var(--color-fg-primary);
  font-family: var(--font-ui);
  font-size: var(--font-size-sm);
  line-height: var(--line-height-tight);
  letter-spacing: var(--letter-spacing-tight);
  overflow: hidden; /* Tauri app, no scroll */
}

button, input, select, textarea {
  font-family: inherit;
  font-size: inherit;
  color: inherit;
  background: transparent;
  border: none;
  padding: 0;
}

/* Remove default focus outlines, we define custom */
button:focus, input:focus, select:focus, textarea:focus {
  outline: none;
}
```

---

## **EJEMPLO: COMPONENTE BOTÓN (TSX)**

```tsx
import React from 'react';
import './Button.css';

interface ButtonProps {
  children: React.ReactNode;
  onClick?: () => void;
  variant?: 'default' | 'primary' | 'danger';
  disabled?: boolean;
  className?: string;
  type?: 'button' | 'submit' | 'reset';
}

export const Button: React.FC<ButtonProps> = ({
  children,
  onClick,
  variant = 'default',
  disabled = false,
  className = '',
  type = 'button',
}) => {
  return (
    <button
      type={type}
      onClick={onClick}
      disabled={disabled}
      className={`btn btn--${variant} ${disabled ? 'btn--disabled' : ''} ${className}`}
    >
      {children}
    </button>
  );
};
```

```css
/* Button.css */
.btn {
  background: var(--color-border-mid);
  color: var(--color-fg-primary);
  border: 2px solid;
  border-color: var(--color-border-light) var(--color-border-dark) var(--color-border-dark) var(--color-border-light);
  padding: 6px 12px;
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-bold);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn:hover:not(.btn--disabled) {
  background: var(--color-border-light);
  border-color: #777 #111 #111 #777;
}

.btn:active:not(.btn--disabled) {
  background: #3a3a3a;
  border-color: var(--color-border-dark) var(--color-border-light) var(--color-border-light) var(--color-border-dark);
  transform: translate(1px, 1px);
}

.btn--disabled {
  background: #2a2a2a;
  color: var(--color-fg-disabled);
  border-color: #444 #333 #333 #444;
  cursor: not-allowed;
  opacity: 0.6;
}

.btn--primary {
  background: var(--color-accent-green-dark);
  border-color: var(--color-accent-green) var(--color-bg-primary) var(--color-bg-primary) var(--color-accent-green);
}

.btn--primary:hover:not(.btn--disabled) {
  background: #3d6018;
  border-color: #00ff00 #0a0a0a #0a0a0a #00ff00;
}

.btn--danger {
  background: #661111;
  border-color: var(--color-accent-red) var(--color-bg-primary) var(--color-bg-primary) var(--color-accent-red);
}

.btn--danger:hover:not(.btn--disabled) {
  background: #881111;
  border-color: #ff3333 #0a0a0a #0a0a0a #ff3333;
}
```

---

## **CONSTRUCCIÓN DE VISTAS COMPLEJAS**

### Estructura Base (Tauri Window)
```
┌─────────────────────────────────────┐
│ [App Title] [Menu 1] [Menu 2] [⚙️]  │ ← TopBar (48px)
├─────────────────────────────────────┤
│                                       │
│  [Panel Left] | [Workspace] | [Ctrl] │ ← Main Content (resizable dividers)
│                                       │
├─────────────────────────────────────┤
│ [Status] [Info] | [Console Log...] │ ← Console (120px)
└─────────────────────────────────────┘
```

### Implementación (Estructura Flexbox)
```tsx
<div className="app-container">
  <TopBar />
  <main className="main-content">
    <aside className="panel-left">
      {/* Controls */}
    </aside>
    <div className="divider-v" />
    <section className="workspace">
      {/* Main UI */}
    </section>
    <div className="divider-v" />
    <aside className="panel-right">
      {/* Params, Settings */}
    </aside>
  </main>
  <Console />
</div>
```

```css
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--color-bg-primary);
}

.main-content {
  display: flex;
  flex: 1;
  gap: 0;
  overflow: hidden;
  padding-top: 48px; /* TopBar offset */
}

.panel-left, .workspace, .panel-right {
  overflow-y: auto;
  border: 1px solid var(--color-border-mid);
  background: var(--color-bg-secondary);
}

.divider-v {
  width: 4px;
  background: var(--color-border-mid);
  cursor: col-resize;
  transition: background var(--transition-fast);
}

.divider-v:hover {
  background: var(--color-accent-green);
}
```

---

## **NO-NOs (Verificación de Pureza)**

- ❌ Gradientes suavizados (`linear-gradient(45deg, #fff, #000)`)
- ❌ Border-radius > 0 (excepción: 2px máximo en casos muy específicos)
- ❌ Box-shadow o drop-shadow
- ❌ Blur, backdrop-filter, glassmorphism
- ❌ Colores pasteles, paleta "amigable"
- ❌ Fuentes genéricas: Inter, Roboto, sans-serif naked
- ❌ Transiciones > 120ms (feels sluggish)
- ❌ Smooth scrolling en UI (scrolling behavior: smooth)
- ❌ Custom scrollbars (os-native es mejor)

---

## **REFERENCIAS VISUALES**

### Inspiraciones Verificadas
1. **WindowMaker** (1990s): Iconografía minimalist, bordes hard, paleta monocromática
2. **Cubase 5.0** (2009): VST windows brutales, 3D bevels, información densa
3. **Ableton Live 9** (2012): Dark UI, grid obsesivo, sans-serif compacta
4. **Linux Ricing (r/unixporn)**: i3, Hyprland, Solarized Dark, Nord palettes
5. **NeXTSTEP/Cocoa (80s-90s)**: Purismo de controles, alignment riguroso
6. **Industrial Design Hardware**: Botones físicos, lighting direccional, honestidad material

---

## **CHECKLIST DE IMPLEMENTACIÓN**

- [ ] CSS Variables definidas en `:root`
- [ ] Todos los botones con 3D bevel (outset/inset)
- [ ] Inputs con border inset (dark top/left)
- [ ] Topbar fixed 48px con menu items
- [ ] Console panel con font monoespaciada
- [ ] Dividers resizables (`cursor: col-resize` / `row-resize`)
- [ ] Focus states azules en inputs
- [ ] Transiciones 60ms max
- [ ] NO drop-shadows en componentes
- [ ] Iconografía 1px stroke
- [ ] Tabindex navegable
- [ ] WCAG AA contrast ratio verificado
- [ ] Paleta monocromática + 3 acentos máximo

---

## **CONCLUSIÓN**

Esta UI es **brutalmente funcional**. No busca agradar visualmente en el sentido moderno, sino **generar confianza** mediante honestidad material y densidad de información. Es la interfaz de una herramienta seria, no un juguete web.

**La verdad es el mejor diseño.**
