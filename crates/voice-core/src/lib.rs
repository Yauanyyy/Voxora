//! Portable domain foundation for Voxora.
//!
//! M2 establishes only the crate boundary and build metadata. Domain values and
//! state transitions begin in M3.

#[cfg(test)]
mod tests {
    #[test]
    fn package_metadata_is_explicit() {
        assert_eq!(env!("CARGO_PKG_NAME"), "voice-core");
        assert_eq!(env!("CARGO_PKG_LICENSE"), "GPL-3.0-only");
    }
}
