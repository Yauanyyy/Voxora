# ADR 0007: Keep provider credentials outside SQLite

- **Status:** Accepted
- **Decision:** Store cloud ASR and LLM secrets in the Windows platform credential store. SQLite and ordinary files contain only opaque credential references and non-secret configuration.
- **Why:** Separate secrets from transcript/history persistence, exports, backups, diagnostics, and user-editable files.
- **Rejected alternatives:** Plaintext JSON or SQLite secrets; logging request credentials; project-operated secret proxy; embedding credentials in fixtures.
- **Consequences:** Provider adapters resolve an opaque reference through the credential port at request time. Redaction tests must prove that secrets never appear in SQLite, JSON, logs, fixtures, crash reports, exports, or plaintext backups; missing credentials produce sanitized, recoverable failure meaning.
