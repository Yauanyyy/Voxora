use std::fmt;
use std::num::NonZeroU64;
use std::str::FromStr;

macro_rules! opaque_id {
    ($name:ident, $code:literal) => {
        #[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(NonZeroU64);

        impl $name {
            #[must_use]
            pub const fn new(value: u64) -> Option<Self> {
                match NonZeroU64::new(value) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }

            #[must_use]
            pub const fn get(self) -> u64 {
                self.0.get()
            }

            #[must_use]
            pub const fn wire_prefix() -> &'static str {
                $code
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new(1).expect("one is non-zero")
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_struct(stringify!($name))
                    .field("value", &self.get())
                    .finish()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "{}-{}", $code, self.get())
            }
        }

        impl FromStr for $name {
            type Err = IdParseError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                let (prefix, number) = value.split_once('-').ok_or(IdParseError::Malformed)?;
                if prefix != $code {
                    return Err(IdParseError::WrongPrefix);
                }
                let number = number.parse::<u64>().map_err(|_| IdParseError::Malformed)?;
                Self::new(number).ok_or(IdParseError::Zero)
            }
        }
    };
}

/// Stable parse failures for opaque IDs.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IdParseError {
    Malformed,
    WrongPrefix,
    Zero,
}

opaque_id!(SessionId, "ses");
opaque_id!(DictationRecordId, "rec");
opaque_id!(RecognitionAttemptId, "att");
opaque_id!(ConfigurationId, "cfg");
opaque_id!(PromptPresetId, "prm");
opaque_id!(HotwordGroupId, "hgrp");
opaque_id!(HotwordId, "hwd");
opaque_id!(ApplicationProfileId, "prof");
opaque_id!(ProcessingRuleId, "rule");
opaque_id!(OperationId, "op");
opaque_id!(TargetId, "tgt");
opaque_id!(AudioReferenceId, "aud");
opaque_id!(CredentialReferenceId, "cred");
opaque_id!(ModelId, "mdl");
opaque_id!(RecoveryId, "rvr");

/// A safe helper for deterministic tests and in-memory adapters.
///
/// # Panics
///
/// Panics when `constructor` rejects the supplied zero value.
#[must_use]
pub fn id_from_u64<T>(value: u64, constructor: fn(u64) -> Option<T>) -> T {
    constructor(value).expect("test identifiers must be non-zero")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_have_stable_codes_and_checked_parsing() {
        let id = SessionId::new(7).unwrap();
        assert_eq!(id.to_string(), "ses-7");
        assert_eq!("ses-7".parse::<SessionId>().unwrap(), id);
        assert!("ses-0".parse::<SessionId>().is_err());
        assert!("rec-7".parse::<SessionId>().is_err());
    }

    #[test]
    fn debug_does_not_contain_sensitive_content_by_construction() {
        let id = AudioReferenceId::new(3).unwrap();
        assert_eq!(format!("{id:?}"), "AudioReferenceId { value: 3 }");
    }
}
