# Agent SOP: PrismSplit Operations

## Operational Mandates
- **Stability:** Rust GUI backend handles process supervision. Never allow a Python crash to hang the GUI main thread.
- **IPC Safety:** All communication between layers must use the Newline-delimited JSON protocol. Validate all JSON payloads.
- **Embedded Hygiene:** All Python dependencies must be declared in `engine/pyproject.toml` and managed via the app's internal runtime manager.
- **Testing:** Verify the Engine Bridge logic with Rust unit tests and Python engine integration tests.

## Core Workflows
1. **Modifying Inference Logic:**
   - Update the Python engine in `engine/`.
   - Update the JSON protocol in `engine/python/prismsplit_protocol.py` if needed.
   - Mirror the changes in the Rust bridge in `src/engine_bridge.rs`.
2. **Updating the UI:**
   - Adhere to "Dark Brutalism" guidelines (3D beveled borders, Tahoma font).
   - Use MonolithUI theme tokens in `src/theme.rs` and custom widget decorators in `src/widgets.rs`.
   - Stream engine logs to the `log_console` panel in the UI.
3. **Model Management:**
   - Use the `download_manager` in Rust for all server interactions.
   - Verify file integrity using SHA-256 after download.
   - Scan local directories using the `scan_local_models` service in `src/backend.rs` for zero-copy imports.

## Documentation SOP
- Update `CHANGELOG.md` for every release.
- Maintain architectural ADRs in `docs/wiki/architecture.md`.
- Keep `docs/MEMORY.md` updated with the latest project status.

## Related Docs
- [Project Identity](./IDENTITY.md)
- [Project Soul](./SOUL.md)
- [Wiki Index](./wiki/index.md)
- [Architecture](./wiki/architecture.md)