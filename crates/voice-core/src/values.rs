use std::fmt;
use std::str::FromStr;

use crate::{
    CaptureLimit, ConfigurationId, DurationLimit, MaterialKind, OperationId, Revision, SessionId,
    Timestamp,
};

/// The exact lifecycle phases defined by the product state machine.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Phase {
    Idle,
    Capturing,
    StoppingCapture,
    Recognizing,
    Processing,
    Delivering,
    Completed,
    Recovery,
}

impl Phase {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Capturing => "capturing",
            Self::StoppingCapture => "stopping_capture",
            Self::Recognizing => "recognizing",
            Self::Processing => "processing",
            Self::Delivering => "delivering",
            Self::Completed => "completed",
            Self::Recovery => "recovery",
        }
    }

    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "idle" => Some(Self::Idle),
            "capturing" => Some(Self::Capturing),
            "stopping_capture" => Some(Self::StoppingCapture),
            "recognizing" => Some(Self::Recognizing),
            "processing" => Some(Self::Processing),
            "delivering" => Some(Self::Delivering),
            "completed" => Some(Self::Completed),
            "recovery" => Some(Self::Recovery),
            _ => None,
        }
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl FromStr for Phase {
    type Err = CodeParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::from_code(value).ok_or(CodeParseError)
    }
}

/// The five and only five terminal outcomes for a Dictation Session.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TerminalOutcome {
    DeliveredAutomatically,
    ManualDeliveryRequired,
    DeliveryUncertain,
    Cancelled,
    Failed,
}

impl TerminalOutcome {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::DeliveredAutomatically => "delivered_automatically",
            Self::ManualDeliveryRequired => "manual_delivery_required",
            Self::DeliveryUncertain => "delivery_uncertain",
            Self::Cancelled => "cancelled",
            Self::Failed => "failed",
        }
    }

    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "delivered_automatically" => Some(Self::DeliveredAutomatically),
            "manual_delivery_required" => Some(Self::ManualDeliveryRequired),
            "delivery_uncertain" => Some(Self::DeliveryUncertain),
            "cancelled" => Some(Self::Cancelled),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

impl fmt::Display for TerminalOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl FromStr for TerminalOutcome {
    type Err = CodeParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::from_code(value).ok_or(CodeParseError)
    }
}

/// Start gestures own their matching stop gesture.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StartMode {
    PushToTalk,
    Toggle,
}

impl StartMode {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::PushToTalk => "push_to_talk",
            Self::Toggle => "toggle",
        }
    }

    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "push_to_talk" => Some(Self::PushToTalk),
            "toggle" => Some(Self::Toggle),
            _ => None,
        }
    }
}

/// Sanitized stage values accepted into failure metadata.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FailureStage {
    Shortcut,
    Credential,
    ModelManagement,
    Capture,
    Recognition,
    Processing,
    Targeting,
    Delivery,
    Persistence,
    Recovery,
    Retry,
}

impl FailureStage {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Shortcut => "shortcut",
            Self::Credential => "credential",
            Self::ModelManagement => "model_management",
            Self::Capture => "capture",
            Self::Recognition => "recognition",
            Self::Processing => "processing",
            Self::Targeting => "targeting",
            Self::Delivery => "delivery",
            Self::Persistence => "persistence",
            Self::Recovery => "recovery",
            Self::Retry => "retry",
        }
    }

    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "shortcut" => Some(Self::Shortcut),
            "credential" => Some(Self::Credential),
            "model_management" => Some(Self::ModelManagement),
            "capture" => Some(Self::Capture),
            "recognition" => Some(Self::Recognition),
            "processing" => Some(Self::Processing),
            "targeting" => Some(Self::Targeting),
            "delivery" => Some(Self::Delivery),
            "persistence" => Some(Self::Persistence),
            "recovery" => Some(Self::Recovery),
            "retry" => Some(Self::Retry),
            _ => None,
        }
    }
}

