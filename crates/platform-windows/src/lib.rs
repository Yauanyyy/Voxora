#![deny(unsafe_code)]
#![allow(clippy::missing_errors_doc)]

use voice_core::{
    CredentialReferenceId, CredentialSecret, DeliveryCertainty, FailureCode, FailureStage,
    RetryMeaning, SanitizedFailure,
};
use voice_ports::{CredentialStorePort, PortResult};

const SERVICE: &str = "Voxora";

pub struct WindowsCredentialStore;

impl WindowsCredentialStore {
    pub fn new() -> Result<Self, SanitizedFailure> {
        #[cfg(windows)]
        {
            let store = windows_native_keyring_store::Store::new().map_err(|_| unavailable())?;
            keyring_core::set_default_store(store);
            Ok(Self)
        }
        #[cfg(not(windows))]
        {
            Err(unavailable())
        }
    }

    #[cfg(windows)]
    fn entry(reference: CredentialReferenceId) -> Result<keyring_core::Entry, SanitizedFailure> {
        keyring_core::Entry::new(SERVICE, &reference.to_string()).map_err(|_| unavailable())
    }
}

fn missing() -> SanitizedFailure {
    SanitizedFailure::from_boundary(
        FailureStage::Credential,
        FailureCode::CredentialMissing,
        RetryMeaning::Retryable,
        DeliveryCertainty::NotApplicable,
    )
}

fn unavailable() -> SanitizedFailure {
    SanitizedFailure::from_boundary(
        FailureStage::Credential,
        FailureCode::CredentialUnavailable,
        RetryMeaning::Retryable,
        DeliveryCertainty::NotApplicable,
    )
}

impl CredentialStorePort for WindowsCredentialStore {
    fn read(&mut self, reference: CredentialReferenceId) -> PortResult<CredentialSecret> {
        #[cfg(windows)]
        {
            let entry = Self::entry(reference)?;
            match entry.get_password() {
                Ok(value) => Ok(CredentialSecret::new(value)),
                Err(keyring_core::Error::NoEntry) => Err(missing()),
                Err(_) => Err(unavailable()),
            }
        }
        #[cfg(not(windows))]
        {
            let _ = reference;
            Err(unavailable())
        }
    }

    fn write(
        &mut self,
        reference: CredentialReferenceId,
        secret: CredentialSecret,
    ) -> PortResult<()> {
        #[cfg(windows)]
        {
            Self::entry(reference)?
                .set_password(secret.as_str())
                .map_err(|_| unavailable())
        }
        #[cfg(not(windows))]
        {
            let _ = (reference, secret);
            Err(unavailable())
        }
    }

    fn delete(&mut self, reference: CredentialReferenceId) -> PortResult<()> {
        #[cfg(windows)]
        {
            let entry = Self::entry(reference)?;
            match entry.delete_credential() {
                Ok(()) => Ok(()),
                Err(keyring_core::Error::NoEntry) => Err(missing()),
                Err(_) => Err(unavailable()),
            }
        }
        #[cfg(not(windows))]
        {
            let _ = reference;
            Err(unavailable())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_windows_constructor_is_fail_closed() {
        #[cfg(not(windows))]
        assert!(WindowsCredentialStore::new().is_err());
    }

    #[test]
    fn credential_failures_are_sanitized() {
        assert!(!format!("{:?}", unavailable()).contains("Voxora"));
        assert_eq!(missing().code(), FailureCode::CredentialMissing);
    }

    #[cfg(windows)]
    #[test]
    fn windows_credential_manager_round_trip_and_delete() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is after the Unix epoch")
            .as_nanos();
        let value = u64::try_from(nanos).unwrap_or(u64::MAX).max(1);
        let reference = CredentialReferenceId::new(value).expect("generated reference is nonzero");
        let mut store = WindowsCredentialStore::new().expect("Windows Credential Manager opens");
        let _ = store.delete(reference);
        store
            .write(
                reference,
                CredentialSecret::new("voxora-synthetic-m4-secret"),
            )
            .expect("synthetic credential writes");
        assert_eq!(
            store
                .read(reference)
                .expect("synthetic credential reads")
                .as_str(),
            "voxora-synthetic-m4-secret"
        );
        store
            .delete(reference)
            .expect("synthetic credential deletes");
        assert_eq!(
            store
                .read(reference)
                .expect_err("deleted credential is missing")
                .code(),
            FailureCode::CredentialMissing
        );
    }
}
