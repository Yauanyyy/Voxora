# ADR 0006: sherpa-onnx as a separately gated local framework

- **Status:** Accepted
- **Decision:** Plan a sherpa-onnx adapter behind the Recognition Engine port for CPU-capable local recognition, while reviewing each model artifact independently.
- **Why:** The framework is a candidate integration boundary for offline recognition; framework selection alone cannot establish the rights or provenance of model weights.
- **Rejected alternatives:** Treating framework acceptance as model approval; bundling unreviewed weights; silently changing between local artifacts.
- **Consequences:** M8 remains blocked until one exact SenseVoice Small artifact and every accompanying file pass the model checklist for source, revision, size, hash, license, commercial use, redistribution, and integration terms. The application package contains no model weights by default.
