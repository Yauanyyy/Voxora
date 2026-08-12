#![allow(
    clippy::collapsible_if,
    clippy::manual_let_else,
    clippy::must_use_candidate,
    clippy::redundant_closure_for_method_calls,
    clippy::unnested_or_patterns
)]

use crate::{
    CancellationTokenId, CaptureLimit, ConfigurationId, DeliveryCertainty,
    DeliveryOperationCorrelation, DictationRecord, DictationRecordId, DurationLimit, FailureCode,
    FailureStage, FinalText, InsertionTarget, LiveCorrelation, MaterialKind, MaterialLedger,
    MaterialState, OperationId, PartialTranscript, PersistenceReport, Phase, ProcessingPlan,
    ProcessingResult, RawTranscript, RecognitionAttempt, RecognitionAttemptId,
    RecognitionCorrelation, RecoveryContext, RecoveryId, RetryCorrelation, RetryMeaning,
    RetryPhase, Revision, SanitizedFailure, SessionId, StartMode, StartRequest,
    TargetOperationCorrelation, TargetResolution, TerminalOutcome, Timestamp, Warning,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LiveCommand {
    Start {
        mode: StartMode,
        request: StartRequest,
    },
    ReleasePushToTalk(LiveCorrelation),
    StopToggle(LiveCorrelation),
    Escape(LiveCorrelation),
    CaptureDeadlineReached(LiveCorrelation),
    RecognitionDeadlineReached(RecognitionCorrelation),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LiveEvent {
    CaptureStarted(LiveCorrelation),
    AudioLevel {
        correlation: LiveCorrelation,
        millibel: i16,
    },
    CaptureStoppedAt {
        correlation: LiveCorrelation,
        audio: Option<crate::RecordedAudio>,
        at: Timestamp,
    },
    CaptureFailed {
        correlation: LiveCorrelation,
        audio: Option<crate::RecordedAudio>,
    },
    CaptureCleanupCompleted {
        correlation: LiveCorrelation,
        audio_cancelled: bool,
        audio_discarded: bool,
        cancellation_cancelled: bool,
    },
    RecognitionPartial {
        correlation: RecognitionCorrelation,
        partial: PartialTranscript,
    },
    RecognitionFinal {
        correlation: RecognitionCorrelation,
        raw: RawTranscript,
    },
    RecognitionEmpty(RecognitionCorrelation),
    RecognitionFailed {
        correlation: RecognitionCorrelation,
        code: FailureCode,
    },
    RecognitionTimedOut(RecognitionCorrelation),
    RecognitionCancelled(RecognitionCorrelation),
    TargetResolvedForOperation {
        correlation: TargetOperationCorrelation,
        resolution: TargetResolution,
    },
    TargetInvalidated(LiveCorrelation),
    FocusChanged(LiveCorrelation),
    ProcessingSucceeded {
        correlation: LiveCorrelation,
        result: ProcessingResult,
    },
    ProcessingFailed {
        correlation: LiveCorrelation,
        code: FailureCode,
    },
    InsertionStartedForOperation(DeliveryOperationCorrelation),
    InsertionSucceededForOperation(DeliveryOperationCorrelation),
    InsertionFailedForOperation(DeliveryOperationCorrelation),
    InsertionUncertainForOperation(DeliveryOperationCorrelation),
    ResultPanelPresentedForOperation {
        correlation: DeliveryOperationCorrelation,
        presented: bool,
    },
    ClipboardFallbackForOperation {
        correlation: DeliveryOperationCorrelation,
        copied: bool,
    },
    PersistenceSucceededForOperation {
        correlation: LiveCorrelation,
        operation_id: OperationId,
        recovery_id: RecoveryId,
        report: PersistenceReport,
    },
    PersistenceFailedForOperation {
        correlation: LiveCorrelation,
        operation_id: OperationId,
        recovery_id: RecoveryId,
    },
    RecoveryPersistenceSucceeded {
        recovery: RecoveryCorrelation,
        report: PersistenceReport,
    },
    RecoveryPersistenceFailed(RecoveryCorrelation),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RecoveryCorrelation {
    recovery: RecoveryId,
    record: DictationRecordId,
    session: SessionId,
}

impl RecoveryCorrelation {
    #[must_use]
    pub const fn new(recovery: RecoveryId, record: DictationRecordId, session: SessionId) -> Self {
        Self {
            recovery,
            record,
            session,
        }
    }
    #[must_use]
    pub const fn recovery_id(self) -> RecoveryId {
        self.recovery
    }
    #[must_use]
    pub const fn record_id(self) -> DictationRecordId {
        self.record
    }
    #[must_use]
    pub const fn session_id(self) -> SessionId {
        self.session
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LiveInput {
    Command(LiveCommand),
    Event(LiveEvent),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LiveEffect {
    StartCapture {
        correlation: LiveCorrelation,
        max_duration: CaptureLimit,
        cancellation_token: CancellationTokenId,
    },
    StopCapture(LiveCorrelation),
    CancelCapture {
        correlation: LiveCorrelation,
        cancellation_token: CancellationTokenId,
    },
    CleanupCapture {
        correlation: LiveCorrelation,
        cancellation_token: CancellationTokenId,
    },
    RetryCaptureCleanup {
        correlation: LiveCorrelation,
        cancellation_token: CancellationTokenId,
    },
    DiscardCaptureAudio {
        session_id: SessionId,
    },
    Cancel(CancellationTokenId),
    ResolveTarget {
        correlation: TargetOperationCorrelation,
    },
    StartRecognition {
        correlation: RecognitionCorrelation,
        audio: crate::RecordedAudio,
        timeout: DurationLimit,
        deadline: Timestamp,
        cancellation_token: CancellationTokenId,
    },
    StartProcessing {
        correlation: LiveCorrelation,
        plan: ProcessingPlan,
        cancellation_token: CancellationTokenId,
    },
    BeginInsertion {
        correlation: DeliveryOperationCorrelation,
        target: InsertionTarget,
        final_text: FinalText,
    },
    PresentResultPanel {
        correlation: DeliveryOperationCorrelation,
        final_text: FinalText,
    },
    CopyToClipboard {
        correlation: DeliveryOperationCorrelation,
        final_text: FinalText,
    },
    PersistRecord {
        correlation: LiveCorrelation,
        operation_id: OperationId,
        recovery_id: RecoveryId,
        record: DictationRecord,
    },
    NotifyUnsavedHistory(LiveCorrelation),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RejectReason {
    NoActiveSession,
    CompetingWork,
    StaleSessionId,
    StaleRevision,
    UnexpectedPhase,
    WrongMode,
    DuplicateStop,
    TerminalCallback,
    StaleAttempt,
    StaleRecovery,
    RevisionOverflow,
    DeadlineOverflow,
    InvalidConfiguration,
    AllocationExhausted,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EventDisposition {
    Applied,
    Ignored(RejectReason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transition {
    state: LiveState,
    effects: Vec<LiveEffect>,
    disposition: EventDisposition,
}
impl Transition {
    #[must_use]
    pub fn applied(state: LiveState, effects: Vec<LiveEffect>) -> Self {
        Self {
            state,
            effects,
            disposition: EventDisposition::Applied,
        }
    }
    #[must_use]
    pub fn ignored(state: LiveState, reason: RejectReason) -> Self {
        Self {
            state,
            effects: Vec::new(),
            disposition: EventDisposition::Ignored(reason),
        }
    }
    #[must_use]
    pub const fn state(&self) -> &LiveState {
        &self.state
    }
    #[must_use]
    pub fn effects(&self) -> &[LiveEffect] {
        &self.effects
    }
    #[must_use]
    pub const fn disposition(&self) -> EventDisposition {
        self.disposition
    }
    #[must_use]
    pub fn into_parts(self) -> (LiveState, Vec<LiveEffect>, EventDisposition) {
        (self.state, self.effects, self.disposition)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum LiveState {
    #[default]
    Idle,
    Active(LiveSessionState),
    Terminal(LiveSessionState),
}
impl LiveState {
    #[must_use]
    pub const fn phase(&self) -> Phase {
        match self {
            Self::Idle => Phase::Idle,
            Self::Active(s) | Self::Terminal(s) => s.phase(),
        }
    }
    #[must_use]
    pub const fn session(&self) -> Option<&LiveSessionState> {
        match self {
            Self::Idle => None,
            Self::Active(s) | Self::Terminal(s) => Some(s),
        }
    }
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Active(_))
    }
    #[must_use]
    pub fn reset_to_idle(self) -> Self {
        if matches!(self, Self::Terminal(_)) {
            Self::Idle
        } else {
            self
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PendingDelivery {
    Insertion(DeliveryOperationCorrelation),
    Manual(DeliveryOperationCorrelation),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveSessionState {
    session_id: SessionId,
    record_id: DictationRecordId,
    session_revision: Revision,
    phase: Phase,
    start_mode: StartMode,
    max_duration: CaptureLimit,
    recognition_timeout: DurationLimit,
    started_at: Timestamp,
    capture_deadline: Timestamp,
    recognition_deadline: Option<Timestamp>,
    cancellation_token: CancellationTokenId,
    recognition_cancellation_token: Option<CancellationTokenId>,
    recovery_id: RecoveryId,
    attempt_id: RecognitionAttemptId,
    attempt_revision: Revision,
    recognition_configuration_id: ConfigurationId,
    processing_plan: ProcessingPlan,
    target_resolution: Option<TargetResolution>,
    target: Option<InsertionTarget>,
    target_operation: Option<TargetOperationCorrelation>,
    target_invalidated: bool,
    audio: Option<crate::RecordedAudio>,
    partial: Option<PartialTranscript>,
    raw: Option<RawTranscript>,
    processed: Option<crate::ProcessedText>,
    final_text: Option<FinalText>,
    materials: MaterialLedger,
    warnings: Vec<Warning>,
    failure: Option<SanitizedFailure>,
    outcome: Option<TerminalOutcome>,
    delivery_irreversible: bool,
    pending_delivery: Option<PendingDelivery>,
    pending_persistence: Option<(LiveCorrelation, OperationId)>,
    pending_cleanup: Option<LiveCorrelation>,
    recovery: Option<RecoveryContext>,
}
impl LiveSessionState {
    fn from_start(mode: StartMode, request: StartRequest) -> Result<Self, RejectReason> {
        let capture_deadline = request
            .max_duration
            .checked_deadline(request.started_at)
            .ok_or(RejectReason::DeadlineOverflow)?;
        Ok(Self {
            session_id: request.session_id,
            record_id: request.record_id,
            session_revision: Revision::first(),
            phase: Phase::Capturing,
            start_mode: mode,
            max_duration: request.max_duration,
            recognition_timeout: request.recognition_timeout,
            started_at: request.started_at,
            capture_deadline,
            recognition_deadline: None,
            cancellation_token: request.cancellation_token,
            recognition_cancellation_token: None,
            recovery_id: request.recovery_id,
            attempt_id: request.recognition_attempt_id,
            attempt_revision: Revision::first(),
            recognition_configuration_id: request.recognition_configuration_id,
            processing_plan: request.processing_plan,
            target_resolution: None,
            target: None,
            target_operation: None,
            target_invalidated: false,
            audio: None,
            partial: None,
            raw: None,
            processed: None,
            final_text: None,
            materials: MaterialLedger::new(),
            warnings: Vec::new(),
            failure: None,
            outcome: None,
            delivery_irreversible: false,
            pending_delivery: None,
            pending_persistence: None,
            pending_cleanup: None,
            recovery: None,
        })
    }
    #[must_use]
    pub const fn session_id(&self) -> SessionId {
        self.session_id
    }
    #[must_use]
    pub const fn record_id(&self) -> DictationRecordId {
        self.record_id
    }
    #[must_use]
    pub const fn session_revision(&self) -> Revision {
        self.session_revision
    }
    #[must_use]
    pub const fn phase(&self) -> Phase {
        self.phase
    }
    #[must_use]
    pub const fn start_mode(&self) -> StartMode {
        self.start_mode
    }
    #[must_use]
    pub const fn attempt_id(&self) -> RecognitionAttemptId {
        self.attempt_id
    }
    #[must_use]
    pub const fn attempt_revision(&self) -> Revision {
        self.attempt_revision
    }
    #[must_use]
    pub const fn cancellation_token(&self) -> CancellationTokenId {
        self.cancellation_token
    }
    #[must_use]
    pub const fn recognition_cancellation_token(&self) -> Option<CancellationTokenId> {
        self.recognition_cancellation_token
    }
    #[must_use]
    pub const fn recovery_id(&self) -> RecoveryId {
        self.recovery_id
    }
    #[must_use]
    pub const fn max_duration(&self) -> CaptureLimit {
        self.max_duration
    }
    #[must_use]
    pub const fn recognition_timeout(&self) -> DurationLimit {
        self.recognition_timeout
    }
    #[must_use]
    pub const fn started_at(&self) -> Timestamp {
        self.started_at
    }
    #[must_use]
    pub const fn deadline(&self) -> Option<Timestamp> {
        Some(self.capture_deadline)
    }
    #[must_use]
    pub const fn capture_deadline(&self) -> Timestamp {
        self.capture_deadline
    }
    #[must_use]
    pub const fn recognition_deadline(&self) -> Option<Timestamp> {
        self.recognition_deadline
    }
    #[must_use]
    pub const fn target(&self) -> Option<&InsertionTarget> {
        self.target.as_ref()
    }
    #[must_use]
    pub const fn target_resolution(&self) -> Option<&TargetResolution> {
        self.target_resolution.as_ref()
    }
    #[must_use]
    pub const fn target_operation(&self) -> Option<TargetOperationCorrelation> {
        self.target_operation
    }
    #[must_use]
    pub const fn target_invalidated(&self) -> bool {
        self.target_invalidated
    }
    #[must_use]
    pub const fn pending_cleanup(&self) -> Option<LiveCorrelation> {
        self.pending_cleanup
    }
    #[must_use]
    pub const fn audio(&self) -> Option<&crate::RecordedAudio> {
        self.audio.as_ref()
    }
    #[must_use]
    pub const fn partial(&self) -> Option<&PartialTranscript> {
        self.partial.as_ref()
    }
    #[must_use]
    pub const fn raw(&self) -> Option<&RawTranscript> {
        self.raw.as_ref()
    }
    #[must_use]
    pub const fn processed(&self) -> Option<&crate::ProcessedText> {
        self.processed.as_ref()
    }
    #[must_use]
    pub const fn final_text(&self) -> Option<&FinalText> {
        self.final_text.as_ref()
    }
    #[must_use]
    pub const fn materials(&self) -> &MaterialLedger {
        &self.materials
    }
    #[must_use]
    pub fn warnings(&self) -> &[Warning] {
        &self.warnings
    }
    #[must_use]
    pub const fn failure(&self) -> Option<SanitizedFailure> {
        self.failure
    }
    #[must_use]
    pub const fn outcome(&self) -> Option<TerminalOutcome> {
        self.outcome
    }
    #[must_use]
    pub const fn delivery_irreversible(&self) -> bool {
        self.delivery_irreversible
    }
    #[must_use]
    pub const fn recovery(&self) -> Option<&RecoveryContext> {
        self.recovery.as_ref()
    }
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        self.phase == Phase::Completed && self.outcome.is_some()
    }
    #[must_use]
    pub fn record_snapshot(&self) -> DictationRecord {
        let mut record = DictationRecord::new(self.record_id, self.session_id);
        if let Some(audio) = self.audio.clone() {
            record.set_recorded_audio(audio);
        }
        if let Some(partial) = self.partial.clone() {
            record.set_partial_transcript(partial);
        }
        if let Some(raw) = self.raw.clone() {
            record.set_raw_transcript(raw);
        }
        if let Some(processed) = self.processed.clone() {
            record.set_processed_text(processed);
        }
        if let Some(final_text) = self.final_text.clone() {
            record.set_final_text(final_text);
        }
        record.set_materials(self.materials.clone());
        record.set_warnings(self.warnings.clone());
        record.set_failure(self.failure);
        if let Some(outcome) = self.outcome {
            record.set_outcome(outcome);
        }
        if let Some(attempt) = recognition_attempt_snapshot(self) {
            record.append_attempt(attempt);
        }
        record
    }
}

fn live_corr(s: &LiveSessionState, phase: Phase) -> LiveCorrelation {
    LiveCorrelation::new(s.session_id, s.session_revision, phase)
}
fn next_revision(r: Revision) -> Result<Revision, RejectReason> {
    r.next().ok_or(RejectReason::RevisionOverflow)
}
fn operation_for(r: Revision) -> Result<OperationId, RejectReason> {
    OperationId::new(r.get()).ok_or(RejectReason::RevisionOverflow)
}
fn boundary_failure(
    stage: FailureStage,
    code: FailureCode,
    retry: RetryMeaning,
    certainty: DeliveryCertainty,
) -> SanitizedFailure {
    SanitizedFailure::from_boundary(stage, code, retry, certainty)
}
fn set_failure_if_absent(
    s: &mut LiveSessionState,
    stage: FailureStage,
    code: FailureCode,
    retry: RetryMeaning,
    certainty: DeliveryCertainty,
) {
    if s.failure.is_none() {
        s.failure = Some(boundary_failure(stage, code, retry, certainty));
    }
}
fn accept_live(s: &LiveSessionState, c: LiveCorrelation, phase: Phase) -> Result<(), RejectReason> {
    if s.session_id != c.session_id() {
        return Err(RejectReason::StaleSessionId);
    }
    if s.session_revision != c.session_revision() {
        return Err(RejectReason::StaleRevision);
    }
    if s.phase != phase || c.expected_phase() != phase {
        return Err(RejectReason::UnexpectedPhase);
    }
    Ok(())
}
fn accept_recognition(s: &LiveSessionState, c: RecognitionCorrelation) -> Result<(), RejectReason> {
    accept_live(s, c.live(), Phase::Recognizing)?;
    if s.attempt_id != c.attempt_id() || s.attempt_revision != c.attempt_revision() {
        return Err(RejectReason::StaleAttempt);
    }
    Ok(())
}
fn recognition_attempt_snapshot(s: &LiveSessionState) -> Option<RecognitionAttempt> {
    if s.partial.is_none()
        && s.raw.is_none()
        && !s
            .failure
            .is_some_and(|f| f.stage() == FailureStage::Recognition)
    {
        return None;
    }
    let mut attempt = RecognitionAttempt::new(
        s.attempt_id,
        s.attempt_revision,
        s.recognition_configuration_id,
    );
    if let Some(partial) = s.partial.clone() {
        attempt.accept_partial(partial);
    }
    if let Some(raw) = s.raw.clone() {
        attempt.accept_final(raw);
    } else if let Some(failure) = s.failure.filter(|f| f.stage() == FailureStage::Recognition) {
        attempt.fail(failure);
    }
    Some(attempt)
}
fn terminal_effects(s: &LiveSessionState) -> Vec<LiveEffect> {
    let Some((correlation, operation_id)) = s.pending_persistence else {
        return Vec::new();
    };
    vec![LiveEffect::PersistRecord {
        correlation,
        operation_id,
        recovery_id: s.recovery_id,
        record: s.record_snapshot(),
    }]
}
fn set_terminal(
    s: &mut LiveSessionState,
    outcome: TerminalOutcome,
    failure: Option<SanitizedFailure>,
) -> Result<(), RejectReason> {
    s.session_revision = next_revision(s.session_revision)?;
    s.phase = Phase::Completed;
    s.outcome = Some(outcome);
    if failure.is_some() {
        s.failure = failure;
    }
    s.pending_delivery = None;
    let op = operation_for(s.session_revision)?;
    let corr = live_corr(s, Phase::Completed);
    s.pending_persistence = Some((corr, op));
    Ok(())
}
fn set_terminal_without_persistence(
    s: &mut LiveSessionState,
    outcome: TerminalOutcome,
    failure: Option<SanitizedFailure>,
) -> Result<(), RejectReason> {
    s.session_revision = next_revision(s.session_revision)?;
    s.phase = Phase::Completed;
    s.outcome = Some(outcome);
    if failure.is_some() {
        s.failure = failure;
    }
    s.pending_delivery = None;
    s.pending_persistence = None;
    Ok(())
}
fn preserve_audio(s: &mut LiveSessionState, audio: Option<crate::RecordedAudio>) {
    if let Some(audio) = audio.filter(crate::RecordedAudio::has_samples) {
        s.audio = Some(audio);
        s.materials.mark_available(MaterialKind::RecordedAudio);
    }
}
fn preserve_partial_warning(s: &mut LiveSessionState) {
    if s.partial.is_some() && !s.warnings.contains(&Warning::IncompletePartialRetained) {
        s.warnings.push(Warning::IncompletePartialRetained);
    }
}
fn issue_manual(s: &mut LiveSessionState) -> Result<LiveEffect, RejectReason> {
    s.session_revision = next_revision(s.session_revision)?;
    let op = operation_for(s.session_revision)?;
    let corr = DeliveryOperationCorrelation::new(live_corr(s, Phase::Delivering), op);
    let final_text = s.final_text.clone().ok_or(RejectReason::UnexpectedPhase)?;
    s.pending_delivery = Some(PendingDelivery::Manual(corr));
    Ok(LiveEffect::PresentResultPanel {
        correlation: corr,
        final_text,
    })
}
fn issue_clipboard(s: &mut LiveSessionState) -> Result<LiveEffect, RejectReason> {
    s.session_revision = next_revision(s.session_revision)?;
    let op = operation_for(s.session_revision)?;
    let corr = DeliveryOperationCorrelation::new(live_corr(s, Phase::Delivering), op);
    let final_text = s.final_text.clone().ok_or(RejectReason::UnexpectedPhase)?;
    s.pending_delivery = Some(PendingDelivery::Manual(corr));
    Ok(LiveEffect::CopyToClipboard {
        correlation: corr,
        final_text,
    })
}
fn issue_insertion(s: &mut LiveSessionState) -> Result<LiveEffect, RejectReason> {
    let (target, final_text) = match (s.target.clone(), s.final_text.clone()) {
        (Some(t), Some(f)) => (t, f),
        _ => return Err(RejectReason::UnexpectedPhase),
    };
    s.session_revision = next_revision(s.session_revision)?;
    let op = operation_for(s.session_revision)?;
    let corr = DeliveryOperationCorrelation::new(live_corr(s, Phase::Delivering), op);
    s.pending_delivery = Some(PendingDelivery::Insertion(corr));
    Ok(LiveEffect::BeginInsertion {
        correlation: corr,
        target,
        final_text,
    })
}
fn accept_delivery(
    s: &LiveSessionState,
    c: DeliveryOperationCorrelation,
) -> Result<DeliveryOperationCorrelation, RejectReason> {
    let expected = match s.pending_delivery {
        Some(PendingDelivery::Insertion(c)) | Some(PendingDelivery::Manual(c)) => c,
        None => return Err(RejectReason::UnexpectedPhase),
    };
    if expected != c {
        return Err(RejectReason::StaleRevision);
    }
    Ok(expected)
}
fn accept_insertion(
    s: &LiveSessionState,
    c: DeliveryOperationCorrelation,
) -> Result<DeliveryOperationCorrelation, RejectReason> {
    let Some(PendingDelivery::Insertion(expected)) = s.pending_delivery else {
        return Err(RejectReason::UnexpectedPhase);
    };
    if expected != c {
        return Err(RejectReason::StaleRevision);
    }
    Ok(expected)
}
fn insertion_started(
    s: &LiveSessionState,
    c: DeliveryOperationCorrelation,
) -> Result<LiveSessionState, RejectReason> {
    accept_insertion(s, c)?;
    let mut next = s.clone();
    next.delivery_irreversible = true;
    Ok(next)
}
fn begin_uncertain(s: &mut LiveSessionState) -> Result<Vec<LiveEffect>, RejectReason> {
    s.delivery_irreversible = true;
    s.outcome = Some(TerminalOutcome::DeliveryUncertain);
    set_failure_if_absent(
        s,
        FailureStage::Delivery,
        FailureCode::InsertionUncertain,
        RetryMeaning::NoAutomaticRetry,
        DeliveryCertainty::Uncertain,
    );
    Ok(vec![issue_manual(s)?])
}
fn finish_manual(
    s: &mut LiveSessionState,
    c: DeliveryOperationCorrelation,
    result: Option<bool>,
) -> Result<(), RejectReason> {
    if !matches!(s.pending_delivery, Some(PendingDelivery::Manual(expected)) if expected == c) {
        return Err(RejectReason::UnexpectedPhase);
    }
    s.pending_delivery = None;
    match result {
        None => s.materials.mark_available(MaterialKind::ResultPanel),
        Some(true) => s.materials.mark_available(MaterialKind::ClipboardFallback),
        Some(false) => {
            set_failure_if_absent(
                s,
                FailureStage::Delivery,
                FailureCode::ManualPreservationFailed,
                RetryMeaning::Retryable,
                DeliveryCertainty::DefiniteFailure,
            );
            if s.outcome.is_none() {
                s.outcome = Some(TerminalOutcome::Failed);
            }
        }
    }
    let outcome = s.outcome.unwrap_or(TerminalOutcome::ManualDeliveryRequired);
    let has_recovery = s.recovery.is_some();
    if has_recovery {
        set_terminal_without_persistence(s, outcome, s.failure)?;
    } else {
        set_terminal(s, outcome, s.failure)?;
    }
    if s.recovery.is_some() {
        let record = s.record_snapshot();
        if let Some(context) = s.recovery.as_mut() {
            context.replace_record(record);
        }
    }
    Ok(())
}

pub fn reduce_live(state: &LiveState, input: LiveInput) -> Transition {
    match input {
        LiveInput::Command(c) => reduce_command(state, c),
        LiveInput::Event(e) => reduce_event(state, e),
    }
}
fn reduce_command(state: &LiveState, command: LiveCommand) -> Transition {
    match command {
        LiveCommand::Start { mode, request } => match state {
            LiveState::Idle => match LiveSessionState::from_start(mode, request.clone()) {
                Ok(session) => Transition::applied(
                    LiveState::Active(session.clone()),
                    vec![LiveEffect::StartCapture {
                        correlation: live_corr(&session, Phase::Capturing),
                        max_duration: request.max_duration,
                        cancellation_token: request.cancellation_token,
                    }],
                ),
                Err(reason) => Transition::ignored(state.clone(), reason),
            },
            _ => Transition::ignored(state.clone(), RejectReason::CompetingWork),
        },
        LiveCommand::ReleasePushToTalk(c) => stop_command(state, c, StartMode::PushToTalk),
        LiveCommand::StopToggle(c) => stop_command(state, c, StartMode::Toggle),
        LiveCommand::CaptureDeadlineReached(c) => {
            let Some(s) = state.session() else {
                return Transition::ignored(state.clone(), RejectReason::NoActiveSession);
            };
            if let Err(reason) = accept_live(s, c, Phase::Capturing) {
                return Transition::ignored(state.clone(), reason);
            }
            let mut next = s.clone();
            next.session_revision = match next_revision(next.session_revision) {
                Ok(r) => r,
                Err(reason) => return Transition::ignored(state.clone(), reason),
            };
            next.phase = Phase::StoppingCapture;
            next.warnings.push(Warning::MaximumDurationReached);
            Transition::applied(
                LiveState::Active(next.clone()),
                vec![LiveEffect::StopCapture(live_corr(
                    &next,
                    Phase::StoppingCapture,
                ))],
            )
        }
        LiveCommand::RecognitionDeadlineReached(c) => {
            let Some(s) = state.session() else {
                return Transition::ignored(state.clone(), RejectReason::NoActiveSession);
            };
            if let Err(reason) = accept_recognition(s, c) {
                return Transition::ignored(state.clone(), reason);
            }
            fail_recognition(state, s, FailureCode::RecognitionTimeout)
        }
        LiveCommand::Escape(c) => escape_command(state, c),
    }
}
fn stop_command(state: &LiveState, c: LiveCorrelation, mode: StartMode) -> Transition {
    let Some(s) = state.session() else {
        return Transition::ignored(state.clone(), RejectReason::NoActiveSession);
    };
    if s.session_id != c.session_id() {
        return Transition::ignored(state.clone(), RejectReason::StaleSessionId);
    }
    if s.session_revision != c.session_revision() {
        return Transition::ignored(state.clone(), RejectReason::StaleRevision);
    }
    if c.expected_phase() != Phase::Capturing || s.phase != Phase::Capturing {
        return Transition::ignored(
            state.clone(),
            if s.phase == Phase::StoppingCapture {
                RejectReason::DuplicateStop
            } else {
                RejectReason::UnexpectedPhase
            },
        );
    }
    if s.start_mode != mode {
        return Transition::ignored(state.clone(), RejectReason::WrongMode);
    }
    let mut next = s.clone();
    next.session_revision = match next_revision(next.session_revision) {
        Ok(r) => r,
        Err(reason) => return Transition::ignored(state.clone(), reason),
    };
    next.phase = Phase::StoppingCapture;
    Transition::applied(
        LiveState::Active(next.clone()),
        vec![LiveEffect::StopCapture(live_corr(
            &next,
            Phase::StoppingCapture,
        ))],
    )
}
fn escape_command(state: &LiveState, c: LiveCorrelation) -> Transition {
    let Some(s) = state.session() else {
        return Transition::ignored(state.clone(), RejectReason::NoActiveSession);
    };
    if s.session_id != c.session_id() {
        return Transition::ignored(state.clone(), RejectReason::StaleSessionId);
    }
    if s.session_revision != c.session_revision() {
        return Transition::ignored(state.clone(), RejectReason::StaleRevision);
    }
    if s.phase == Phase::Capturing {
        let mut next = s.clone();
        next.session_revision = match next_revision(next.session_revision) {
            Ok(r) => r,
            Err(reason) => return Transition::ignored(state.clone(), reason),
        };
        next.phase = Phase::Completed;
        next.outcome = Some(TerminalOutcome::Cancelled);
        next.audio = None;
        next.partial = None;
        next.raw = None;
        next.processed = None;
        next.final_text = None;
        next.materials = MaterialLedger::new();
        let cleanup =
            LiveCorrelation::new(next.session_id, next.session_revision, Phase::Completed);
        next.pending_cleanup = Some(cleanup);
        return Transition::applied(
            LiveState::Terminal(next),
            vec![LiveEffect::CleanupCapture {
                correlation: cleanup,
                cancellation_token: s.cancellation_token,
            }],
        );
    }
    if s.phase == Phase::Completed || s.phase == Phase::Recovery {
        return Transition::ignored(state.clone(), RejectReason::TerminalCallback);
    }
    let mut next = s.clone();
    if s.delivery_irreversible {
        next.session_revision = match next_revision(next.session_revision) {
            Ok(r) => r,
            Err(reason) => return Transition::ignored(state.clone(), reason),
        };
        return match begin_uncertain(&mut next) {
            Ok(effects) => Transition::applied(LiveState::Active(next), effects),
            Err(reason) => Transition::ignored(state.clone(), reason),
        };
    }
    next.session_revision = match next_revision(next.session_revision) {
        Ok(r) => r,
        Err(reason) => return Transition::ignored(state.clone(), reason),
    };
    let token = next
        .recognition_cancellation_token
        .unwrap_or(next.cancellation_token);
    next.phase = Phase::Completed;
    next.outcome = Some(TerminalOutcome::Cancelled);
    next.session_revision = match next_revision(next.session_revision) {
        Ok(r) => r,
        Err(reason) => return Transition::ignored(state.clone(), reason),
    };
    let op = match operation_for(next.session_revision) {
        Ok(op) => op,
        Err(reason) => return Transition::ignored(state.clone(), reason),
    };
    let pc = live_corr(&next, Phase::Completed);
    next.pending_persistence = Some((pc, op));
    let mut effects = vec![LiveEffect::Cancel(token)];
    effects.extend(terminal_effects(&next));
    Transition::applied(LiveState::Terminal(next), effects)
}

fn reduce_event(state: &LiveState, event: LiveEvent) -> Transition {
    let Some(s) = state.session() else {
        return Transition::ignored(state.clone(), RejectReason::NoActiveSession);
    };
    if matches!(state, LiveState::Terminal(_)) {
        return reduce_terminal_event(state, event);
    }
    reduce_active_event(s, event)
}
fn persistence_matches(
    s: &LiveSessionState,
    c: LiveCorrelation,
    op: OperationId,
) -> Result<(), RejectReason> {
    let Some((expected, expected_op)) = s.pending_persistence else {
        return Err(RejectReason::TerminalCallback);
    };
    if expected != c {
        return Err(RejectReason::StaleRevision);
    }
    if op != expected_op {
        return Err(RejectReason::StaleRevision);
    }
    Ok(())
}
#[allow(clippy::too_many_lines)]
fn reduce_terminal_event(state: &LiveState, event: LiveEvent) -> Transition {
    let LiveState::Terminal(s) = state else {
        return Transition::ignored(state.clone(), RejectReason::NoActiveSession);
    };
    match event {
        LiveEvent::CaptureCleanupCompleted {
            correlation,
            audio_cancelled,
            audio_discarded,
            cancellation_cancelled,
        } => {
            if s.pending_cleanup != Some(correlation) {
                return Transition::ignored(state.clone(), RejectReason::StaleRevision);
            }
            let mut next = s.clone();
            if audio_cancelled && audio_discarded && cancellation_cancelled {
                next.pending_cleanup = None;
                return Transition::applied(LiveState::Terminal(next), Vec::new());
            }
            if !(audio_cancelled && audio_discarded && cancellation_cancelled) {
                set_failure_if_absent(
                    &mut next,
                    FailureStage::Capture,
                    FailureCode::CaptureCleanupFailed,
                    RetryMeaning::Retryable,
                    DeliveryCertainty::NotApplicable,
                );
            }
            Transition::applied(
                LiveState::Terminal(next),
                vec![LiveEffect::RetryCaptureCleanup {
                    correlation,
                    cancellation_token: s.cancellation_token,
                }],
            )
        }
        LiveEvent::PersistenceSucceededForOperation {
            correlation,
            operation_id,
            recovery_id,
            report,
        } => {
            if let Err(reason) = persistence_matches(s, correlation, operation_id) {
                return Transition::ignored(state.clone(), reason);
            }
            if s.recovery_id != recovery_id {
                return Transition::ignored(state.clone(), RejectReason::StaleRecovery);
            }
            let mut next = s.clone();
            next.materials.mark_durable(&report.durable_materials);
            next.pending_persistence = None;
            if next.materials.all_available_durable() {
                return Transition::applied(LiveState::Terminal(next), Vec::new());
            }
            if !next.warnings.contains(&Warning::PersistenceUnsaved) {
                next.warnings.push(Warning::PersistenceUnsaved);
            }
            let mut record = next.record_snapshot();
            record.set_materials(next.materials.clone());
            record.set_warnings(next.warnings.clone());
            record.set_failure(next.failure);
            next.recovery = Some(RecoveryContext::new(recovery_id, record));
            next.phase = Phase::Recovery;
            let mut effects = vec![LiveEffect::NotifyUnsavedHistory(correlation)];
            if next.final_text.is_some()
                && next.outcome != Some(TerminalOutcome::DeliveredAutomatically)
                && !next.materials.state(MaterialKind::ResultPanel).available()
                && !next
                    .materials
                    .state(MaterialKind::ClipboardFallback)
                    .available()
            {
                if let Ok(effect) = issue_manual(&mut next) {
                    effects.push(effect);
                }
            }
            Transition::applied(LiveState::Terminal(next), effects)
        }
        LiveEvent::PersistenceFailedForOperation {
            correlation,
            operation_id,
            recovery_id,
        } => persistence_failed(state, s, correlation, operation_id, recovery_id),
        LiveEvent::RecoveryPersistenceSucceeded { recovery, report } => {
            let Some(context) = s.recovery.as_ref() else {
                return Transition::ignored(state.clone(), RejectReason::StaleRecovery);
            };
            if context.is_closed()
                || context.id() != recovery.recovery_id()
                || context.record_id() != recovery.record_id()
                || context.session_id() != recovery.session_id()
            {
                return Transition::ignored(state.clone(), RejectReason::StaleRecovery);
            }
            let mut next = s.clone();
            if let Some(context) = next.recovery.as_mut() {
                context.mark_durable(&report.durable_materials);
                next.materials.mark_durable(&report.durable_materials);
                if context.record().materials().all_available_durable() {
                    context.close();
                }
            }
            let closed = next
                .recovery
                .as_ref()
                .is_some_and(RecoveryContext::is_closed);
            if closed {
                next.phase = Phase::Completed;
                Transition::applied(LiveState::Terminal(next), Vec::new())
            } else {
                if !next.warnings.contains(&Warning::PersistenceUnsaved) {
                    next.warnings.push(Warning::PersistenceUnsaved);
                }
                Transition::applied(
                    LiveState::Terminal(next),
                    vec![LiveEffect::NotifyUnsavedHistory(LiveCorrelation::new(
                        s.session_id,
                        s.session_revision,
                        Phase::Recovery,
                    ))],
                )
            }
        }
        LiveEvent::RecoveryPersistenceFailed(recovery) => {
            let Some(context) = s.recovery.as_ref() else {
                return Transition::ignored(state.clone(), RejectReason::StaleRecovery);
            };
            if context.is_closed()
                || context.id() != recovery.recovery_id()
                || context.record_id() != recovery.record_id()
                || context.session_id() != recovery.session_id()
            {
                return Transition::ignored(state.clone(), RejectReason::StaleRecovery);
            }
            Transition::applied(state.clone(), Vec::new())
        }
        LiveEvent::ResultPanelPresentedForOperation {
            correlation,
            presented,
        } => terminal_manual_panel(state, s, correlation, presented),
        LiveEvent::ClipboardFallbackForOperation {
            correlation,
            copied,
        } => terminal_manual_clipboard(state, s, correlation, copied),
        _ => Transition::ignored(state.clone(), RejectReason::TerminalCallback),
    }
}
fn persistence_failed(
    state: &LiveState,
    s: &LiveSessionState,
    c: LiveCorrelation,
    op: OperationId,
    recovery_id: RecoveryId,
) -> Transition {
    if s.recovery_id != recovery_id {
        return Transition::ignored(state.clone(), RejectReason::StaleRecovery);
    }
    if let Err(reason) = persistence_matches(s, c, op) {
        return Transition::ignored(state.clone(), reason);
    }
    let mut next = s.clone();
    next.materials.mark_all_non_durable();
    if !next.warnings.contains(&Warning::PersistenceUnsaved) {
        next.warnings.push(Warning::PersistenceUnsaved);
    }
    let mut record = next.record_snapshot();
    record.set_materials(next.materials.clone());
    record.set_warnings(next.warnings.clone());
    record.set_failure(next.failure);
    next.recovery = Some(RecoveryContext::new(recovery_id, record));
    next.pending_persistence = None;
    next.phase = Phase::Recovery;
    let mut effects = vec![LiveEffect::NotifyUnsavedHistory(c)];
    if next.final_text.is_some()
        && next.outcome != Some(TerminalOutcome::DeliveredAutomatically)
        && !next.materials.state(MaterialKind::ResultPanel).available()
        && !next
            .materials
            .state(MaterialKind::ClipboardFallback)
            .available()
    {
        if let Ok(effect) = issue_manual(&mut next) {
            effects.push(effect);
        }
    }
    Transition::applied(LiveState::Terminal(next), effects)
}
fn terminal_manual_panel(
    state: &LiveState,
    s: &LiveSessionState,
    c: DeliveryOperationCorrelation,
    presented: bool,
) -> Transition {
    if s.recovery.as_ref().is_some_and(RecoveryContext::is_closed) {
        return Transition::ignored(state.clone(), RejectReason::StaleRecovery);
    }
    if !matches!(s.pending_delivery, Some(PendingDelivery::Manual(expected)) if expected == c) {
        return Transition::ignored(state.clone(), RejectReason::StaleRevision);
    }
    let mut next = s.clone();
    if presented {
        next.materials.mark_available(MaterialKind::ResultPanel);
        match finish_manual(&mut next, c, None) {
            Ok(()) => {
                Transition::applied(LiveState::Terminal(next.clone()), terminal_effects(&next))
            }
            Err(reason) => Transition::ignored(state.clone(), reason),
        }
    } else {
        match issue_clipboard(&mut next) {
            Ok(effect) => Transition::applied(LiveState::Terminal(next), vec![effect]),
            Err(reason) => Transition::ignored(state.clone(), reason),
        }
    }
}
fn terminal_manual_clipboard(
    state: &LiveState,
    s: &LiveSessionState,
    c: DeliveryOperationCorrelation,
    copied: bool,
) -> Transition {
    if s.recovery.as_ref().is_some_and(RecoveryContext::is_closed) {
        return Transition::ignored(state.clone(), RejectReason::StaleRecovery);
    }
    if !matches!(s.pending_delivery, Some(PendingDelivery::Manual(expected)) if expected == c) {
        return Transition::ignored(state.clone(), RejectReason::StaleRevision);
    }
    let mut next = s.clone();
    match finish_manual(&mut next, c, Some(copied)) {
        Ok(()) => Transition::applied(LiveState::Terminal(next.clone()), terminal_effects(&next)),
        Err(reason) => Transition::ignored(state.clone(), reason),
    }
}

#[allow(clippy::too_many_lines)]
fn reduce_active_event(s: &LiveSessionState, event: LiveEvent) -> Transition {
    match event {
        LiveEvent::CaptureStarted(c) => {
            if let Err(reason) = accept_live(s, c, Phase::Capturing) {
                Transition::ignored(LiveState::Active(s.clone()), reason)
            } else {
                Transition::applied(LiveState::Active(s.clone()), Vec::new())
            }
        }
        LiveEvent::AudioLevel {
            correlation,
            millibel,
        } => {
            if let Err(reason) = accept_live(s, correlation, Phase::Capturing) {
                return Transition::ignored(LiveState::Active(s.clone()), reason);
            }
            let mut next = s.clone();
            if millibel <= -50 && !next.warnings.contains(&Warning::LowVolume) {
                next.warnings.push(Warning::LowVolume);
            }
            Transition::applied(LiveState::Active(next), Vec::new())
        }
        LiveEvent::CaptureStoppedAt {
            correlation,
            audio,
            at,
        } => capture_stopped(s, correlation, audio, at),
        LiveEvent::CaptureFailed { correlation, audio } => {
            let expected = if s.phase == Phase::Capturing {
                Phase::Capturing
            } else {
                Phase::StoppingCapture
            };
            if let Err(reason) = accept_live(s, correlation, expected) {
                return Transition::ignored(LiveState::Active(s.clone()), reason);
            }
            let mut next = s.clone();
            preserve_audio(&mut next, audio);
            let failure = boundary_failure(
                FailureStage::Capture,
                FailureCode::DeviceFailure,
                RetryMeaning::Retryable,
                DeliveryCertainty::NotApplicable,
            );
            if let Err(reason) = set_terminal(&mut next, TerminalOutcome::Failed, Some(failure)) {
                return Transition::ignored(LiveState::Active(s.clone()), reason);
            }
            let cleanup = live_corr(&next, Phase::Completed);
            next.pending_cleanup = Some(cleanup);
            let mut effects = vec![LiveEffect::CleanupCapture {
                correlation: cleanup,
                cancellation_token: s.cancellation_token,
            }];
            effects.extend(terminal_effects(&next));
            Transition::applied(LiveState::Terminal(next), effects)
        }
        LiveEvent::RecognitionPartial {
            correlation,
            partial,
        } => {
            if let Err(reason) = accept_recognition(s, correlation) {
                return Transition::ignored(LiveState::Active(s.clone()), reason);
            }
            let mut next = s.clone();
            next.partial = Some(partial);
            next.materials
                .mark_available(MaterialKind::PartialTranscript);
            Transition::applied(LiveState::Active(next), Vec::new())
        }
        LiveEvent::RecognitionFinal { correlation, raw } => {
            if let Err(reason) = accept_recognition(s, correlation) {
                return Transition::ignored(LiveState::Active(s.clone()), reason);
            }
            if raw.as_str().trim().is_empty() {
                return fail_recognition(
                    &LiveState::Active(s.clone()),
                    s,
                    FailureCode::RecognitionEmpty,
                );
            }
            let mut next = s.clone();
            next.raw = Some(raw.clone());
            next.partial = None;
            next.materials.mark_available(MaterialKind::RawTranscript);
            next.materials
                .set(MaterialKind::PartialTranscript, MaterialState::Absent);
            next.processing_plan = next.processing_plan.with_raw_transcript(raw);
            next.session_revision = match next_revision(next.session_revision) {
                Ok(r) => r,
                Err(reason) => return Transition::ignored(LiveState::Active(s.clone()), reason),
            };
            next.phase = Phase::Processing;
            next.recognition_deadline = None;
            let c = live_corr(&next, Phase::Processing);
            Transition::applied(
                LiveState::Active(next.clone()),
                vec![LiveEffect::StartProcessing {
                    correlation: c,
                    plan: next.processing_plan.clone(),
                    cancellation_token: next.cancellation_token,
                }],
            )
        }
        LiveEvent::RecognitionEmpty(c) => {
            if let Err(reason) = accept_recognition(s, c) {
                Transition::ignored(LiveState::Active(s.clone()), reason)
            } else {
                fail_recognition(
                    &LiveState::Active(s.clone()),
                    s,
                    FailureCode::RecognitionEmpty,
                )
            }
        }
        LiveEvent::RecognitionFailed { correlation, code } => {
            if let Err(reason) = accept_recognition(s, correlation) {
                Transition::ignored(LiveState::Active(s.clone()), reason)
            } else {
                fail_recognition(&LiveState::Active(s.clone()), s, code)
            }
        }
        LiveEvent::RecognitionTimedOut(c) => {
            if let Err(reason) = accept_recognition(s, c) {
                Transition::ignored(LiveState::Active(s.clone()), reason)
            } else {
                fail_recognition(
                    &LiveState::Active(s.clone()),
                    s,
                    FailureCode::RecognitionTimeout,
                )
            }
        }
        LiveEvent::RecognitionCancelled(c) => {
            if let Err(reason) = accept_recognition(s, c) {
                Transition::ignored(LiveState::Active(s.clone()), reason)
            } else {
                fail_recognition(
                    &LiveState::Active(s.clone()),
                    s,
                    FailureCode::RecognitionCancelled,
                )
            }
        }
        LiveEvent::TargetResolvedForOperation {
            correlation,
            resolution,
        } => {
            if s.target_operation != Some(correlation)
                || !matches!(
                    s.phase,
                    Phase::Recognizing | Phase::Processing | Phase::Delivering
                )
            {
                return Transition::ignored(
                    LiveState::Active(s.clone()),
                    RejectReason::StaleRevision,
                );
            }
            let mut next = s.clone();
            next.target_operation = None;
            next.target = match &resolution {
                TargetResolution::Eligible(t) => Some(t.clone()),
                TargetResolution::Ineligible => None,
            };
            next.target_resolution = Some(resolution);
            if next.target.is_none() {
                set_failure_if_absent(
                    &mut next,
                    FailureStage::Targeting,
                    FailureCode::TargetUnavailable,
                    RetryMeaning::NoAutomaticRetry,
                    DeliveryCertainty::NotApplicable,
                );
            }
            let effects = if next.phase == Phase::Delivering && !next.delivery_irreversible {
                if next.target.is_some() {
                    match issue_insertion(&mut next) {
                        Ok(effect) => vec![effect],
                        Err(_) => Vec::new(),
                    }
                } else {
                    match issue_manual(&mut next) {
                        Ok(effect) => vec![effect],
                        Err(_) => Vec::new(),
                    }
                }
            } else {
                Vec::new()
            };
            Transition::applied(LiveState::Active(next), effects)
        }
        LiveEvent::TargetInvalidated(c) | LiveEvent::FocusChanged(c) => {
            if !matches!(
                s.phase,
                Phase::Recognizing | Phase::Processing | Phase::Delivering
            ) || s.target_invalidated
            {
                return Transition::ignored(
                    LiveState::Active(s.clone()),
                    RejectReason::StaleRevision,
                );
            }
            if let Err(reason) = accept_live(s, c, s.phase) {
                return Transition::ignored(LiveState::Active(s.clone()), reason);
            }
            let mut next = s.clone();
            next.target_invalidated = true;
            next.target_operation = None;
            next.target = None;
            next.target_resolution = Some(TargetResolution::Ineligible);
            set_failure_if_absent(
                &mut next,
                FailureStage::Targeting,
                FailureCode::TargetInvalid,
                RetryMeaning::NoAutomaticRetry,
                DeliveryCertainty::NotApplicable,
            );
            if !next.warnings.contains(&Warning::TargetChanged) {
                next.warnings.push(Warning::TargetChanged);
            }
            match s.phase {
                Phase::Recognizing | Phase::Processing => {
                    Transition::applied(LiveState::Active(next), Vec::new())
                }
                Phase::Delivering
                    if matches!(next.pending_delivery, Some(PendingDelivery::Manual(_))) =>
                {
                    Transition::applied(LiveState::Active(next), Vec::new())
                }
                Phase::Delivering if next.delivery_irreversible => {
                    next.pending_delivery = None;
                    match begin_uncertain(&mut next) {
                        Ok(effects) => Transition::applied(LiveState::Active(next), effects),
                        Err(reason) => Transition::ignored(LiveState::Active(s.clone()), reason),
                    }
                }
                Phase::Delivering => {
                    next.pending_delivery = None;
                    match issue_manual(&mut next) {
                        Ok(effect) => Transition::applied(LiveState::Active(next), vec![effect]),
                        Err(reason) => Transition::ignored(LiveState::Active(s.clone()), reason),
                    }
                }
                _ => {
                    Transition::ignored(LiveState::Active(s.clone()), RejectReason::UnexpectedPhase)
                }
            }
        }
        LiveEvent::ProcessingSucceeded {
            correlation,
            result,
        } => {
            if let Err(reason) = accept_live(s, correlation, Phase::Processing) {
                return Transition::ignored(LiveState::Active(s.clone()), reason);
            }
            let mut next = s.clone();
            next.processed = result.processed_text;
            next.final_text = Some(result.final_text);
            if next.processed.is_some() {
                next.materials.mark_available(MaterialKind::ProcessedText);
            }
            next.materials.mark_available(MaterialKind::FinalText);
            next.session_revision = match next_revision(next.session_revision) {
                Ok(r) => r,
                Err(reason) => return Transition::ignored(LiveState::Active(s.clone()), reason),
            };
            next.phase = Phase::Delivering;
            let effects = if next.target.is_some() {
                match issue_insertion(&mut next) {
                    Ok(effect) => vec![effect],
                    Err(_) => Vec::new(),
                }
            } else if next.target_resolution.is_some() {
                match issue_manual(&mut next) {
                    Ok(effect) => vec![effect],
                    Err(_) => Vec::new(),
                }
            } else {
                Vec::new()
            };
            Transition::applied(LiveState::Active(next), effects)
        }
        LiveEvent::ProcessingFailed { correlation, code } => {
            if let Err(reason) = accept_live(s, correlation, Phase::Processing) {
                return Transition::ignored(LiveState::Active(s.clone()), reason);
            }
            let Some(raw) = s.raw.clone() else {
                let mut next = s.clone();
                let f = boundary_failure(
                    FailureStage::Processing,
                    code,
                    RetryMeaning::NotRetryable,
                    DeliveryCertainty::NotApplicable,
                );
                if let Err(reason) = set_terminal(&mut next, TerminalOutcome::Failed, Some(f)) {
                    return Transition::ignored(LiveState::Active(s.clone()), reason);
                }
                let mut effects = Vec::new();
                if code == FailureCode::ProcessingTimeout {
                    effects.push(LiveEffect::Cancel(next.cancellation_token));
                }
                effects.extend(terminal_effects(&next));
                return Transition::applied(LiveState::Terminal(next.clone()), effects);
            };
            let mut next = s.clone();
            next.processed = None;
            next.final_text = Some(FinalText::new(raw.as_str()));
            set_failure_if_absent(
                &mut next,
                FailureStage::Processing,
                code,
                RetryMeaning::NotRetryable,
                DeliveryCertainty::NotApplicable,
            );
            next.materials.mark_available(MaterialKind::FinalText);
            if !next.warnings.contains(&Warning::ProcessingFallback) {
                next.warnings.push(Warning::ProcessingFallback);
            }
            next.session_revision = match next_revision(next.session_revision) {
                Ok(r) => r,
                Err(reason) => return Transition::ignored(LiveState::Active(s.clone()), reason),
            };
            next.phase = Phase::Delivering;
            let mut effects = Vec::new();
            if code == FailureCode::ProcessingTimeout {
                effects.push(LiveEffect::Cancel(next.cancellation_token));
            }
            effects.extend(if next.target.is_some() {
                match issue_insertion(&mut next) {
                    Ok(effect) => vec![effect],
                    Err(_) => Vec::new(),
                }
            } else if next.target_resolution.is_some() {
                match issue_manual(&mut next) {
                    Ok(effect) => vec![effect],
                    Err(_) => Vec::new(),
                }
            } else {
                Vec::new()
            });
            Transition::applied(LiveState::Active(next), effects)
        }
        LiveEvent::InsertionStartedForOperation(c) => match insertion_started(s, c) {
            Ok(next) => Transition::applied(LiveState::Active(next), Vec::new()),
            Err(reason) => Transition::ignored(LiveState::Active(s.clone()), reason),
        },
        LiveEvent::InsertionSucceededForOperation(c) => match accept_insertion(s, c) {
            Ok(_) => insertion_result(s, c, InjectionResult::Confirmed),
            Err(reason) => Transition::ignored(LiveState::Active(s.clone()), reason),
        },
        LiveEvent::InsertionFailedForOperation(c) => match accept_insertion(s, c) {
            Ok(_) => insertion_result(s, c, InjectionResult::DefiniteFailure),
            Err(reason) => Transition::ignored(LiveState::Active(s.clone()), reason),
        },
        LiveEvent::InsertionUncertainForOperation(c) => match accept_insertion(s, c) {
            Ok(_) => insertion_result(s, c, InjectionResult::Uncertain),
            Err(reason) => Transition::ignored(LiveState::Active(s.clone()), reason),
        },
        LiveEvent::ResultPanelPresentedForOperation {
            correlation,
            presented,
        } => match accept_delivery(s, correlation) {
            Ok(_) => panel_result(s, correlation, presented),
            Err(reason) => Transition::ignored(LiveState::Active(s.clone()), reason),
        },
        LiveEvent::ClipboardFallbackForOperation {
            correlation,
            copied,
        } => match accept_delivery(s, correlation) {
            Ok(_) => clipboard_result(s, correlation, copied),
            Err(reason) => Transition::ignored(LiveState::Active(s.clone()), reason),
        },
        LiveEvent::CaptureCleanupCompleted { .. }
        | LiveEvent::PersistenceSucceededForOperation { .. }
        | LiveEvent::PersistenceFailedForOperation { .. }
        | LiveEvent::RecoveryPersistenceSucceeded { .. }
        | LiveEvent::RecoveryPersistenceFailed(_) => {
            Transition::ignored(LiveState::Active(s.clone()), RejectReason::UnexpectedPhase)
        }
    }
}

fn capture_stopped(
    s: &LiveSessionState,
    c: LiveCorrelation,
    audio: Option<crate::RecordedAudio>,
    at: Timestamp,
) -> Transition {
    if let Err(reason) = accept_live(s, c, Phase::StoppingCapture) {
        return Transition::ignored(LiveState::Active(s.clone()), reason);
    }
    let mut next = s.clone();
    preserve_audio(&mut next, audio);
    let Some(audio) = next.audio.clone() else {
        let f = boundary_failure(
            FailureStage::Capture,
            FailureCode::EmptyAudio,
            RetryMeaning::NotRetryable,
            DeliveryCertainty::NotApplicable,
        );
        if let Err(reason) = set_terminal(&mut next, TerminalOutcome::Failed, Some(f)) {
            return Transition::ignored(LiveState::Active(s.clone()), reason);
        }
        return Transition::applied(LiveState::Terminal(next.clone()), terminal_effects(&next));
    };
    let deadline = match at.checked_add(next.recognition_timeout) {
        Some(d) => d,
        None => {
            return Transition::ignored(
                LiveState::Active(s.clone()),
                RejectReason::DeadlineOverflow,
            );
        }
    };
    next.session_revision = match next_revision(next.session_revision) {
        Ok(r) => r,
        Err(reason) => return Transition::ignored(LiveState::Active(s.clone()), reason),
    };
    next.phase = Phase::Recognizing;
    next.recognition_deadline = Some(deadline);
    next.recognition_cancellation_token = Some(next.cancellation_token);
    next.session_revision = match next_revision(next.session_revision) {
        Ok(r) => r,
        Err(reason) => return Transition::ignored(LiveState::Active(s.clone()), reason),
    };
    let op = match operation_for(next.session_revision) {
        Ok(op) => op,
        Err(reason) => return Transition::ignored(LiveState::Active(s.clone()), reason),
    };
    let target = TargetOperationCorrelation::new(live_corr(&next, Phase::Recognizing), op);
    next.target_operation = Some(target);
    let rec = RecognitionCorrelation::new(target.live(), next.attempt_id, next.attempt_revision);
    Transition::applied(
        LiveState::Active(next.clone()),
        vec![
            LiveEffect::ResolveTarget {
                correlation: target,
            },
            LiveEffect::StartRecognition {
                correlation: rec,
                audio,
                timeout: next.recognition_timeout,
                deadline,
                cancellation_token: next.cancellation_token,
            },
        ],
    )
}
fn fail_recognition(state: &LiveState, s: &LiveSessionState, code: FailureCode) -> Transition {
    let mut next = s.clone();
    preserve_partial_warning(&mut next);
    let f = boundary_failure(
        FailureStage::Recognition,
        code,
        RetryMeaning::Retryable,
        DeliveryCertainty::NotApplicable,
    );
    if let Err(reason) = set_terminal(&mut next, TerminalOutcome::Failed, Some(f)) {
        return Transition::ignored(state.clone(), reason);
    }
    let token = next
        .recognition_cancellation_token
        .unwrap_or(next.cancellation_token);
    let mut effects = vec![LiveEffect::Cancel(token)];
    effects.extend(terminal_effects(&next));
    Transition::applied(LiveState::Terminal(next), effects)
}
#[derive(Clone, Copy)]
enum InjectionResult {
    Confirmed,
    DefiniteFailure,
    Uncertain,
}
fn insertion_result(
    s: &LiveSessionState,
    _c: DeliveryOperationCorrelation,
    result: InjectionResult,
) -> Transition {
    let mut next = s.clone();
    next.pending_delivery = None;
    match result {
        InjectionResult::Confirmed => {
            next.delivery_irreversible = true;
            if let Err(reason) =
                set_terminal(&mut next, TerminalOutcome::DeliveredAutomatically, None)
            {
                return Transition::ignored(LiveState::Active(s.clone()), reason);
            }
            Transition::applied(LiveState::Terminal(next.clone()), terminal_effects(&next))
        }
        InjectionResult::DefiniteFailure => {
            set_failure_if_absent(
                &mut next,
                FailureStage::Delivery,
                FailureCode::InjectionFailed,
                RetryMeaning::Retryable,
                DeliveryCertainty::DefiniteFailure,
            );
            match issue_manual(&mut next) {
                Ok(effect) => Transition::applied(LiveState::Active(next), vec![effect]),
                Err(reason) => Transition::ignored(LiveState::Active(s.clone()), reason),
            }
        }
        InjectionResult::Uncertain => match begin_uncertain(&mut next) {
            Ok(effects) => Transition::applied(LiveState::Active(next), effects),
            Err(reason) => Transition::ignored(LiveState::Active(s.clone()), reason),
        },
    }
}
fn panel_result(
    s: &LiveSessionState,
    c: DeliveryOperationCorrelation,
    presented: bool,
) -> Transition {
    if !matches!(s.pending_delivery, Some(PendingDelivery::Manual(expected)) if expected == c) {
        return Transition::ignored(LiveState::Active(s.clone()), RejectReason::StaleRevision);
    }
    let mut next = s.clone();
    if presented {
        next.materials.mark_available(MaterialKind::ResultPanel);
        match finish_manual(&mut next, c, None) {
            Ok(()) => {
                Transition::applied(LiveState::Terminal(next.clone()), terminal_effects(&next))
            }
            Err(reason) => Transition::ignored(LiveState::Active(s.clone()), reason),
        }
    } else {
        match issue_clipboard(&mut next) {
            Ok(effect) => Transition::applied(LiveState::Active(next), vec![effect]),
            Err(reason) => Transition::ignored(LiveState::Active(s.clone()), reason),
        }
    }
}
fn clipboard_result(
    s: &LiveSessionState,
    c: DeliveryOperationCorrelation,
    copied: bool,
) -> Transition {
    if !matches!(s.pending_delivery, Some(PendingDelivery::Manual(expected)) if expected == c) {
        return Transition::ignored(LiveState::Active(s.clone()), RejectReason::StaleRevision);
    }
    let mut next = s.clone();
    match finish_manual(&mut next, c, Some(copied)) {
        Ok(()) => Transition::applied(LiveState::Terminal(next.clone()), terminal_effects(&next)),
        Err(reason) => Transition::ignored(LiveState::Active(s.clone()), reason),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RetryCommand {
    Begin {
        attempt_id: RecognitionAttemptId,
        configuration_id: ConfigurationId,
        timeout: DurationLimit,
        cancellation_token: CancellationTokenId,
        recovery_id: RecoveryId,
        started_at: Timestamp,
    },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RetryEvent {
    AttemptPersistenceSucceeded(RetryCorrelation),
    AttemptPersistenceFailed(RetryCorrelation),
    RecognitionPartial {
        correlation: RetryCorrelation,
        partial: PartialTranscript,
    },
    RecognitionFinal {
        correlation: RetryCorrelation,
        raw: RawTranscript,
    },
    RecognitionEmpty(RetryCorrelation),
    RecognitionFailed(RetryCorrelation),
    RecognitionTimedOut(RetryCorrelation),
    RecognitionCancelled(RetryCorrelation),
    ResultPersistenceSucceeded(RetryCorrelation),
    ResultPersistenceFailed(RetryCorrelation),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RetryEffect {
    PersistAttempt {
        correlation: RetryCorrelation,
        record: DictationRecord,
        attempt: RecognitionAttempt,
    },
    StartRecognition {
        correlation: RetryCorrelation,
        audio: crate::RecordedAudio,
        timeout: DurationLimit,
        deadline: Timestamp,
        cancellation_token: CancellationTokenId,
    },
    PersistResult {
        correlation: RetryCorrelation,
        record: DictationRecord,
        attempt: RecognitionAttempt,
    },
    ArchiveRecovery {
        recovery_id: RecoveryId,
        record: DictationRecord,
    },
    Cancel(CancellationTokenId),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryState {
    record: DictationRecord,
    pending_record: Option<DictationRecord>,
    active: Option<RetryAttemptState>,
    retry_revision: Revision,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct RetryAttemptState {
    correlation: RetryCorrelation,
    timeout: DurationLimit,
    deadline: Timestamp,
    cancellation_token: CancellationTokenId,
    recovery_id: RecoveryId,
    attempt: RecognitionAttempt,
}
impl RetryState {
    #[must_use]
    pub fn new(record: DictationRecord) -> Self {
        Self {
            record,
            pending_record: None,
            active: None,
            retry_revision: Revision::first(),
        }
    }
    #[must_use]
    pub const fn record(&self) -> &DictationRecord {
        &self.record
    }
    #[must_use]
    pub const fn pending_record(&self) -> Option<&DictationRecord> {
        self.pending_record.as_ref()
    }
    #[must_use]
    pub fn active(&self) -> Option<RetryCorrelation> {
        self.active.as_ref().map(|a| a.correlation)
    }
    #[must_use]
    pub const fn retry_revision(&self) -> Revision {
        self.retry_revision
    }
    #[must_use]
    pub fn deadline(&self) -> Option<Timestamp> {
        self.active.as_ref().map(|a| a.deadline)
    }
    #[must_use]
    pub fn cancellation_token(&self) -> Option<CancellationTokenId> {
        self.active.as_ref().map(|a| a.cancellation_token)
    }
    #[must_use]
    pub fn pending_attempt(&self) -> Option<&RecognitionAttempt> {
        self.active.as_ref().map(|a| &a.attempt)
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryTransition {
    state: RetryState,
    effects: Vec<RetryEffect>,
    disposition: EventDisposition,
}
impl RetryTransition {
    #[must_use]
    pub const fn disposition(&self) -> EventDisposition {
        self.disposition
    }
    #[must_use]
    pub const fn state(&self) -> &RetryState {
        &self.state
    }
    #[must_use]
    pub fn effects(&self) -> &[RetryEffect] {
        &self.effects
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RetryInput {
    Command(RetryCommand),
    Event(RetryEvent),
}
fn retry_ignored(state: &RetryState, reason: RejectReason) -> RetryTransition {
    RetryTransition {
        state: state.clone(),
        effects: Vec::new(),
        disposition: EventDisposition::Ignored(reason),
    }
}
fn retry_match(active: &RetryAttemptState, incoming: RetryCorrelation) -> Result<(), RejectReason> {
    if active.correlation == incoming {
        Ok(())
    } else if active.correlation.record_id() != incoming.record_id()
        || active.correlation.originating_session_id() != incoming.originating_session_id()
    {
        Err(RejectReason::StaleSessionId)
    } else {
        Err(RejectReason::StaleAttempt)
    }
}
fn retry_begin(
    state: &RetryState,
    attempt_id: RecognitionAttemptId,
    configuration_id: ConfigurationId,
    timeout: DurationLimit,
    token: CancellationTokenId,
    recovery_id: RecoveryId,
    started_at: Timestamp,
) -> RetryTransition {
    if state.active.is_some() {
        return retry_ignored(state, RejectReason::CompetingWork);
    }
    if state.record.outcome().is_none()
        || !state.record.is_durable()
        || !state.record.has_usable_durable_audio()
    {
        return retry_ignored(state, RejectReason::CompetingWork);
    }
    let revision = match state.record.attempts().last().map(|a| a.revision()) {
        Some(r) => match r.next() {
            Some(r) => r,
            None => return retry_ignored(state, RejectReason::RevisionOverflow),
        },
        None => Revision::first(),
    };
    let deadline = match started_at.checked_add(timeout) {
        Some(d) => d,
        None => return retry_ignored(state, RejectReason::DeadlineOverflow),
    };
    let attempt = RecognitionAttempt::new(attempt_id, revision, configuration_id);
    let corr = RetryCorrelation::new_with_retry_phase(
        state.record.id(),
        state.record.originating_session_id(),
        attempt_id,
        revision,
        Phase::Recognizing,
        RetryPhase::PendingAttemptPersistence,
    );
    let mut pending = state.record.clone();
    pending.append_attempt(attempt.clone());
    let mut next = state.clone();
    next.pending_record = Some(pending.clone());
    next.active = Some(RetryAttemptState {
        correlation: corr,
        timeout,
        deadline,
        cancellation_token: token,
        recovery_id,
        attempt: attempt.clone(),
    });
    next.retry_revision = revision;
    RetryTransition {
        state: next,
        effects: vec![RetryEffect::PersistAttempt {
            correlation: corr,
            record: pending,
            attempt,
        }],
        disposition: EventDisposition::Applied,
    }
}
#[must_use]
pub fn reduce_retry(state: &RetryState, input: RetryInput) -> RetryTransition {
    match input {
        RetryInput::Command(RetryCommand::Begin {
            attempt_id,
            configuration_id,
            timeout,
            cancellation_token,
            recovery_id,
            started_at,
        }) => retry_begin(
            state,
            attempt_id,
            configuration_id,
            timeout,
            cancellation_token,
            recovery_id,
            started_at,
        ),
        RetryInput::Event(event) => reduce_retry_event(state, event),
    }
}
fn retry_update_pending(
    state: &RetryState,
    code: FailureCode,
    final_text: Option<RawTranscript>,
) -> RetryTransition {
    let Some(active) = state.active.as_ref() else {
        return retry_ignored(state, RejectReason::NoActiveSession);
    };
    if active.correlation.expected_retry_phase() != RetryPhase::Recognizing {
        return retry_ignored(state, RejectReason::UnexpectedPhase);
    }
    let mut next = state.clone();
    let failure = final_text.is_none().then(|| {
        boundary_failure(
            FailureStage::Retry,
            code,
            RetryMeaning::Retryable,
            DeliveryCertainty::NotApplicable,
        )
    });
    let Some(active_next) = next.active.as_mut() else {
        return retry_ignored(state, RejectReason::NoActiveSession);
    };
    if let Some(raw) = final_text {
        active_next.attempt.accept_final(raw);
    } else if let Some(failure) = failure {
        active_next.attempt.fail(failure);
    }
    let attempt = active_next.attempt.clone();
    let mut record = next
        .pending_record
        .clone()
        .unwrap_or_else(|| next.record.clone());
    if let Some(last) = record.attempts_mut_last_for_m3() {
        *last = attempt.clone();
    }
    let corr = RetryCorrelation::new_with_retry_phase(
        active.correlation.record_id(),
        active.correlation.originating_session_id(),
        active.correlation.attempt_id(),
        active.correlation.attempt_revision(),
        Phase::Recognizing,
        RetryPhase::PendingResultPersistence,
    );
    active_next.correlation = corr;
    next.pending_record = Some(record.clone());
    let mut effects = vec![RetryEffect::PersistResult {
        correlation: corr,
        record,
        attempt,
    }];
    if failure.is_some() {
        effects.insert(0, RetryEffect::Cancel(active.cancellation_token));
    }
    RetryTransition {
        state: next,
        effects,
        disposition: EventDisposition::Applied,
    }
}
#[allow(clippy::too_many_lines)]
fn reduce_retry_event(state: &RetryState, event: RetryEvent) -> RetryTransition {
    let corr = match &event {
        RetryEvent::AttemptPersistenceSucceeded(c)
        | RetryEvent::AttemptPersistenceFailed(c)
        | RetryEvent::RecognitionPartial { correlation: c, .. }
        | RetryEvent::RecognitionFinal { correlation: c, .. }
        | RetryEvent::RecognitionEmpty(c)
        | RetryEvent::RecognitionFailed(c)
        | RetryEvent::RecognitionTimedOut(c)
        | RetryEvent::RecognitionCancelled(c)
        | RetryEvent::ResultPersistenceSucceeded(c)
        | RetryEvent::ResultPersistenceFailed(c) => *c,
    };
    let Some(active) = state.active.as_ref() else {
        return retry_ignored(state, RejectReason::NoActiveSession);
    };
    if let Err(reason) = retry_match(active, corr) {
        return retry_ignored(state, reason);
    }
    match event {
        RetryEvent::AttemptPersistenceSucceeded(_) => {
            if corr.expected_retry_phase() != RetryPhase::PendingAttemptPersistence {
                return retry_ignored(state, RejectReason::UnexpectedPhase);
            }
            let Some(audio) = state.record.recorded_audio().cloned() else {
                return retry_ignored(state, RejectReason::UnexpectedPhase);
            };
            let next_corr = RetryCorrelation::new_with_retry_phase(
                corr.record_id(),
                corr.originating_session_id(),
                corr.attempt_id(),
                corr.attempt_revision(),
                Phase::Recognizing,
                RetryPhase::Recognizing,
            );
            let mut next = state.clone();
            let Some(mut durable_record) = next.pending_record.clone() else {
                return retry_ignored(state, RejectReason::UnexpectedPhase);
            };
            durable_record.mark_durable();
            next.record = durable_record;
            if let Some(a) = next.active.as_mut() {
                a.correlation = next_corr;
            }
            RetryTransition {
                state: next,
                effects: vec![RetryEffect::StartRecognition {
                    correlation: next_corr,
                    audio,
                    timeout: active.timeout,
                    deadline: active.deadline,
                    cancellation_token: active.cancellation_token,
                }],
                disposition: EventDisposition::Applied,
            }
        }
        RetryEvent::AttemptPersistenceFailed(_) => {
            if corr.expected_retry_phase() != RetryPhase::PendingAttemptPersistence {
                return retry_ignored(state, RejectReason::UnexpectedPhase);
            }
            let mut next = state.clone();
            next.active = None;
            next.pending_record = None;
            RetryTransition {
                state: next,
                effects: Vec::new(),
                disposition: EventDisposition::Applied,
            }
        }
        RetryEvent::RecognitionPartial { partial, .. } => {
            if corr.expected_retry_phase() != RetryPhase::Recognizing {
                return retry_ignored(state, RejectReason::UnexpectedPhase);
            }
            let mut next = state.clone();
            if let Some(a) = next.active.as_mut() {
                a.attempt.accept_partial(partial);
            }
            RetryTransition {
                state: next,
                effects: Vec::new(),
                disposition: EventDisposition::Applied,
            }
        }
        RetryEvent::RecognitionFinal { raw, .. } => {
            if corr.expected_retry_phase() != RetryPhase::Recognizing {
                retry_ignored(state, RejectReason::UnexpectedPhase)
            } else if raw.as_str().trim().is_empty() {
                retry_update_pending(state, FailureCode::RetryEmpty, None)
            } else {
                retry_update_pending(state, FailureCode::RetryEmpty, Some(raw))
            }
        }
        RetryEvent::RecognitionEmpty(_) => {
            retry_update_pending(state, FailureCode::RetryEmpty, None)
        }
        RetryEvent::RecognitionFailed(_) => {
            retry_update_pending(state, FailureCode::RetryProvider, None)
        }
        RetryEvent::RecognitionTimedOut(_) => {
            retry_update_pending(state, FailureCode::RetryTimeout, None)
        }
        RetryEvent::RecognitionCancelled(_) => {
            retry_update_pending(state, FailureCode::RetryCancelled, None)
        }
        RetryEvent::ResultPersistenceSucceeded(_) => {
            if corr.expected_retry_phase() != RetryPhase::PendingResultPersistence {
                return retry_ignored(state, RejectReason::UnexpectedPhase);
            }
            let mut next = state.clone();
            let Some(record) = next.pending_record.take() else {
                return retry_ignored(state, RejectReason::UnexpectedPhase);
            };
            next.record = record;
            next.record.mark_durable();
            next.active = None;
            RetryTransition {
                state: next,
                effects: Vec::new(),
                disposition: EventDisposition::Applied,
            }
        }
        RetryEvent::ResultPersistenceFailed(_) => {
            if corr.expected_retry_phase() != RetryPhase::PendingResultPersistence {
                return retry_ignored(state, RejectReason::UnexpectedPhase);
            }
            let mut next = state.clone();
            next.active = None;
            RetryTransition {
                state: next,
                effects: state
                    .pending_record
                    .clone()
                    .map(|record| {
                        vec![RetryEffect::ArchiveRecovery {
                            recovery_id: active.recovery_id,
                            record,
                        }]
                    })
                    .unwrap_or_default(),
                disposition: EventDisposition::Applied,
            }
        }
    }
}