/// Sanitized, project-owned failure codes.  No provider response can enter this type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FailureCode {
    ShortcutRegistration,
    CredentialMissing,
    CredentialUnavailable,
    ModelMissing,
    ModelInvalid,
    ModelManagement,
    EmptyAudio,
    DeviceFailure,
    CaptureCleanupFailed,
    RecognitionEmpty,
    RecognitionProvider,
    RecognitionTimeout,
    RecognitionCancelled,
    ProcessingStep,
    ProcessingTimeout,
    TargetUnavailable,
    TargetInvalid,
    InjectionFailed,
    ManualPreservationFailed,
    InsertionUncertain,
    PersistenceUnavailable,
    RecoveryUnavailable,
    RetryIneligible,
    RetryProvider,
    RetryTimeout,
    RetryCancelled,
    RetryEmpty,
}

impl FailureCode {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::ShortcutRegistration => "shortcut_registration",
            Self::CredentialMissing => "credential_missing",
            Self::CredentialUnavailable => "credential_unavailable",
            Self::ModelMissing => "model_missing",
            Self::ModelInvalid => "model_invalid",
            Self::ModelManagement => "model_management",
            Self::EmptyAudio => "empty_audio",
            Self::DeviceFailure => "device_failure",
            Self::CaptureCleanupFailed => "capture_cleanup_failed",
            Self::RecognitionEmpty => "recognition_empty",
            Self::RecognitionProvider => "recognition_provider",
            Self::RecognitionTimeout => "recognition_timeout",
            Self::RecognitionCancelled => "recognition_cancelled",
            Self::ProcessingStep => "processing_step",
            Self::ProcessingTimeout => "processing_timeout",
            Self::TargetUnavailable => "target_unavailable",
            Self::TargetInvalid => "target_invalid",
            Self::InjectionFailed => "injection_failed",
            Self::ManualPreservationFailed => "manual_preservation_failed",
            Self::InsertionUncertain => "insertion_uncertain",
            Self::PersistenceUnavailable => "persistence_unavailable",
            Self::RecoveryUnavailable => "recovery_unavailable",
            Self::RetryIneligible => "retry_ineligible",
            Self::RetryProvider => "retry_provider",
            Self::RetryTimeout => "retry_timeout",
            Self::RetryCancelled => "retry_cancelled",
            Self::RetryEmpty => "retry_empty",
        }
    }

    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "shortcut_registration" => Some(Self::ShortcutRegistration),
            "credential_missing" => Some(Self::CredentialMissing),
            "credential_unavailable" => Some(Self::CredentialUnavailable),
            "model_missing" => Some(Self::ModelMissing),
            "model_invalid" => Some(Self::ModelInvalid),
            "model_management" => Some(Self::ModelManagement),
            "empty_audio" => Some(Self::EmptyAudio),
            "device_failure" => Some(Self::DeviceFailure),
            "capture_cleanup_failed" => Some(Self::CaptureCleanupFailed),
            "recognition_empty" => Some(Self::RecognitionEmpty),
            "recognition_provider" => Some(Self::RecognitionProvider),
            "recognition_timeout" => Some(Self::RecognitionTimeout),
            "recognition_cancelled" => Some(Self::RecognitionCancelled),
            "processing_step" => Some(Self::ProcessingStep),
            "processing_timeout" => Some(Self::ProcessingTimeout),
            "target_unavailable" => Some(Self::TargetUnavailable),
            "target_invalid" => Some(Self::TargetInvalid),
            "injection_failed" => Some(Self::InjectionFailed),
            "manual_preservation_failed" => Some(Self::ManualPreservationFailed),
            "insertion_uncertain" => Some(Self::InsertionUncertain),
            "persistence_unavailable" => Some(Self::PersistenceUnavailable),
            "recovery_unavailable" => Some(Self::RecoveryUnavailable),
            "retry_ineligible" => Some(Self::RetryIneligible),
            "retry_provider" => Some(Self::RetryProvider),
            "retry_timeout" => Some(Self::RetryTimeout),
            "retry_cancelled" => Some(Self::RetryCancelled),
            "retry_empty" => Some(Self::RetryEmpty),
            _ => None,
        }
    }

    /// Map an adapter-provided code into the only codes valid for a boundary.
    ///
    /// Provider and processor adapters are allowed to report a project-owned
    /// code, but the receiving boundary still owns the stage.  Invalid pairs
    /// therefore become the stable generic code for that stage instead of
    /// entering persisted failure metadata.
    #[must_use]
    pub const fn for_stage(stage: FailureStage, code: Self) -> Self {
        match stage {
            FailureStage::Shortcut => Self::ShortcutRegistration,
            FailureStage::Credential => match code {
                Self::CredentialMissing | Self::CredentialUnavailable => code,
                _ => Self::CredentialUnavailable,
            },
            FailureStage::ModelManagement => match code {
                Self::ModelMissing | Self::ModelInvalid | Self::ModelManagement => code,
                _ => Self::ModelManagement,
            },
            FailureStage::Capture => match code {
                Self::EmptyAudio | Self::DeviceFailure | Self::CaptureCleanupFailed => code,
                _ => Self::DeviceFailure,
            },
            FailureStage::Recognition => match code {
                Self::RecognitionEmpty
                | Self::RecognitionProvider
                | Self::RecognitionTimeout
                | Self::RecognitionCancelled => code,
                _ => Self::RecognitionProvider,
            },
            FailureStage::Processing => match code {
                Self::ProcessingStep | Self::ProcessingTimeout => code,
                _ => Self::ProcessingStep,
            },
            FailureStage::Targeting => match code {
                Self::TargetUnavailable | Self::TargetInvalid => code,
                _ => Self::TargetUnavailable,
            },
            FailureStage::Delivery => match code {
                Self::InjectionFailed
                | Self::ManualPreservationFailed
                | Self::InsertionUncertain => code,
                _ => Self::InjectionFailed,
            },
            FailureStage::Persistence => Self::PersistenceUnavailable,
            FailureStage::Recovery => Self::RecoveryUnavailable,
            FailureStage::Retry => match code {
                Self::RetryIneligible
                | Self::RetryProvider
                | Self::RetryTimeout
                | Self::RetryCancelled
                | Self::RetryEmpty => code,
                _ => Self::RetryProvider,
            },
        }
    }
}

