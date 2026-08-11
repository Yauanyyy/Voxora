# ADR 0005: Ports and adapters with inward dependencies

- **Status:** Accepted
- **Decision:** Keep domain and use-case logic behind explicit portable ports. Adapters implement those ports and depend inward; the Tauri composition root selects adapters.
- **Why:** Isolate provider/platform details, make failure meaning portable, and enable fake-port testing before native integration.
- **Rejected alternatives:** A catch-all orchestrator; provider/platform SDKs in `voice-core`; inward crates importing adapter types; UI-owned session state.
- **Consequences:** Capture, recognition, processing, targeting, history, recovery, credentials, model management, and insertion remain separable responsibilities. Adapter errors are translated to sanitized portable stages/codes.
