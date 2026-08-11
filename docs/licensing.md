# Licensing and Dependency Review Policy

## Project expression and independent implementation

Voxora's project SPDX expression is `GPL-3.0-only`, which means GNU GPL version 3 only; it is not `GPL-3.0-or-later`. [`LICENSE`](../LICENSE) is the unmodified canonical GNU GPL version 3 text. Product concepts may inform requirements, but implementation must be independently authored: do not copy, rewrite, port, translate, or derive source from SayIt or another GPL/AGPL project.

M1 distributes no runtime dependency, native component, asset, provider SDK, inference framework, or model. [`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md) must remain truthful as future milestones add reviewed items.

## Fail-closed review

No artifact may be added, enabled, bundled, or described as approved until its review record is complete. This applies separately to:

- Rust and JavaScript/TypeScript source dependencies;
- native libraries, platform SDKs, and bundled binaries;
- images, fonts, sample media, icons, and other assets;
- provider SDKs, protocol references, and generated client material;
- inference frameworks and their native/runtime components;
- model weights and every accompanying tokenizer, vocabulary, configuration, preprocessing, conversion, or metadata file.

Reject by project policy any AGPL, SSPL, non-commercial, research-only, field-of-use-restricted, source-unclear, or otherwise unacceptable artifact unless the user explicitly changes the policy after a documented review. A policy rejection is not necessarily a statement that the artifact is legally incompatible with GPL; record the specific policy reason.

## Dependency/native/asset checklist

For every dependency, native component, SDK/protocol reference, or asset, record all of the following before acceptance:

1. identity, exact version/revision, and feature(s) used;
2. authoritative project/source URL, retrieval date, and provenance evidence (including hashes where relevant);
3. selected license branch, exact governing license text or terms, SPDX identifier if applicable, and compatibility rationale for `GPL-3.0-only`;
4. whether the item is source, binary, generated material, bundled, downloaded by the user, or only referenced;
5. binary/source redistribution duties, attribution and notice duties, and any source-offer or Corresponding Source implications;
6. security/advisory and maintenance evidence, known restrictions, and invalidation conditions;
7. reviewer, review date, decision, and required `THIRD_PARTY_NOTICES.md` entry.

An automated scanner or package manifest is evidence, not a substitute for inspecting authoritative source, exact license text, distribution conditions, notices, and provenance.

## Model artifact checklist

Model approval is independent from inference-framework approval. For every exact artifact and revision, record:

1. publisher, official source, retrieval method/date, and provenance evidence;
2. every distributed or downloaded file, including tokenizer, vocabulary, configuration, preprocessing, conversion, and metadata files;
3. format, exact size, SHA-256, and installation/activation path;
4. exact license text/terms, commercial-use rights, redistribution rights, and conversion/derivative terms;
5. the license of every accompanying file and whether converted ONNX weights remain governed by source-weight terms;
6. distribution path (bundled, user-initiated download, or user import), notices, source/facilitation obligations, and deletion/update rules;
7. review status/date, reviewer, security/provenance evidence, and invalidation conditions such as hash, version, license, or source changes.

User-initiated download does not automatically remove project distribution or facilitation obligations. Model weights never enter the application package merely because a framework can load them. No automatic model update or background check is allowed by default.

## sherpa-onnx and SenseVoice gate

sherpa-onnx is only the planned local inference framework; its framework license and integration terms do not approve a SenseVoice model. M8 is blocked until one exact SenseVoice Small artifact passes the complete model checklist with source, exact version/revision, size, SHA-256, license, commercial-use rights, redistribution terms, every accompanying-file license, and sherpa-onnx integration review. Selecting sherpa-onnx therefore never represents that any model artifact is accepted.

## Notice and change control

Any dependency, native component, asset, provider protocol/SDK, framework, model, conversion, or version change requires a fresh review and a synchronized notice update. Removing an item also requires checking that no source, binary, generated file, or documentation still claims it is distributed. Review records must be retained with the task evidence; unknown or incomplete results fail closed.