/// Whether a failure may be retried by an explicit user command.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RetryMeaning {
    Retryable,
    NotRetryable,
    NoAutomaticRetry,
}

impl RetryMeaning {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Retryable => "retryable",
            Self::NotRetryable => "not_retryable",
            Self::NoAutomaticRetry => "no_automatic_retry",
        }
    }

    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "retryable" => Some(Self::Retryable),
            "not_retryable" => Some(Self::NotRetryable),
            "no_automatic_retry" => Some(Self::NoAutomaticRetry),
            _ => None,
        }
    }
}

/// Delivery certainty is orthogonal to the terminal outcome and warning set.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DeliveryCertainty {
    NotApplicable,
    Confirmed,
    DefiniteFailure,
    Uncertain,
}

impl DeliveryCertainty {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NotApplicable => "not_applicable",
            Self::Confirmed => "confirmed",
            Self::DefiniteFailure => "definite_failure",
            Self::Uncertain => "uncertain",
        }
    }

    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "not_applicable" => Some(Self::NotApplicable),
            "confirmed" => Some(Self::Confirmed),
            "definite_failure" => Some(Self::DefiniteFailure),
            "uncertain" => Some(Self::Uncertain),
            _ => None,
        }
    }
}

/// A bounded failure object safe to persist or display in history.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SanitizedFailure {
    stage: FailureStage,
    code: FailureCode,
    retry: RetryMeaning,
    certainty: DeliveryCertainty,
}

impl SanitizedFailure {
    /// Construct failure metadata only when the stage/code pair is valid.
    #[must_use]
    pub const fn new(
        stage: FailureStage,
        code: FailureCode,
        retry: RetryMeaning,
        certainty: DeliveryCertainty,
    ) -> Option<Self> {
        Self::try_new(stage, code, retry, certainty)
    }

