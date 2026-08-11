# Third-Party Notices

## M1 baseline

This documentation-stage repository currently distributes no third-party runtime component, native binary, image, font, sample media, provider SDK, inference framework, or model weight. The planned names in the product and architecture documents are design references, not present distribution contents. In particular, the planned sherpa-onnx integration and any SenseVoice artifact have not been approved or included.

The Voxora source and documentation are independently authored and use the project SPDX expression `GPL-3.0-only`; see [`LICENSE`](LICENSE) and [`docs/licensing.md`](docs/licensing.md). That expression does not change the license of a future third-party work.

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
