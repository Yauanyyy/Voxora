# Third-Party Notices

## M2 workspace and CI baseline

M2 adds exact public-registry source/build dependencies for a buildable Windows desktop shell and its CI. The authoritative review, versions, sources, selected license branches, inventory counts, advisory posture, and invalidation conditions are in [`docs/dependency-reviews/m2-workspace-ci.md`](docs/dependency-reviews/m2-workspace-ci.md). Exact resolved bytes are represented by [`Cargo.lock`](Cargo.lock) and [`apps/desktop/package-lock.json`](apps/desktop/package-lock.json).

Direct application/build dependencies are Tauri 2.11.5 and tauri-build 2.6.3 under Apache-2.0 OR MIT; React and React DOM 19.2.8 under MIT; and the reviewed TypeScript, Vite, Vitest, ESLint, Prettier, Tauri CLI, type-declaration, and CI toolchain packages listed in the review. The selected Windows Cargo graph uses compatible branches from 0BSD, Apache-2.0, BSD-3-Clause, CC0-1.0, MIT, MIT-0, MPL-2.0, Unicode-3.0, Unlicense, and Zlib. The npm lock graph uses Apache-2.0, Apache-2.0 OR MIT, BSD-2-Clause, BSD-3-Clause, BlueOak-1.0.0, ISC, MIT, and MPL-2.0 declarations. License and source checks fail on unknown or denied data.

M2 commits no third-party native DLL, WebView2 installer, font, sample media, provider SDK, inference framework, model manifest, model weight, tokenizer, vocabulary, or model artifact. The Windows environment supplies the WebView2 runtime. The placeholder SVG/ICO application icon is independently project-authored and is not third-party content. Planned sherpa-onnx and SenseVoice names remain design references only and are not approved or included.

The Voxora source, documentation, and project-authored icon use project SPDX expression `GPL-3.0-only`; see [`LICENSE`](LICENSE) and [`docs/licensing.md`](docs/licensing.md). That expression does not alter the licenses of third-party works. A future binary distribution must preserve every applicable third-party notice and satisfy the GPL Corresponding Source obligations; M9 owns release-package auditing.

## Future record format

Every distributed or downloaded component that requires a notice must receive a reviewed record before it is used or shipped. Record at least:

| Field | Required information |
| --- | --- |
| Identity and version | Package, native component, asset, protocol/SDK, framework, or exact model artifact and revision. |
| Source and provenance | Authoritative source URL or archive, retrieval evidence, and how the exact bytes were obtained. |
| License | SPDX identifier where applicable, exact license text or governing terms, and compatibility rationale. |
| Use and distribution | Feature use, whether bundled or user-downloaded, binary/source redistribution duties, and any Corresponding Source or source-offer obligation. |
| Model details | Every file, format, size, SHA-256, commercial-use and redistribution rights, conversion/derivative terms, and accompanying-file licenses. |
| Notices and security | Required attributions/notices, advisory evidence, and known restrictions. |
| Review | Reviewer, review date, status, and invalidation conditions. |

The complete checklists and rejection policy live in [`docs/licensing.md`](docs/licensing.md). A scanner or package manifest is evidence only; it never replaces source, license-text, distribution, notice, or model review.

## Maintenance rule

Update this file in the same change that adds, removes, upgrades, converts, bundles, or enables a dependency, native component, asset, provider SDK, inference framework, or model artifact. Do not list planned components as currently distributed, and do not mark an item approved until the fail-closed review is complete.