    /// Validate a stage/code pairing at a boundary where failures originate.
    #[must_use]
    pub const fn try_new(
        stage: FailureStage,
        code: FailureCode,
        retry: RetryMeaning,
        certainty: DeliveryCertainty,
    ) -> Option<Self> {
        let valid = match stage {
            FailureStage::Shortcut => matches!(code, FailureCode::ShortcutRegistration),
            FailureStage::Credential => {
                matches!(
                    code,
                    FailureCode::CredentialMissing | FailureCode::CredentialUnavailable
                )
            }
            FailureStage::ModelManagement => matches!(
                code,
                FailureCode::ModelMissing
                    | FailureCode::ModelInvalid
                    | FailureCode::ModelManagement
            ),
            FailureStage::Capture => matches!(
                code,
                FailureCode::EmptyAudio
                    | FailureCode::DeviceFailure
                    | FailureCode::CaptureCleanupFailed
            ),
            FailureStage::Recognition => matches!(
                code,
                FailureCode::RecognitionEmpty
                    | FailureCode::RecognitionProvider
                    | FailureCode::RecognitionTimeout
                    | FailureCode::RecognitionCancelled
            ),
            FailureStage::Processing => {
                matches!(
                    code,
                    FailureCode::ProcessingStep | FailureCode::ProcessingTimeout
                )
            }
            FailureStage::Targeting => {
                matches!(
                    code,
                    FailureCode::TargetUnavailable | FailureCode::TargetInvalid
                )
            }
            FailureStage::Delivery => matches!(
                code,
                FailureCode::InjectionFailed
                    | FailureCode::ManualPreservationFailed
                    | FailureCode::InsertionUncertain
            ),
            FailureStage::Persistence => matches!(code, FailureCode::PersistenceUnavailable),
            FailureStage::Recovery => matches!(code, FailureCode::RecoveryUnavailable),
            FailureStage::Retry => matches!(
                code,
                FailureCode::RetryIneligible
                    | FailureCode::RetryProvider
                    | FailureCode::RetryTimeout
                    | FailureCode::RetryCancelled
                    | FailureCode::RetryEmpty
            ),
        };
        if valid {
            Some(Self {
                stage,
                code,
                retry,
                certainty,
            })
        } else {
            None
        }
    }

    /// Construct a boundary failure after mapping an adapter/provider code to
    /// the stage that owns it.  This is the only public infallible constructor.
    #[must_use]
    pub const fn from_boundary(
        stage: FailureStage,
        code: FailureCode,
        retry: RetryMeaning,
        certainty: DeliveryCertainty,
    ) -> Self {
        let code = FailureCode::for_stage(stage, code);
        Self {
            stage,
            code,
            retry,
            certainty,
        }
    }

    #[must_use]
    pub const fn stage(self) -> FailureStage {
        self.stage
    }

    #[must_use]
    pub const fn code(self) -> FailureCode {
        self.code
    }

    #[must_use]
    pub const fn retry(self) -> RetryMeaning {
        self.retry
    }

    #[must_use]
    pub const fn certainty(self) -> DeliveryCertainty {
        self.certainty
    }
}

#[cfg(test)]
mod failure_tests {
    use super::*;

    #[test]
    fn sanitized_failure_accepts_only_valid_stage_code_pairs() {
        let valid = SanitizedFailure::new(
            FailureStage::Recognition,
            FailureCode::RecognitionProvider,
            RetryMeaning::Retryable,
            DeliveryCertainty::NotApplicable,
        );
        assert!(valid.is_some());

        let invalid = SanitizedFailure::new(
            FailureStage::Recognition,
            FailureCode::InjectionFailed,
            RetryMeaning::Retryable,
            DeliveryCertainty::NotApplicable,
        );
        assert!(invalid.is_none());
    }

