# ADR 0003: No project-operated server

- **Status:** Accepted
- **Decision:** Voxora operates as a local desktop application with no project-operated server, account system, cloud sync, telemetry, team service, or server self-hosting component.
- **Why:** Keep audio, text, Prompts, credentials, and history under user control and avoid a mandatory service dependency.
- **Rejected alternatives:** A hosted account/sync backend; a project proxy for provider traffic; an optional server that becomes a release dependency.
- **Consequences:** Cloud recognition/LLM calls go directly to explicit user-configured providers. Application and model updates are manual/user-initiated; there is no application auto-update or background model check.
