# M4 local persistence and configuration dependency review

Review date: 2026-08-12. Status: approved for the M4 graph.

`rusqlite 0.40.2` is used with `default-features = false` and only the
`backup` and `bundled` features. The backup API produces a consistent SQLite
snapshot, and the bundled SQLite amalgamation is used for reproducible
desktop builds; no load-extension, SQLCipher, OpenSSL, or network feature is
enabled. Its direct license is MIT, and the resolved `libsqlite3-sys 0.38.2`
and bundled SQLite sources are represented by the lockfile. The adapter stores
metadata/text only; audio remains separate files.

Portable Base URL validation uses the already locked `url 2.5.8` parser with
default features disabled. This activates no network behavior; it supplies
standards-compliant absolute URL, host, userinfo, port, query, and fragment
parsing before persistence. Its resolved public-registry transitive graph and
allowed licenses are represented in `Cargo.lock` and checked by `cargo deny`.

The Windows-only credential adapter uses `keyring-core 1.0.0` and
`windows-native-keyring-store 1.1.0`, both MIT OR Apache-2.0. The dependency
edges are target-scoped to `cfg(windows)`; Linux and macOS builds do not pull
the Windows store. The selected store is Windows Credential Manager. Voxora
passes only opaque `CredentialReferenceId`-derived service/user identifiers to
the store and maps backend details to sanitized credential codes.

Resolved transitive dependencies include `fallible-iterator 0.3.0`,
`fallible-streaming-iterator 0.1.9`, `smallvec 1.15.2`, `bitflags 2.13.1`,
`byteorder 1.5.0`, `windows-sys 0.61.2`, `zeroize 1.9.0`, and `log 0.4.33`.
The bundled SQLite build path also resolves `cc 1.4.2`,
`find-msvc-tools 0.1.10`, `shlex 2.0.1`, `pkg-config 0.3.33`, and
`vcpkg 0.2.15`; the selected bundled build uses `cc`, while the other build
helpers remain target/build-path alternatives in the resolved graph. They are
public crates under licenses allowed by `deny.toml`; exact versions, checksums,
and source registry are in `Cargo.lock`.

No provider SDK, model, network client, native DLL, telemetry, or project
server is introduced. Re-review is required if any selected feature, version,
source, or target scope changes.
