# Project Hygiene

## Coding Standards

### Rust (Backend & Core)
- **Error Handling:** Use `anyhow` for app-level logic. NEVER use `unwrap()`.
- **Process Management:** Ensure Python processes are killed on app exit (handled in `on_exit`).
- **Safety:** Leverage Rust's memory safety and thread-safe primitives (`Arc`, `Mutex`, `tokio::sync`).

### Rust (egui GUI)
- **Styling:** Adhere strictly to the "Dark Brutalism" design language. No rounded corners (`Rounding::ZERO`) or drop shadows.
- **Asynchrony:** Never block the main GUI thread; spawn async tasks using the Tokio runtime, sending update events via `AppMsg` channels.
- **Components:** Reuse theme variables in `src/theme.rs` and the beveled fieldset in `src/widgets.rs`.

### Python (Engine)
- **Modularity:** Keep inference backends decoupled from the main runner logic.
- **Protocol:** Adhere strictly to the `prismsplit_protocol.py` JSON schemas.
- **Version:** Target Python 3.9 - 3.11 for compatibility.

### Git Workflow
- **Commit Messages:** Follow Conventional Commits (`feat:`, `fix:`, `docs:`, `chore:`).
- **Hygienic Commits:** Avoid monolithic commits. Keep changes surgical and focused.

## Documentation
- Maintain `docs/MEMORY.md` with every significant milestone or architectural shift.
- Update the wiki when modifying the engine protocol or UI design patterns.