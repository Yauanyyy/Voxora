//! Portable domain values and deterministic state transitions for Voxora.
//!
//! This crate deliberately uses only the Rust standard library.  It contains no
//! provider, platform, UI, persistence, or runtime types; those capabilities are
//! represented by the contracts in `voice-ports`.

#![deny(unsafe_code)]

use std::fmt;
use std::num::NonZeroU64;

mod ids;
mod materials;
mod processing;
mod reducer;
mod values;

pub use ids::*;
pub use materials::*;
pub use processing::*;
pub use reducer::*;
pub use values::*;

/// A checked, non-zero revision used to correlate asynchronous work.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Revision(NonZeroU64);

impl Revision {
    /// The first revision in a new correlation.
    #[must_use]
    pub const fn first() -> Self {
        // SAFETY: one is non-zero and the value is constructed in a const context.
        Self(NonZeroU64::MIN)
    }

    /// Construct a revision, rejecting zero.
    #[must_use]
    pub const fn new(value: u64) -> Option<Self> {
        match NonZeroU64::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    /// Return the numeric wire representation.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0.get()
    }

    /// Advance the revision, returning `None` on overflow.
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self.0.get().checked_add(1) {
            Some(value) => Self::new(value),
            None => None,
        }
    }
}

impl Default for Revision {
    fn default() -> Self {
        Self::first()
    }
}

impl fmt::Debug for Revision {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("Revision")
            .field(&self.get())
            .finish()
    }
}

impl fmt::Display for Revision {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(formatter)
    }
}

/// A deterministic monotonic time value expressed in milliseconds.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Timestamp(u64);

impl Timestamp {
    #[must_use]
    pub const fn new(milliseconds: u64) -> Self {
        Self(milliseconds)
    }

    #[must_use]
    pub const fn milliseconds(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn checked_add(self, duration: DurationLimit) -> Option<Self> {
        match self.0.checked_add(duration.milliseconds()) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

/// A checked duration used by capture and recognition deadlines.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DurationLimit(u64);

impl DurationLimit {
    #[must_use]
    pub const fn new(milliseconds: u64) -> Option<Self> {
        if milliseconds == 0 {
            None
        } else {
            Some(Self(milliseconds))
        }
    }

    #[must_use]
    pub const fn from_seconds(seconds: u64) -> Option<Self> {
        match seconds.checked_mul(1_000) {
            Some(milliseconds) => Self::new(milliseconds),
            None => None,
        }
    }

    #[must_use]
    pub const fn milliseconds(self) -> u64 {
        self.0
    }
}

/// The user-configurable capture maximum.  M3 deliberately validates the
/// product boundary here instead of allowing an arbitrary duration to reach
/// the session reducer.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CaptureLimit(DurationLimit);

impl CaptureLimit {
    pub const MIN_SECONDS: u64 = 60;
    pub const MAX_SECONDS: u64 = 300;

    #[must_use]
    pub const fn from_seconds(seconds: u64) -> Option<Self> {
        if seconds < Self::MIN_SECONDS || seconds > Self::MAX_SECONDS {
            return None;
        }
        match DurationLimit::from_seconds(seconds) {
            Some(duration) => Some(Self(duration)),
            None => None,
        }
    }

    #[must_use]
    #[allow(clippy::manual_is_multiple_of)]
    pub const fn from_duration(duration: DurationLimit) -> Option<Self> {
        if duration.milliseconds() % 1_000 != 0 {
            return None;
        }
        Self::from_seconds(duration.milliseconds() / 1_000)
    }

    #[must_use]
    pub const fn duration(self) -> DurationLimit {
        self.0
    }

    #[must_use]
    pub const fn seconds(self) -> u64 {
        self.0.milliseconds() / 1_000
    }

    #[must_use]
    pub const fn checked_deadline(self, started_at: Timestamp) -> Option<Timestamp> {
        started_at.checked_add(self.0)
    }
}

/// A compact cancellation token identifier.  The token itself lives in the
/// cancellation port; the domain only carries its opaque correlation.
pub type CancellationTokenId = OperationId;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revision_and_duration_reject_zero_and_overflow() {
        assert!(Revision::new(0).is_none());
        assert_eq!(Revision::first().get(), 1);
        assert!(Revision::new(u64::MAX).expect("nonzero").next().is_none());
        assert!(DurationLimit::new(0).is_none());
        assert_eq!(
            Timestamp::new(5)
                .checked_add(DurationLimit::new(7).unwrap())
                .unwrap()
                .milliseconds(),
            12
        );
    }

    #[test]
    fn portable_wire_codes_reject_unknown_values() {
        assert_eq!(Phase::from_code("processing"), Some(Phase::Processing));
        assert!(Phase::from_code("not-a-phase").is_none());
        assert_eq!(
            TerminalOutcome::from_code("failed"),
            Some(TerminalOutcome::Failed)
        );
        assert!(TerminalOutcome::from_code("warning").is_none());
        assert_eq!(Warning::from_code("low_volume"), Some(Warning::LowVolume));
        assert!(FailureCode::from_code("provider-body").is_none());
        assert_eq!(Durability::from_code("durable"), Some(Durability::Durable));
        assert!(MaterialKind::from_code("audio-bytes").is_none());
    }
}
