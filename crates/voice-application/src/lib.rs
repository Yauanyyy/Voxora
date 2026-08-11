//! Portable application-use-case boundary for Voxora.
//!
//! M2 establishes only the inward dependencies on `voice-core` and
//! `voice-ports`. Session-scoped coordination begins in M3.

use voice_core as _;
use voice_ports as _;

#[cfg(test)]
mod tests {
    #[test]
    fn package_metadata_is_explicit() {
        assert_eq!(env!("CARGO_PKG_NAME"), "voice-application");
        assert_eq!(env!("CARGO_PKG_LICENSE"), "GPL-3.0-only");
    }
}
