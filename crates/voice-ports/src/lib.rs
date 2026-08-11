//! Portable capability boundaries for Voxora.
//!
//! M2 establishes only the inward dependency on `voice-core`. Capability
//! contracts begin in M3.

use voice_core as _;

#[cfg(test)]
mod tests {
    #[test]
    fn package_metadata_is_explicit() {
        assert_eq!(env!("CARGO_PKG_NAME"), "voice-ports");
        assert_eq!(env!("CARGO_PKG_LICENSE"), "GPL-3.0-only");
    }
}
