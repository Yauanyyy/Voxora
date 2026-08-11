# ADR 0001: Windows-first delivery with a portable core

- **Status:** Accepted
- **Decision:** Deliver the first release for Windows while keeping `voice-core`, ports, and application use cases portable and free of Windows-specific conditionals or types.
- **Why:** Windows is the initial user and integration target, but portable lifecycle and domain behavior must remain testable on other systems and independent of native APIs.
- **Rejected alternatives:** A Windows-only core; implementing multiple desktop platforms before the first release; leaking Windows types into shared code.
- **Consequences:** Native capture, shortcuts, targeting, credentials, and insertion stay in `platform-windows`. Cross-platform adapters may be added after first release without redesigning the domain boundary.
