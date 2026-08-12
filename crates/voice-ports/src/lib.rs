//! Portable capability contracts and deterministic scripted fakes.
//!
//! Ports are synchronous on purpose in M3.  Real adapters can translate their
//! callbacks into the correlated `voice_core` events without leaking provider or
//! platform types inward.

#![deny(unsafe_code)]
#![allow(clippy::assigning_clones, clippy::missing_errors_doc)]

use std::collections::{HashMap, VecDeque};

use voice_core::{
    AudioReferenceId, CancellationTokenId, CredentialReferenceId, CredentialSecret,
    DeliveryOperationCorrelation, DictationRecord, DictationRecordId, DurationLimit, FailureCode,
    FinalText, InsertionTarget, LiveCorrelation, ModelId, PersistenceReport, ProcessingPlan,
    ProcessingResult, RecognitionCorrelation, RecordedAudio, RecoveryCorrelation, RetryCorrelation,
    SanitizedFailure, SessionId, StartMode, TargetOperationCorrelation, TargetResolution,
    Timestamp,
};

/// Checked exhaustion reported by deterministic allocation/clock helpers.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AllocationError {
    Exhausted,
}

impl std::fmt::Display for AllocationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exhausted => formatter.write_str("deterministic allocation exhausted"),
        }
    }
}

impl std::error::Error for AllocationError {}

/// All port failures are already sanitized by the adapter boundary.
pub type PortResult<T> = Result<T, SanitizedFailure>;

/// An ordered, non-sensitive record of fake calls.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PortCall {
    AudioStart(SessionId),
    AudioStop(SessionId),
    AudioCancel(SessionId),
    AudioDiscard(SessionId),
    AudioDelete(AudioReferenceId),
    ShortcutRegister(StartMode),
    RecognitionStart(RecognitionCorrelationEnvelope),
    RecognitionCancel(CancellationTokenId),
    ProcessingStart(LiveCorrelation),
    ProcessingCancel(CancellationTokenId),
    ProcessingStep(&'static str),
    TargetResolve(TargetOperationCorrelation),
    TargetValidate(voice_core::TargetId),
    Insert(voice_core::TargetId),
    ResultPanel(DeliveryOperationCorrelation),
    Clipboard(DeliveryOperationCorrelation),
    CredentialRead(CredentialReferenceId),
    CredentialWrite(CredentialReferenceId),
    HistoryPersist(DictationRecordId),
    HistoryRecovery(DictationRecordId),
    ModelInspect(ModelId),
    ModelActivate(ModelId),
    ModelDelete(ModelId),
}

/// Capture request submitted by the application service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AudioStartRequest {
    pub session_id: SessionId,
    pub max_duration: DurationLimit,
    pub cancellation_token: CancellationTokenId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AudioStopRequest {
    pub session_id: SessionId,
}

/// Full correlation kind recorded by recognition fakes and adapters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecognitionCorrelationEnvelope {
    Live(RecognitionCorrelation),
    Retry(RetryCorrelation),
}

/// Recognition request. Responses are injected as correlated core events.
/// Live and history-retry requests are distinct at the port boundary so a
/// retry can never fabricate a live session revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecognitionRequest {
    Live {
        correlation: RecognitionCorrelation,
        audio: RecordedAudio,
        timeout: DurationLimit,
        cancellation_token: CancellationTokenId,
    },
    Retry {
        correlation: RetryCorrelation,
        audio: RecordedAudio,
        timeout: DurationLimit,
        cancellation_token: CancellationTokenId,
    },
}

