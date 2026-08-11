# Roadmap Index

The milestone order, entry gates, acceptance criteria, and completion definition in [`docs/implementation-plan.md`](implementation-plan.md) are the sole delivery authority. This page is only a navigation index and must not become a second implementation plan. All milestones remain subject to the independent-implementation, privacy, recovery, dependency, model, and licensing constraints in the master plan.

| Milestone | Planned focus | Authority |
| --- | --- | --- |
| M0 | Governance, shared language, repository rules, agent roles, and master plan. | [Implementation plan — M0](implementation-plan.md#m0--governance-and-master-plan) |
| M1 | Product, architecture, lifecycle, testing, licensing, roadmap, and ADR baseline. | [Implementation plan — M1](implementation-plan.md#m1--product-architecture-licensing-and-decision-baseline) |
| M2 | Smallest buildable Rust/Tauri/React workspace and CI skeleton without speculative adapters. | [Implementation plan — M2](implementation-plan.md#m2--workspace-and-ci-skeleton) |
| M3 | Portable domain, ports, session state machine, fake adapters, and exhaustive lifecycle tests. | [Implementation plan — M3](implementation-plan.md#m3--portable-domain-and-session-state-machine) |
| M4 | SQLite settings/history, separate audio artifacts, retention/deletion, credential adapter, Prompt/rule catalogs, and Hotword groups. | [Implementation plan — M4](implementation-plan.md#m4--local-persistence-and-configuration) |
| M5 | Desktop UX with fake adapters, tray/settings/history surfaces, overlay, Result Panel, and frontend verification. | [Implementation plan — M5](implementation-plan.md#m5--desktop-ux-with-fake-adapters) |
| M6 | Windows audio, shortcuts, targeting, clipboard/SendInput insertion, and safe native fallbacks. | [Implementation plan — M6](implementation-plan.md#m6--windows-audio-shortcuts-targeting-and-insertion) |
| M7 | Doubao streaming ASR and OpenAI-compatible stateless processing after review. | [Implementation plan — M7](implementation-plan.md#m7--doubao-asr-and-openai-compatible-processing) |
| M8 | CPU-capable local ASR and user-controlled model manager after an exact SenseVoice artifact gate. | [Implementation plan — M8](implementation-plan.md#m8--local-asr-and-model-manager) |
| M9 | Windows-first release hardening, packaging, accessibility/privacy/recovery checks, notices, and release guidance. | [Implementation plan — M9](implementation-plan.md#m9--release-hardening) |
| M10 | Post-first-release candidates: Hotword Candidates, additional models/providers/rules, direct audio export, and non-Windows adapters. | [Implementation plan — M10](implementation-plan.md#m10--post-first-release-candidates) |

Current repository status is documentation/design stage. M1 does not claim that M2–M9 functionality, runtime dependencies, model weights, CI, or installers exist.