    #[test]
    fn boundary_mapping_reclassifies_invalid_provider_codes() {
        let failure = SanitizedFailure::from_boundary(
            FailureStage::Processing,
            FailureCode::RecognitionProvider,
            RetryMeaning::NotRetryable,
            DeliveryCertainty::NotApplicable,
        );
        assert_eq!(failure.stage(), FailureStage::Processing);
        assert_eq!(failure.code(), FailureCode::ProcessingStep);
    }
}

/// Warnings are deliberately separate from outcomes and failures.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Warning {
    MaximumDurationReached,
    ProcessingFallback,
    PersistenceUnsaved,
    IncompletePartialRetained,
    TargetChanged,
    LowVolume,
}

impl Warning {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::MaximumDurationReached => "maximum_duration_reached",
            Self::ProcessingFallback => "processing_fallback",
            Self::PersistenceUnsaved => "persistence_unsaved",
            Self::IncompletePartialRetained => "incomplete_partial_retained",
            Self::TargetChanged => "target_changed",
            Self::LowVolume => "low_volume",
        }
    }

    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "maximum_duration_reached" => Some(Self::MaximumDurationReached),
            "processing_fallback" => Some(Self::ProcessingFallback),
            "persistence_unsaved" => Some(Self::PersistenceUnsaved),
            "incomplete_partial_retained" => Some(Self::IncompletePartialRetained),
            "target_changed" => Some(Self::TargetChanged),
            "low_volume" => Some(Self::LowVolume),
            _ => None,
        }
    }
}

/// Stable parse error for enum wire codes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CodeParseError;

/// Correlation shared by every live-session event.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LiveCorrelation {
    session_id: SessionId,
    session_revision: Revision,
    expected_phase: Phase,
}

impl LiveCorrelation {
    #[must_use]
    pub const fn new(
        session_id: SessionId,
        session_revision: Revision,
        expected_phase: Phase,
    ) -> Self {
        Self {
            session_id,
            session_revision,
            expected_phase,
        }
    }

    #[must_use]
    pub const fn session_id(self) -> SessionId {
        self.session_id
    }

    #[must_use]
    pub const fn session_revision(self) -> Revision {
        self.session_revision
    }

    #[must_use]
    pub const fn expected_phase(self) -> Phase {
        self.expected_phase
    }
}

/// Recognition correlation for live attempts.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RecognitionCorrelation {
    live: LiveCorrelation,
    attempt_id: crate::RecognitionAttemptId,
    attempt_revision: Revision,
}

/// Correlation for the one target-resolution operation issued at capture end.
/// The operation ID is intentionally independent from later session revisions,
/// allowing recognition and processing to complete in either order while still
/// rejecting fabricated, duplicate, and older callbacks exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TargetOperationCorrelation {
    live: LiveCorrelation,
    operation_id: OperationId,
}

impl TargetOperationCorrelation {
    #[must_use]
    pub const fn new(live: LiveCorrelation, operation_id: OperationId) -> Self {
        Self { live, operation_id }
    }

    #[must_use]
    pub const fn live(self) -> LiveCorrelation {
        self.live
    }

    #[must_use]
    pub const fn operation_id(self) -> OperationId {
        self.operation_id
    }
}

/// Correlation for delivery and manual-preservation operations.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DeliveryOperationCorrelation {
    live: LiveCorrelation,
    operation_id: OperationId,
}

impl DeliveryOperationCorrelation {
    #[must_use]
    pub const fn new(live: LiveCorrelation, operation_id: OperationId) -> Self {
        Self { live, operation_id }
    }

    #[must_use]
    pub const fn live(self) -> LiveCorrelation {
        self.live
    }

    #[must_use]
    pub const fn operation_id(self) -> OperationId {
        self.operation_id
    }
}

/// Exact subphases of a record-scoped recognition retry.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RetryPhase {
    PendingAttemptPersistence,
    Recognizing,
    PendingResultPersistence,
}

