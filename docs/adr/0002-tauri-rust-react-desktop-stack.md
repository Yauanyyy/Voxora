# ADR 0002: Tauri 2, Rust, React, and TypeScript

- **Status:** Accepted
- **Decision:** Use a Tauri 2 desktop composition root with Rust application/domain code and a React + TypeScript UI.
- **Why:** Rust provides a portable, testable core and explicit adapter boundary; Tauri supplies the Windows desktop shell; React/TypeScript render state and settings effectively.
- **Rejected alternatives:** A browser-only product requiring a server; a UI-owned session orchestrator; coupling portable code directly to a frontend framework.
- **Consequences:** Tauri maps commands/events and composes selected adapters. React submits commands and renders state but never owns Dictation Session coordination. M1 creates no workspace or runtime dependency.
