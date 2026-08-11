# M2 workspace and CI dependency review

## Decision

Approved for the M2 build skeleton on 2026-08-12. This review covers the exact source/build dependencies locked by `Cargo.lock` and `apps/desktop/package-lock.json`, the Rust/Node toolchain pins used by CI, the two official GitHub Actions pinned by commit, and the project-authored application icon.

The approval is limited to the M2 Windows desktop shell and portable-crate CI. It does not approve a provider SDK, native adapter, model framework, model artifact, paid service, network destination, or later product feature. Any version, source, feature, target, license, advisory, distribution, or integrity change invalidates the affected part of this review.

## Direct Rust dependencies

| Component | Exact version | Feature/use | Authoritative source and selected license | Distribution and obligations | Decision |
| --- | --- | --- | --- | --- | --- |
| `tauri` | 2.11.5 | Windows desktop composition root with default desktop runtime; no plugins or commands | [tauri-apps/tauri](https://github.com/tauri-apps/tauri), Apache-2.0 OR MIT | Source is fetched from crates.io; a built executable statically incorporates applicable Rust dependencies. Preserve license/notice obligations and Corresponding Source for the GPL-covered application distribution. | Approved for the M2 Windows shell only. |
| `tauri-build` | 2.6.3 | Build-script resource/config generation | [tauri-apps/tauri](https://github.com/tauri-apps/tauri), Apache-2.0 OR MIT | Build-time source fetched from crates.io; no separately committed binary. | Approved for M2. |

`voice-core`, `voice-ports`, `voice-application`, and `voxora-desktop` are independently authored workspace packages under `GPL-3.0-only`. Workspace path dependencies use exact `=0.1.0` versions and follow the accepted inward dependency direction.

## Direct npm dependencies and tools

| Component | Exact version | Use | Authoritative source | License | Decision |
| --- | --- | --- | --- | --- | --- |
| `react`, `react-dom` | 19.2.8 | Static M2 shell rendering | [facebook/react](https://github.com/facebook/react) | MIT | Approved. |
| `@tauri-apps/cli` | 2.11.4 | Local/CI Tauri build command | [tauri-apps/tauri](https://github.com/tauri-apps/tauri) | Apache-2.0 OR MIT | Approved as a build tool. |
| `vite` | 8.2.1 | Frontend build/dev server | [vitejs/vite](https://github.com/vitejs/vite) | MIT | Approved as a build tool. |
| `@vitejs/plugin-react` | 6.0.5 | React transform | [vitejs/vite-plugin-react](https://github.com/vitejs/vite-plugin-react) | MIT | Approved as a build tool. |
| `typescript` | 6.0.3 | Type checking | [microsoft/TypeScript](https://github.com/microsoft/TypeScript) | Apache-2.0 | Approved as a build tool. |
| `vitest` | 4.1.10 | Frontend unit-test baseline | [vitest-dev/vitest](https://github.com/vitest-dev/vitest) | MIT | Approved as a test tool. |
| `eslint`, `@eslint/js` | 10.8.1, 10.0.1 | Frontend lint | [eslint/eslint](https://github.com/eslint/eslint) | MIT | Approved as lint tools. |
| `typescript-eslint` | 8.67.0 | TypeScript lint integration | [typescript-eslint/typescript-eslint](https://github.com/typescript-eslint/typescript-eslint) | MIT | Approved as a lint tool. |
| `globals` | 17.9.0 | ESLint environment definitions | [sindresorhus/globals](https://github.com/sindresorhus/globals) | MIT | Approved as a lint-data package. |
| `prettier` | 3.9.6 | Deterministic frontend/script/workflow formatting | [prettier/prettier](https://github.com/prettier/prettier) | MIT | Approved as a formatting tool. |
| `@types/node` | 24.13.3 | Node type declarations | [DefinitelyTyped](https://github.com/DefinitelyTyped/DefinitelyTyped) | MIT | Approved as type declarations. |
| `@types/react`, `@types/react-dom` | 19.2.18, 19.2.4 | React type declarations | [DefinitelyTyped](https://github.com/DefinitelyTyped/DefinitelyTyped) | MIT | Approved as type declarations. |

All npm packages are fetched from `https://registry.npmjs.org/`. The npm 11 lockfile records an exact version, registry URL, SHA-512 integrity value, and declared license for every resolved package entry. After `npm ci --ignore-scripts`, policy validation also reads each installed package's own `package.json`: its derived package identity, exact version, and license must match the lock and reviewed allowlist, and a missing non-optional installation fails closed. Platform-filtered optional entries may be absent, so the Windows desktop job repeats this check for the Windows-specific packages it installs.

## CI and toolchain inputs

| Input | Exact pin | Source/license | Decision |
| --- | --- | --- | --- |
| Rust toolchain | 1.97.1, distribution dated 2026-07-16 | [Rust distributions](https://static.rust-lang.org/dist/channel-rust-stable.toml); Rust toolchain terms apply | Approved as the compiler/formatter/Clippy toolchain; not bundled. |
| Node.js | 24.15.0 | [nodejs/node](https://github.com/nodejs/node), project license inventory applies | Approved as the CI/development runtime; not bundled. |
| npm | lockfile v3, npm 11 compatible | [npm/cli](https://github.com/npm/cli), Artistic-2.0 | Approved as the package manager; not bundled. |
| `cargo-deny` | 0.20.2 | [EmbarkStudios/cargo-deny](https://github.com/EmbarkStudios/cargo-deny), Apache-2.0 OR MIT | Approved as a CI/development policy tool; installed from crates.io, not bundled. |
| `actions/checkout` | `d23441a48e516b6c34aea4fa41551a30e30af803` (v6) | [actions/checkout](https://github.com/actions/checkout), MIT | Approved and commit-pinned. |
| `actions/setup-node` | `249970729cb0ef3589644e2896645e5dc5ba9c38` (v6) | [actions/setup-node](https://github.com/actions/setup-node), MIT | Approved and commit-pinned. |

## Resolved inventory evidence

### Cargo Windows graph

`cargo-deny 0.20.2 list --format json` reported 238 unique package identifiers for `x86_64-pc-windows-msvc`. License-file matches are not mutually exclusive because a crate may offer multiple branches:

| License identifier | Matched crate identifiers |
| --- | ---: |
| 0BSD | 1 |
| Apache-2.0 | 159 |
| BSD-3-Clause | 4 |
| CC0-1.0 | 1 |
| GPL-3.0-only | 4 workspace packages |
| MIT | 206 |
| MIT-0 | 1 |
| MPL-2.0 | 5 |
| Unicode-3.0 | 19 |
| Unlicense | 6 |
| Zlib | 3 |

The accepted license branches in `deny.toml` are limited to those actually needed to satisfy the Windows graph. Unknown registries and Git sources fail, wildcard dependency declarations fail, yanked packages fail, vulnerabilities fail, and unsound advisories fail for the entire selected graph. Duplicate versions are allowed because they are resolved transitive choices in the exact lockfile rather than an approval of arbitrary versions.

Unmaintained advisories fail for workspace packages. Third-party unmaintained-only advisories are reported as maintenance evidence but do not fail M2 when no safe upstream replacement exists. The Windows graph currently reaches `unic-char-property`, `unic-char-range`, `unic-common`, `unic-ucd-ident`, and `unic-ucd-version` through `tauri-utils` → `urlpattern` (RUSTSEC-2025-0081, RUSTSEC-2025-0075, RUSTSEC-2025-0080, RUSTSEC-2025-0100, and RUSTSEC-2025-0098). These advisories describe unmaintained projects, not a published vulnerability or unsoundness. The crates are not called directly by Voxora, and a future Tauri update must re-evaluate or remove them. Any vulnerability, unsound advisory, source change, or new license still fails closed.

The Cargo policy intentionally selects the Windows desktop target. M2 compiles the dependency-free portable workspace crates on Windows, macOS, and Linux, but it builds and distributes no macOS/Linux desktop application. Adding a non-Windows desktop target requires a new dependency and advisory review before CI or distribution is enabled.

### npm lock graph

`apps/desktop/package-lock.json` contains 175 resolved third-party package entries. The fail-closed validator reported:

| Declared license expression | Package entries |
| --- | ---: |
| Apache-2.0 OR MIT | 12 |
| Apache-2.0 | 15 |
| BSD-2-Clause | 6 |
| BSD-3-Clause | 2 |
| BlueOak-1.0.0 | 1 |
| ISC | 7 |
| MIT | 120 |
| MPL-2.0 | 12 |

The validator rejects a missing or unreviewed lock license, non-registry source, missing exact version, missing SHA-512 integrity value, missing required installation, installed identity/version mismatch, or installed license that differs from or is denied despite the lock metadata. Synthetic negative tests prove these fail-closed paths without real credentials or private data. `npm audit` reported zero known vulnerabilities when the lockfile was created; future lock changes must be re-audited and re-reviewed.

## Native components and assets

Tauri's Windows build uses the Microsoft WebView2 interfaces represented by the reviewed crates in the Cargo graph. M2 commits no WebView2 installer, native DLL, SDK archive, or other third-party binary; the runtime is supplied by the Windows environment. Packaging and redistribution review remains an M9 obligation.

`apps/desktop/app-icon.svg` is an original Voxora M2 placeholder made from simple project-authored vector shapes. `apps/desktop/src-tauri/icons/icon.ico` was deterministically generated from that SVG by the locked Tauri CLI. It contains no third-party artwork, font, trademark, or copied logo and is licensed with the Voxora project under `GPL-3.0-only`.

No font, sample media, provider SDK, inference framework, model manifest, model weight, tokenizer, vocabulary, or model-related binary is included or approved.

## Security, maintenance, and invalidation

- Product code makes no network request and contains no credential path.
- CI retrieves only pinned toolchains/actions and exact public-registry dependencies; it makes no paid provider call and downloads no model.
- Tracked/untracked source scanning uses credential-shaped patterns and synthetic tests; it does not print secret values.
- The committed executable output and `node_modules` are ignored and not distributed by the source branch.
- Re-review is mandatory for any direct or transitive version, source URL, integrity, license, Tauri feature, target platform, action commit, native component, or asset-source change.
- The reviewer for this record is the primary Codex agent executing the user-authorized M2 plan on 2026-08-12. The user retains merge ownership.