impl RetryPhase {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::PendingAttemptPersistence => "pending_attempt_persistence",
            Self::Recognizing => "recognizing",
            Self::PendingResultPersistence => "pending_result_persistence",
        }
    }

    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "pending_attempt_persistence" => Some(Self::PendingAttemptPersistence),
            "recognizing" => Some(Self::Recognizing),
            "pending_result_persistence" => Some(Self::PendingResultPersistence),
            _ => None,
        }
    }
}

impl RecognitionCorrelation {
    #[must_use]
    pub const fn new(
        live: LiveCorrelation,
        attempt_id: crate::RecognitionAttemptId,
        attempt_revision: Revision,
    ) -> Self {
        Self {
            live,
            attempt_id,
            attempt_revision,
        }
    }

    #[must_use]
    pub const fn live(self) -> LiveCorrelation {
        self.live
    }

    #[must_use]
    pub const fn attempt_id(self) -> crate::RecognitionAttemptId {
        self.attempt_id
    }

    #[must_use]
    pub const fn attempt_revision(self) -> Revision {
        self.attempt_revision
    }
}

/// Full correlation required by record-scoped retry callbacks.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RetryCorrelation {
    record_id: crate::DictationRecordId,
    originating_session_id: SessionId,
    attempt_id: crate::RecognitionAttemptId,
    attempt_revision: Revision,
    expected_phase: Phase,
    retry_phase: RetryPhase,
}

impl RetryCorrelation {
    #[must_use]
    pub const fn new(
        record_id: crate::DictationRecordId,
        originating_session_id: SessionId,
        attempt_id: crate::RecognitionAttemptId,
        attempt_revision: Revision,
        expected_phase: Phase,
    ) -> Self {
        let retry_phase = match expected_phase {
            Phase::Recognizing => RetryPhase::Recognizing,
            _ => RetryPhase::PendingAttemptPersistence,
        };
        Self::new_with_retry_phase(
            record_id,
            originating_session_id,
            attempt_id,
            attempt_revision,
            expected_phase,
            retry_phase,
        )
    }

    #[must_use]
    pub const fn new_with_retry_phase(
        record_id: crate::DictationRecordId,
        originating_session_id: SessionId,
        attempt_id: crate::RecognitionAttemptId,
        attempt_revision: Revision,
        expected_phase: Phase,
        retry_phase: RetryPhase,
    ) -> Self {
        Self {
            record_id,
            originating_session_id,
            attempt_id,
            attempt_revision,
            expected_phase,
            retry_phase,
        }
    }

    #[must_use]
    pub const fn record_id(self) -> crate::DictationRecordId {
        self.record_id
    }

    #[must_use]
    pub const fn originating_session_id(self) -> SessionId {
        self.originating_session_id
    }

    #[must_use]
    pub const fn attempt_id(self) -> crate::RecognitionAttemptId {
        self.attempt_id
    }

    #[must_use]
    pub const fn attempt_revision(self) -> Revision {
        self.attempt_revision
    }

    #[must_use]
    pub const fn expected_phase(self) -> Phase {
        self.expected_phase
    }

    #[must_use]
    pub const fn expected_retry_phase(self) -> RetryPhase {
        self.retry_phase
    }
}

/// Start command payload supplied by the application after allocating opaque IDs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartRequest {
    pub session_id: SessionId,
    pub record_id: crate::DictationRecordId,
    pub max_duration: CaptureLimit,
    pub recognition_timeout: DurationLimit,
    pub started_at: Timestamp,
    pub cancellation_token: crate::CancellationTokenId,
    pub recovery_id: crate::RecoveryId,
    pub recognition_attempt_id: crate::RecognitionAttemptId,
    pub recognition_configuration_id: ConfigurationId,
    pub processing_plan: crate::ProcessingPlan,
}

/// Result of processing a retained Raw Transcript.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessingResult {
    pub processed_text: Option<crate::ProcessedText>,
    pub final_text: crate::FinalText,
}

/// Material selections reported by persistence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistenceReport {
    pub durable_materials: Vec<MaterialKind>,
}
