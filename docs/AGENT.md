# Agent SOP: PrismSplit Operations

## Operational Mandates
- **Stability:** Rust backend handles process supervision. Never allow a Python crash to hang the Tauri main thread.
- **IPC Safety:** All communication between layers must use the Newline-delimited JSON protocol. Validate all JSON payloads.
- **Embedded Hygiene:** All Python dependencies must be declared in `engine/pyproject.toml` and managed via the app's internal runtime manager.
- **Testing:** Verify the Engine Bridge logic with Rust unit tests and Python engine integration tests.

## Core Workflows
1. **Modifying Inference Logic:**
   - Update the Python engine in `engine/`.
   - Update the JSON protocol in `engine/python/protocol.py` if needed.
   - Mirror the changes in the Rust bridge in `src-tauri/src/engine_bridge/`.
2. **Updating the UI:**
   - Adhere to "Dark Brutalism" guidelines (3D beveled borders, Tahoma font).
   - Use established CSS variables for themes.
   - Stream engine logs to the `LogConsole` component.
3. **Model Management:**
   - Use the `DownloadManager` in Rust for all server interactions.
   - Verify file integrity using SHA-256 after download.
   - Scan local directories using the `Scanner` service for zero-copy imports.

## Documentation SOP
- Update `CHANGELOG.md` for every release.
- Maintain architectural ADRs in `docs/wiki/architecture.md`.
- Keep `docs/MEMORY.md` updated with the latest project status.

## Related Docs
- [Project Identity](./IDENTITY.md)
- [Project Soul](./SOUL.md)
- [Wiki Index](./wiki/index.md)
- [Architecture](./wiki/architecture.md)