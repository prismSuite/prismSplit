# Project Soul

## Vision
To demystify and democratize state-of-the-art audio separation by providing a professional, industrial-grade desktop application that handles the "dependency hell" of AI inference for the user.

## Principles
1. **Dependency Isolation:** The user should never need to "install Python" or manage pip environments. The app is self-contained.
2. **Deterministic States:** Every operation (download, extract, verify) must have a clear, verifiable outcome (Success, Failure, Cancelled).
3. **Data Integrity:** Models are verified via SHA-256; local models are identified via MD5. No duplicate data, no corrupted weights.
4. **Industrial Aesthetics:** Respect the skeuomorphic roots of audio engineering. Content is the sun; tools orbit it in a dense, functional layout.
5. **Real-Time Transparency:** Never hide what the engine is doing. Stream logs, telemetry, and progress events immediately.

## Values
- **The "UVR" Heritage:** Honor and extend the research of the Ultimate Vocal Remover team.
- **Windows Mastery:** Build for the platform where the majority of audio engineers work.
- **Surgical Precision:** High-performance Rust orchestrating specialized Python inference.