/// Processing request.  The plan contains no provider/platform objects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessingRequest {
    pub correlation: LiveCorrelation,
    pub plan: ProcessingPlan,
    pub cancellation_token: CancellationTokenId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetRequest {
    pub correlation: TargetOperationCorrelation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsertionRequest {
    pub correlation: DeliveryOperationCorrelation,
    pub target: InsertionTarget,
    pub final_text: FinalText,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResultPanelRequest {
    pub correlation: DeliveryOperationCorrelation,
    pub final_text: FinalText,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardRequest {
    pub correlation: DeliveryOperationCorrelation,
    pub final_text: FinalText,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryPersistRequest {
    pub record_id: DictationRecordId,
    pub record: DictationRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryPersistRequest {
    pub correlation: RecoveryCorrelation,
    pub record: DictationRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryAttemptPersistRequest {
    pub correlation: RetryCorrelation,
    pub record: DictationRecord,
    pub attempt: voice_core::RecognitionAttempt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryResultPersistRequest {
    pub correlation: RetryCorrelation,
    pub record: DictationRecord,
    pub attempt: voice_core::RecognitionAttempt,
}

/// Capture and audio-artifact capability.
pub trait AudioCapturePort {
    fn start(&mut self, request: AudioStartRequest) -> PortResult<()>;
    fn stop(&mut self, request: AudioStopRequest) -> PortResult<()>;
    fn delete(&mut self, reference: AudioReferenceId) -> PortResult<()>;
    fn cancel(&mut self, request: AudioStopRequest) -> PortResult<()>;
    fn discard(&mut self, session_id: SessionId) -> PortResult<()>;
}

/// Global shortcut registration and event intake boundary.
pub trait ShortcutPort {
    fn register(&mut self, mode: StartMode) -> PortResult<()>;
}

/// Recognition provider boundary.  Partial/final callbacks are mapped to core events by the caller.
pub trait RecognitionEnginePort {
    fn recognize(&mut self, request: RecognitionRequest) -> PortResult<()>;
    fn cancel(&mut self, token: CancellationTokenId) -> PortResult<()>;
}

/// Ordered local/LLM processing boundary.
pub trait TextProcessorPort {
    fn process(&mut self, request: ProcessingRequest) -> PortResult<ProcessingResult>;
    fn cancel(&mut self, token: CancellationTokenId) -> PortResult<()>;
}

/// Current-focus target resolver.
pub trait TargetResolverPort {
    fn resolve(&mut self, request: TargetRequest) -> PortResult<TargetResolution>;
}

/// Delivery-time target validity check.
pub trait TargetValidatorPort {
    fn validate(&mut self, target: &InsertionTarget) -> PortResult<bool>;
}

/// External text insertion boundary.
pub trait TextInjectorPort {
    fn insert(&mut self, request: InsertionRequest) -> PortResult<InjectionDisposition>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InjectionDisposition {
    Confirmed,
    DefiniteFailure,
    Uncertain,
}

/// Non-focus-stealing manual result surface.
pub trait ResultPanelPort {
    fn present(&mut self, request: ResultPanelRequest) -> PortResult<bool>;
}

/// Clipboard last-resort boundary.
pub trait ClipboardPort {
    fn copy(&mut self, request: ClipboardRequest) -> PortResult<bool>;
}

/// Opaque credential store.  Implementations must not serialize the secret.
pub trait CredentialStorePort {
    fn read(&mut self, reference: CredentialReferenceId) -> PortResult<CredentialSecret>;
    fn write(
        &mut self,
        reference: CredentialReferenceId,
        secret: CredentialSecret,
    ) -> PortResult<()>;
}

/// History and recovery persistence boundary.
pub trait HistoryStorePort {
    fn persist(&mut self, request: HistoryPersistRequest) -> PortResult<PersistenceReport>;
    fn persist_recovery(
        &mut self,
        request: RecoveryPersistRequest,
    ) -> PortResult<PersistenceReport>;
    fn persist_retry_attempt(&mut self, request: RetryAttemptPersistRequest) -> PortResult<()>;
    fn persist_retry_result(&mut self, request: RetryResultPersistRequest) -> PortResult<()>;
}

/// Reviewed model artifact manager boundary.
pub trait ModelManagerPort {
    fn inspect(&mut self, model: ModelId) -> PortResult<ModelArtifactStatus>;
    fn activate(&mut self, model: ModelId) -> PortResult<()>;
    fn delete(&mut self, model: ModelId) -> PortResult<()>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelArtifactStatus {
    Available,
    Missing,
    Invalid,
}

/// Deterministic clock used by portable application services.
pub trait ClockPort {
    fn now(&self) -> Timestamp;
}

/// Cancellation is explicit and observable; it does not require an async runtime.
pub trait CancellationPort {
    fn allocate(&mut self) -> Result<CancellationTokenId, AllocationError>;
    fn cancel(&mut self, token: CancellationTokenId) -> PortResult<()>;
    fn is_cancelled(&self, token: CancellationTokenId) -> bool;
}

/// Opaque identifier allocation boundary.
pub trait IdentifierSource {
    fn next_session_id(&mut self) -> Result<SessionId, AllocationError>;
    fn next_record_id(&mut self) -> Result<DictationRecordId, AllocationError>;
    fn next_attempt_id(&mut self) -> Result<voice_core::RecognitionAttemptId, AllocationError>;
    fn next_operation_id(&mut self) -> Result<voice_core::OperationId, AllocationError>;
    fn next_recovery_id(&mut self) -> Result<voice_core::RecoveryId, AllocationError>;
}

/// Synthetic, deterministic identifier source.  IDs never derive from content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeterministicIdentifierSource {
    next: u64,
}

impl DeterministicIdentifierSource {
    #[must_use]
    pub fn new(start: u64) -> Self {
        Self {
            next: if start == 0 { 1 } else { start },
        }
    }

    fn take(&mut self) -> Result<u64, AllocationError> {
        if self.next == 0 {
            return Err(AllocationError::Exhausted);
        }
        let value = self.next;
        self.next = value.checked_add(1).unwrap_or(0);
        Ok(value)
    }
}

impl Default for DeterministicIdentifierSource {
    fn default() -> Self {
        Self::new(1)
    }
}

impl IdentifierSource for DeterministicIdentifierSource {
    fn next_session_id(&mut self) -> Result<SessionId, AllocationError> {
        SessionId::new(self.take()?).ok_or(AllocationError::Exhausted)
    }

    fn next_record_id(&mut self) -> Result<DictationRecordId, AllocationError> {
        DictationRecordId::new(self.take()?).ok_or(AllocationError::Exhausted)
    }

    fn next_attempt_id(&mut self) -> Result<voice_core::RecognitionAttemptId, AllocationError> {
        voice_core::RecognitionAttemptId::new(self.take()?).ok_or(AllocationError::Exhausted)
    }

    fn next_operation_id(&mut self) -> Result<voice_core::OperationId, AllocationError> {
        voice_core::OperationId::new(self.take()?).ok_or(AllocationError::Exhausted)
    }

    fn next_recovery_id(&mut self) -> Result<voice_core::RecoveryId, AllocationError> {
        voice_core::RecoveryId::new(self.take()?).ok_or(AllocationError::Exhausted)
    }
}

/// Deterministic mutable clock.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeterministicClock {
    now: Timestamp,
}

impl DeterministicClock {
    #[must_use]
    pub const fn new(now: Timestamp) -> Self {
        Self { now }
    }

    pub const fn set(&mut self, now: Timestamp) {
        self.now = now;
    }

    /// Advance the fake clock without mutating it when the addition overflows.
    pub fn advance(&mut self, milliseconds: u64) -> Result<(), AllocationError> {
        let next = self
            .now
            .milliseconds()
            .checked_add(milliseconds)
            .ok_or(AllocationError::Exhausted)?;
        self.now = Timestamp::new(next);
        Ok(())
    }
}

impl Default for DeterministicClock {
    fn default() -> Self {
        Self::new(Timestamp::new(0))
    }
}

impl ClockPort for DeterministicClock {
    fn now(&self) -> Timestamp {
        self.now
    }
}

/// Deterministic cancellation registry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeterministicCancellation {
    next: u64,
    cancelled: HashMap<CancellationTokenId, bool>,
    calls: Vec<PortCall>,
}

impl Default for DeterministicCancellation {
    fn default() -> Self {
        Self::new(1)
    }
}

impl DeterministicCancellation {
    #[must_use]
    pub fn new(start: u64) -> Self {
        Self {
            next: if start == 0 { 1 } else { start },
            cancelled: HashMap::new(),
            calls: Vec::new(),
        }
    }

    #[must_use]
    pub fn calls(&self) -> &[PortCall] {
        &self.calls
    }
}

impl CancellationPort for DeterministicCancellation {
    fn allocate(&mut self) -> Result<CancellationTokenId, AllocationError> {
        if self.next == 0 {
            return Err(AllocationError::Exhausted);
        }
        let value = self.next;
        self.next = value.checked_add(1).unwrap_or(0);
        let token = CancellationTokenId::new(value).ok_or(AllocationError::Exhausted)?;
        self.cancelled.insert(token, false);
        Ok(token)
    }

    fn cancel(&mut self, token: CancellationTokenId) -> PortResult<()> {
        self.cancelled.insert(token, true);
        self.calls.push(PortCall::RecognitionCancel(token));
        Ok(())
    }

    fn is_cancelled(&self, token: CancellationTokenId) -> bool {
        self.cancelled.get(&token).copied().unwrap_or(false)
    }
}

/// Scripted audio fake with ordered calls and queued outcomes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FakeAudioCapture {
    pub calls: Vec<PortCall>,
    pub start_results: VecDeque<PortResult<()>>,
    pub stop_results: VecDeque<PortResult<()>>,
    pub cancel_results: VecDeque<PortResult<()>>,
    pub discard_results: VecDeque<PortResult<()>>,
    pub delete_results: VecDeque<PortResult<()>>,
}

impl AudioCapturePort for FakeAudioCapture {
    fn start(&mut self, request: AudioStartRequest) -> PortResult<()> {
        self.calls.push(PortCall::AudioStart(request.session_id));
        self.start_results.pop_front().unwrap_or(Ok(()))
    }

    fn stop(&mut self, request: AudioStopRequest) -> PortResult<()> {
        self.calls.push(PortCall::AudioStop(request.session_id));
        self.stop_results.pop_front().unwrap_or(Ok(()))
    }

    fn cancel(&mut self, request: AudioStopRequest) -> PortResult<()> {
        self.calls.push(PortCall::AudioCancel(request.session_id));
        self.cancel_results.pop_front().unwrap_or(Ok(()))
    }

    fn discard(&mut self, session_id: SessionId) -> PortResult<()> {
        self.calls.push(PortCall::AudioDiscard(session_id));
        self.discard_results.pop_front().unwrap_or(Ok(()))
    }

    fn delete(&mut self, reference: AudioReferenceId) -> PortResult<()> {
        self.calls.push(PortCall::AudioDelete(reference));
        self.delete_results.pop_front().unwrap_or(Ok(()))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FakeShortcutRegistry {
    pub calls: Vec<PortCall>,
    pub results: VecDeque<PortResult<()>>,
}

impl ShortcutPort for FakeShortcutRegistry {
    fn register(&mut self, mode: StartMode) -> PortResult<()> {
        self.calls.push(PortCall::ShortcutRegister(mode));
        self.results.pop_front().unwrap_or(Ok(()))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FakeRecognitionEngine {
    pub calls: Vec<PortCall>,
    pub results: VecDeque<PortResult<()>>,
    pub cancel_results: VecDeque<PortResult<()>>,
}

impl RecognitionEnginePort for FakeRecognitionEngine {
    fn recognize(&mut self, request: RecognitionRequest) -> PortResult<()> {
        let correlation = match request {
            RecognitionRequest::Live { correlation, .. } => {
                RecognitionCorrelationEnvelope::Live(correlation)
            }
            RecognitionRequest::Retry { correlation, .. } => {
                RecognitionCorrelationEnvelope::Retry(correlation)
            }
        };
        self.calls.push(PortCall::RecognitionStart(correlation));
        self.results.pop_front().unwrap_or(Ok(()))
    }

    fn cancel(&mut self, token: CancellationTokenId) -> PortResult<()> {
        self.calls.push(PortCall::RecognitionCancel(token));
        self.cancel_results.pop_front().unwrap_or(Ok(()))
    }
}

/// Scripted processing fake.  Each enabled step records a stable label and
/// consumes one scripted result; disabled steps are skipped without a call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScriptedProcessingResult {
    Continue,
    Output(voice_core::ProcessedText),
    Fail(FailureCode),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FakeTextProcessor {
    pub calls: Vec<PortCall>,
    pub scripts: VecDeque<ScriptedProcessingResult>,
    pub cancel_results: VecDeque<PortResult<()>>,
}

impl TextProcessorPort for FakeTextProcessor {
    fn process(&mut self, request: ProcessingRequest) -> PortResult<ProcessingResult> {
        self.calls
            .push(PortCall::ProcessingStart(request.correlation));
        let mut working = request.plan.raw_transcript().as_str().to_owned();
        let mut processed = None;
        for step in request.plan.steps() {
            if !step.is_enabled() {
                continue;
            }
            let label = match step {
                voice_core::ProcessingStep::BuiltIn { .. } => "built_in",
                voice_core::ProcessingStep::LanguageModel { .. } => "llm",
            };
            self.calls.push(PortCall::ProcessingStep(label));
            match self
                .scripts
                .pop_front()
                .unwrap_or(ScriptedProcessingResult::Continue)
            {
                ScriptedProcessingResult::Continue => {}
                ScriptedProcessingResult::Output(output) => {
                    working = output.as_str().to_owned();
                    processed = Some(output);
                }
                ScriptedProcessingResult::Fail(code) => {
                    return Err(SanitizedFailure::from_boundary(
                        voice_core::FailureStage::Processing,
                        code,
                        voice_core::RetryMeaning::Retryable,
                        voice_core::DeliveryCertainty::NotApplicable,
                    ));
                }
            }
        }
        let final_text = FinalText::new(working);
        Ok(ProcessingResult {
            processed_text: processed,
            final_text,
        })
    }

    fn cancel(&mut self, token: CancellationTokenId) -> PortResult<()> {
        self.calls.push(PortCall::ProcessingCancel(token));
        self.cancel_results.pop_front().unwrap_or(Ok(()))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FakeTargetResolver {
    pub calls: Vec<PortCall>,
    pub results: VecDeque<PortResult<TargetResolution>>,
}

impl TargetResolverPort for FakeTargetResolver {
    fn resolve(&mut self, request: TargetRequest) -> PortResult<TargetResolution> {
        self.calls
            .push(PortCall::TargetResolve(request.correlation));
        self.results
            .pop_front()
            .unwrap_or(Ok(TargetResolution::Ineligible))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FakeTargetValidator {
    pub calls: Vec<PortCall>,
    pub results: VecDeque<PortResult<bool>>,
}

impl TargetValidatorPort for FakeTargetValidator {
    fn validate(&mut self, target: &InsertionTarget) -> PortResult<bool> {
        self.calls.push(PortCall::TargetValidate(target.id()));
        self.results.pop_front().unwrap_or(Ok(true))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FakeTextInjector {
    pub calls: Vec<PortCall>,
    pub results: VecDeque<PortResult<InjectionDisposition>>,
}

impl TextInjectorPort for FakeTextInjector {
    fn insert(&mut self, request: InsertionRequest) -> PortResult<InjectionDisposition> {
        self.calls.push(PortCall::Insert(request.target.id()));
        self.results
            .pop_front()
            .unwrap_or(Ok(InjectionDisposition::Confirmed))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FakeResultPanel {
    pub calls: Vec<PortCall>,
    pub results: VecDeque<PortResult<bool>>,
}

impl ResultPanelPort for FakeResultPanel {
    fn present(&mut self, request: ResultPanelRequest) -> PortResult<bool> {
        self.calls.push(PortCall::ResultPanel(request.correlation));
        self.results.pop_front().unwrap_or(Ok(true))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FakeClipboard {
    pub calls: Vec<PortCall>,
    pub results: VecDeque<PortResult<bool>>,
}

impl ClipboardPort for FakeClipboard {
    fn copy(&mut self, request: ClipboardRequest) -> PortResult<bool> {
        self.calls.push(PortCall::Clipboard(request.correlation));
        self.results.pop_front().unwrap_or(Ok(true))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FakeCredentialStore {
    pub calls: Vec<PortCall>,
    pub values: HashMap<CredentialReferenceId, CredentialSecret>,
}

impl CredentialStorePort for FakeCredentialStore {
    fn read(&mut self, reference: CredentialReferenceId) -> PortResult<CredentialSecret> {
        self.calls.push(PortCall::CredentialRead(reference));
        self.values.get(&reference).cloned().ok_or_else(|| {
            SanitizedFailure::from_boundary(
                voice_core::FailureStage::Credential,
                FailureCode::CredentialMissing,
                voice_core::RetryMeaning::Retryable,
                voice_core::DeliveryCertainty::NotApplicable,
            )
        })
    }

    fn write(
        &mut self,
        reference: CredentialReferenceId,
        secret: CredentialSecret,
    ) -> PortResult<()> {
        self.calls.push(PortCall::CredentialWrite(reference));
        self.values.insert(reference, secret);
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FakeHistoryStore {
    pub calls: Vec<PortCall>,
    pub persist_results: VecDeque<PortResult<PersistenceReport>>,
    pub recovery_results: VecDeque<PortResult<PersistenceReport>>,
    pub retry_results: VecDeque<PortResult<()>>,
}

impl HistoryStorePort for FakeHistoryStore {
    fn persist(&mut self, request: HistoryPersistRequest) -> PortResult<PersistenceReport> {
        self.calls.push(PortCall::HistoryPersist(request.record_id));
        self.persist_results
            .pop_front()
            .unwrap_or(Ok(PersistenceReport {
                durable_materials: request.record.materials().available_kinds(),
            }))
    }

    fn persist_recovery(
        &mut self,
        request: RecoveryPersistRequest,
    ) -> PortResult<PersistenceReport> {
        self.calls
            .push(PortCall::HistoryRecovery(request.correlation.record_id()));
        self.recovery_results
            .pop_front()
            .unwrap_or(Ok(PersistenceReport {
                durable_materials: request.record.materials().available_kinds(),
            }))
    }

    fn persist_retry_attempt(&mut self, request: RetryAttemptPersistRequest) -> PortResult<()> {
        self.calls
            .push(PortCall::HistoryPersist(request.correlation.record_id()));
        self.retry_results.pop_front().unwrap_or(Ok(()))
    }

    fn persist_retry_result(&mut self, request: RetryResultPersistRequest) -> PortResult<()> {
        self.calls
            .push(PortCall::HistoryPersist(request.correlation.record_id()));
        self.retry_results.pop_front().unwrap_or(Ok(()))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FakeModelManager {
    pub calls: Vec<PortCall>,
    pub inspect_results: VecDeque<PortResult<ModelArtifactStatus>>,
    pub activate_results: VecDeque<PortResult<()>>,
    pub delete_results: VecDeque<PortResult<()>>,
}

impl ModelManagerPort for FakeModelManager {
    fn inspect(&mut self, model: ModelId) -> PortResult<ModelArtifactStatus> {
        self.calls.push(PortCall::ModelInspect(model));
        self.inspect_results
            .pop_front()
            .unwrap_or(Ok(ModelArtifactStatus::Available))
    }

    fn activate(&mut self, model: ModelId) -> PortResult<()> {
        self.calls.push(PortCall::ModelActivate(model));
        self.activate_results.pop_front().unwrap_or(Ok(()))
    }

    fn delete(&mut self, model: ModelId) -> PortResult<()> {
        self.calls.push(PortCall::ModelDelete(model));
        self.delete_results.pop_front().unwrap_or(Ok(()))
    }
}

/// A small reusable ordered call log for application-level tests.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CallLog {
    calls: Vec<PortCall>,
}

impl CallLog {
    pub fn push(&mut self, call: PortCall) {
        self.calls.push(call);
    }

    #[must_use]
    pub fn calls(&self) -> &[PortCall] {
        &self.calls
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voice_core::{
        ApplicationIdentity, ConfigurationId, ProcessingStep, RawTranscript, Revision, TargetId,
        TargetToken,
    };

    #[test]
    fn deterministic_ids_and_clock_are_repeatable() {
        let mut ids = DeterministicIdentifierSource::new(10);
        assert_eq!(ids.next_session_id().unwrap().get(), 10);
        assert_eq!(ids.next_record_id().unwrap().get(), 11);
        let mut clock = DeterministicClock::new(Timestamp::new(4));
        clock.advance(3).unwrap();
        assert_eq!(clock.now().milliseconds(), 7);
    }

    #[test]
    fn scripted_processor_skips_disabled_steps_and_records_order() {
        let live = LiveCorrelation::new(
            SessionId::new(1).unwrap(),
            Revision::first(),
            voice_core::Phase::Processing,
        );
        let mut processor = FakeTextProcessor {
            scripts: VecDeque::from([ScriptedProcessingResult::Output(
                voice_core::ProcessedText::new("synthetic output"),
            )]),
            ..FakeTextProcessor::default()
        };
        let request = ProcessingRequest {
            correlation: live,
            plan: ProcessingPlan::new(
                RawTranscript::new("synthetic raw"),
                vec![
                    ProcessingStep::BuiltIn {
                        rule_id: ConfigurationId::new(2).unwrap(),
                        enabled: false,
                    },
                    ProcessingStep::LanguageModel {
                        configuration_id: Some(ConfigurationId::new(3).unwrap()),
                        enabled: true,
                    },
                ],
            )
            .unwrap(),
            cancellation_token: CancellationTokenId::new(4).unwrap(),
        };
        let result = processor.process(request).unwrap();
        assert_eq!(result.final_text.as_str(), "synthetic output");
        assert_eq!(
            processor.calls,
            vec![
                PortCall::ProcessingStart(live),
                PortCall::ProcessingStep("llm")
            ]
        );
    }

    #[test]
    fn target_and_secret_debug_are_redacted() {
        let target = InsertionTarget::new(
            TargetId::new(2).unwrap(),
            TargetToken::new("private-token"),
            Some(ApplicationIdentity::new("private-app")),
        );
        assert!(!format!("{target:?}").contains("private"));
        let secret = CredentialSecret::new("private-secret");
        assert!(!format!("{secret:?}").contains("private"));
    }

    #[test]
    fn recognition_fake_keeps_live_and_retry_correlations_distinct() {
        let live = RecognitionCorrelation::new(
            LiveCorrelation::new(
                SessionId::new(1).unwrap(),
                Revision::first(),
                voice_core::Phase::Recognizing,
            ),
            voice_core::RecognitionAttemptId::new(2).unwrap(),
            Revision::first(),
        );
        let retry = voice_core::RetryCorrelation::new(
            voice_core::DictationRecordId::new(3).unwrap(),
            SessionId::new(4).unwrap(),
            voice_core::RecognitionAttemptId::new(5).unwrap(),
            Revision::first(),
            voice_core::Phase::Recognizing,
        );
        let audio = RecordedAudio::new(AudioReferenceId::new(6).unwrap(), true);
        let mut engine = FakeRecognitionEngine::default();
        engine
            .recognize(RecognitionRequest::Live {
                correlation: live,
                audio: audio.clone(),
                timeout: DurationLimit::from_seconds(1).unwrap(),
                cancellation_token: CancellationTokenId::new(7).unwrap(),
            })
            .unwrap();
        engine
            .recognize(RecognitionRequest::Retry {
                correlation: retry,
                audio,
                timeout: DurationLimit::from_seconds(1).unwrap(),
                cancellation_token: CancellationTokenId::new(8).unwrap(),
            })
            .unwrap();
        assert_eq!(
            engine.calls,
            vec![
                PortCall::RecognitionStart(RecognitionCorrelationEnvelope::Live(live)),
                PortCall::RecognitionStart(RecognitionCorrelationEnvelope::Retry(retry)),
            ]
        );
    }

    #[test]
    fn deterministic_allocators_and_clock_fail_without_wrap_or_mutation() {
        let mut ids = DeterministicIdentifierSource::new(u64::MAX);
        assert_eq!(ids.next_session_id().unwrap().get(), u64::MAX);
        assert_eq!(ids.next_record_id(), Err(AllocationError::Exhausted));

        let mut cancellation = DeterministicCancellation::new(u64::MAX);
        assert_eq!(cancellation.allocate().unwrap().get(), u64::MAX);
        assert_eq!(cancellation.allocate(), Err(AllocationError::Exhausted));

        let mut clock = DeterministicClock::new(Timestamp::new(u64::MAX - 1));
        assert_eq!(clock.advance(2), Err(AllocationError::Exhausted));
        assert_eq!(clock.now(), Timestamp::new(u64::MAX - 1));
    }

    #[test]
    fn fake_audio_and_processor_make_cancellation_calls_observable() {
        let session = SessionId::new(10).unwrap();
        let token = CancellationTokenId::new(11).unwrap();
        let mut audio = FakeAudioCapture::default();
        audio
            .cancel(AudioStopRequest {
                session_id: session,
            })
            .unwrap();
        audio.discard(session).unwrap();
        assert_eq!(
            audio.calls,
            vec![
                PortCall::AudioCancel(session),
                PortCall::AudioDiscard(session)
            ]
        );

        let mut processor = FakeTextProcessor::default();
        processor.cancel(token).unwrap();
        assert_eq!(processor.calls, vec![PortCall::ProcessingCancel(token)]);
    }
}
