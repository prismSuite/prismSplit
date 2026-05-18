# Project Hygiene

## Coding Standards

### Rust (Backend)
- **Error Handling:** Use `anyhow` for app-level logic. NEVER use `unwrap()`.
- **Process Management:** Ensure Python processes are killed on app exit.
- **Safety:** Leverage Rust's memory safety and thread-safe primitives (`Arc`, `Mutex`, `tokio::sync`).

### Python (Engine)
- **Modularity:** Keep inference backends decoupled from the main runner logic.
- **Protocol:** Adhere strictly to the `protocol.py` JSON schemas.
- **Version:** Target Python 3.10 exclusively for embedded compatibility.

### TypeScript (Frontend)
- **Styling:** Adhere to the "Dark Brutalism" design language. No rounded corners.
- **IPC:** Use strictly typed interfaces for all Tauri command calls and event listeners.
- **Performance:** Optimize React renders for telemetry-heavy components (LogConsole, ProgressBars).

### Git Workflow
- **Commit Messages:** Follow Conventional Commits (`feat:`, `fix:`, `docs:`, `chore:`).
- **Hygienic Commits:** Avoid monolithic commits. Keep changes surgical and focused.

## Documentation
- Maintain `docs/MEMORY.md` with every significant milestone or architectural shift.
- Update the wiki when modifying the engine protocol or UI design patterns.