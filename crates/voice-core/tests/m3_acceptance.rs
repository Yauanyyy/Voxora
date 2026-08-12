#![allow(clippy::similar_names)]

use voice_core::{
    AudioReferenceId, CancellationTokenId, CaptureLimit, ConfigurationId, DictationRecord,
    DictationRecordId, Durability, DurationLimit, EventDisposition, FailureCode, FailureStage,
    FinalText, InsertionTarget, LiveCommand, LiveCorrelation, LiveEffect, LiveEvent, LiveInput,
    LiveState, MaterialKind, MaterialLedger, MaterialState, PersistenceReport, Phase,
    ProcessingPlan, ProcessingResult, ProcessingStep, RawTranscript, RecognitionAttemptId,
    RecognitionCorrelation, RejectReason, RetryCommand, RetryEvent, RetryInput, RetryPhase,
    RetryState, SessionId, StartMode, StartRequest, TargetId, TargetOperationCorrelation,
    TargetResolution, TargetToken, TerminalOutcome, Timestamp, Warning, reduce_live, reduce_retry,
};

fn capture_limit(seconds: u64) -> CaptureLimit {
    CaptureLimit::from_seconds(seconds).expect("valid capture limit")
}

fn request() -> StartRequest {
    StartRequest {
        session_id: SessionId::new(1).unwrap(),
        record_id: DictationRecordId::new(2).unwrap(),
        max_duration: capture_limit(60),
        recognition_timeout: DurationLimit::from_seconds(2).unwrap(),
        started_at: Timestamp::new(10),
        cancellation_token: CancellationTokenId::new(3).unwrap(),
        recovery_id: voice_core::RecoveryId::new(9).unwrap(),
        recognition_attempt_id: RecognitionAttemptId::new(4).unwrap(),
        recognition_configuration_id: ConfigurationId::new(5).unwrap(),
        processing_plan: ProcessingPlan::new(
            RawTranscript::new("placeholder"),
            vec![ProcessingStep::BuiltIn {
                rule_id: ConfigurationId::new(6).unwrap(),
                enabled: true,
            }],
        )
        .unwrap(),
    }
}

fn started() -> LiveState {
    reduce_live(
        &LiveState::Idle,
        LiveInput::Command(LiveCommand::Start {
            mode: StartMode::PushToTalk,
            request: request(),
        }),
    )
    .state()
    .clone()
}

fn stopping(state: &LiveState) -> LiveState {
    let session = state.session().unwrap();
    reduce_live(
        state,
        LiveInput::Command(LiveCommand::ReleasePushToTalk(LiveCorrelation::new(
            session.session_id(),
            session.session_revision(),
            Phase::Capturing,
        ))),
    )
    .state()
    .clone()
}

fn recognizing(state: &LiveState) -> LiveState {
    let state = stopping(state);
    let session = state.session().unwrap();
    reduce_live(
        &state,
        LiveInput::Event(LiveEvent::CaptureStoppedAt {
            correlation: LiveCorrelation::new(
                session.session_id(),
                session.session_revision(),
                Phase::StoppingCapture,
            ),
            audio: Some(voice_core::RecordedAudio::new(
                AudioReferenceId::new(7).unwrap(),
                true,
            )),
            at: Timestamp::new(12),
        }),
    )
    .state()
    .clone()
}

fn recognition_correlation(state: &LiveState) -> RecognitionCorrelation {
    let session = state.session().unwrap();
    RecognitionCorrelation::new(
        LiveCorrelation::new(
            session.session_id(),
            session.session_revision(),
            Phase::Recognizing,
        ),
        session.attempt_id(),
        session.attempt_revision(),
    )
}

fn target_correlation(state: &LiveState) -> TargetOperationCorrelation {
    state.session().unwrap().target_operation().unwrap()
}

fn processing(state: &LiveState) -> LiveState {
    let state = recognizing(state);
    let correlation = recognition_correlation(&state);
    reduce_live(
        &state,
        LiveInput::Event(LiveEvent::RecognitionFinal {
            correlation,
            raw: RawTranscript::new("synthetic raw"),
        }),
    )
    .state()
    .clone()
}

fn target() -> TargetResolution {
    TargetResolution::Eligible(InsertionTarget::new(
        TargetId::new(8).unwrap(),
        TargetToken::new("synthetic-target"),
        None,
    ))
}

fn delivering_with_insertion() -> (LiveState, voice_core::DeliveryOperationCorrelation) {
    let recognizing_state = recognizing(&started());
    let with_target = reduce_live(
        &recognizing_state,
        LiveInput::Event(LiveEvent::TargetResolvedForOperation {
            correlation: target_correlation(&recognizing_state),
            resolution: target(),
        }),
    );
    let recognition = recognition_correlation(&recognizing_state);
    let processing = reduce_live(
        with_target.state(),
        LiveInput::Event(LiveEvent::RecognitionFinal {
            correlation: recognition,
            raw: RawTranscript::new("synthetic raw"),
        }),
    );
    let processing_session = processing.state().session().unwrap();
    let delivering = reduce_live(
        processing.state(),
        LiveInput::Event(LiveEvent::ProcessingSucceeded {
            correlation: LiveCorrelation::new(
                processing_session.session_id(),
                processing_session.session_revision(),
                Phase::Processing,
            ),
            result: ProcessingResult {
                processed_text: None,
                final_text: FinalText::new("synthetic raw"),
            },
        }),
    );
    let insertion = delivering
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::BeginInsertion { correlation, .. } => Some(*correlation),
            _ => None,
        })
        .expect("targeted delivery must issue insertion");
    (delivering.state().clone(), insertion)
}

#[test]
fn capture_limit_bounds_and_checked_deadline_overflow() {
    assert!(CaptureLimit::from_seconds(59).is_none());
    assert!(CaptureLimit::from_seconds(301).is_none());
    let mut invalid = request();
    invalid.started_at = Timestamp::new(u64::MAX);
    let transition = reduce_live(
        &LiveState::Idle,
        LiveInput::Command(LiveCommand::Start {
            mode: StartMode::Toggle,
            request: invalid,
        }),
    );
    assert_eq!(
        transition.disposition(),
        EventDisposition::Ignored(RejectReason::DeadlineOverflow)
    );
    assert!(transition.effects().is_empty());
}

#[test]
fn mode_guard_maximum_and_capture_discard_are_exact() {
    let state = started();
    let session = state.session().unwrap();
    let wrong = reduce_live(
        &state,
        LiveInput::Command(LiveCommand::StopToggle(LiveCorrelation::new(
            session.session_id(),
            session.session_revision(),
            Phase::Capturing,
        ))),
    );
    assert_eq!(
        wrong.disposition(),
        EventDisposition::Ignored(RejectReason::WrongMode)
    );
    let deadline = reduce_live(
        &state,
        LiveInput::Command(LiveCommand::CaptureDeadlineReached(LiveCorrelation::new(
            session.session_id(),
            session.session_revision(),
            Phase::Capturing,
        ))),
    );
    assert_eq!(deadline.state().phase(), Phase::StoppingCapture);
    assert!(
        deadline
            .state()
            .session()
            .unwrap()
            .warnings()
            .contains(&Warning::MaximumDurationReached)
    );

    let escaped = reduce_live(
        &state,
        LiveInput::Command(LiveCommand::Escape(LiveCorrelation::new(
            session.session_id(),
            session.session_revision(),
            Phase::Capturing,
        ))),
    );
    assert_eq!(
        escaped.state().session().unwrap().outcome(),
        Some(TerminalOutcome::Cancelled)
    );
    assert!(
        escaped
            .effects()
            .iter()
            .any(|effect| matches!(effect, LiveEffect::CleanupCapture { .. }))
    );
    assert!(
        !escaped
            .state()
            .session()
            .unwrap()
            .materials()
            .state(MaterialKind::RecordedAudio)
            .available()
    );
}

#[test]
fn capture_stop_rejects_stale_duplicate_and_wrong_mode_without_mutation() {
    let state = started();
    let session = state.session().unwrap();
    let expected = LiveCorrelation::new(
        session.session_id(),
        session.session_revision(),
        Phase::Capturing,
    );
    let stale_session = LiveCorrelation::new(
        SessionId::new(99).unwrap(),
        session.session_revision(),
        Phase::Capturing,
    );
    let stale = reduce_live(
        &state,
        LiveInput::Command(LiveCommand::ReleasePushToTalk(stale_session)),
    );
    assert_eq!(
        stale.disposition(),
        EventDisposition::Ignored(RejectReason::StaleSessionId)
    );
    assert_eq!(stale.state(), &state);
    assert!(stale.effects().is_empty());

    let stopped = reduce_live(
        &state,
        LiveInput::Command(LiveCommand::ReleasePushToTalk(expected)),
    );
    assert_eq!(stopped.disposition(), EventDisposition::Applied);
    assert_eq!(stopped.state().phase(), Phase::StoppingCapture);
    assert_eq!(stopped.effects().len(), 1);
    let duplicate = reduce_live(
        stopped.state(),
        LiveInput::Command(LiveCommand::ReleasePushToTalk(expected)),
    );
    assert_eq!(
        duplicate.disposition(),
        EventDisposition::Ignored(RejectReason::StaleRevision)
    );
    assert_eq!(duplicate.state(), stopped.state());
    assert!(duplicate.effects().is_empty());
}

#[test]
#[allow(clippy::too_many_lines)]
fn capture_failure_best_effort_audio_is_retained_and_missing_audio_is_valid() {
    let state = started();
    let session = state.session().unwrap();
    let correlation = LiveCorrelation::new(
        session.session_id(),
        session.session_revision(),
        Phase::Capturing,
    );
    let audio_failure = reduce_live(
        &state,
        LiveInput::Event(LiveEvent::CaptureFailed {
            correlation,
            audio: Some(voice_core::RecordedAudio::new(
                AudioReferenceId::new(55).unwrap(),
                true,
            )),
        }),
    );
    let failed_session = audio_failure.state().session().unwrap();
    assert_eq!(failed_session.outcome(), Some(TerminalOutcome::Failed));
    assert!(failed_session.audio().is_some());
    assert!(
        failed_session
            .materials()
            .state(MaterialKind::RecordedAudio)
            .available()
    );
    assert!(
        !audio_failure
            .effects()
            .iter()
            .any(|effect| matches!(effect, LiveEffect::StartRecognition { .. }))
    );
    let audio_cleanup = audio_failure
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::CleanupCapture { correlation, .. } => Some(*correlation),
            _ => None,
        })
        .expect("capture failure must clean up the capture boundary");
    assert_eq!(failed_session.pending_cleanup(), Some(audio_cleanup));
    assert!(matches!(
        audio_failure.effects().first(),
        Some(LiveEffect::CleanupCapture { .. })
    ));
    let cleaned_audio = reduce_live(
        audio_failure.state(),
        LiveInput::Event(LiveEvent::CaptureCleanupCompleted {
            correlation: audio_cleanup,
            audio_cancelled: true,
            audio_discarded: true,
            cancellation_cancelled: true,
        }),
    );
    let cleaned_session = cleaned_audio.state().session().unwrap();
    assert!(cleaned_session.pending_cleanup().is_none());
    assert!(cleaned_session.audio().is_some());
    assert!(
        cleaned_session
            .materials()
            .state(MaterialKind::RecordedAudio)
            .available()
    );

    let missing_audio = reduce_live(
        &state,
        LiveInput::Event(LiveEvent::CaptureFailed {
            correlation,
            audio: None,
        }),
    );
    let missing_session = missing_audio.state().session().unwrap();
    assert_eq!(missing_session.outcome(), Some(TerminalOutcome::Failed));
    assert!(missing_session.audio().is_none());
    assert!(
        !missing_session
            .materials()
            .state(MaterialKind::RecordedAudio)
            .available()
    );
    assert!(
        !missing_audio
            .effects()
            .iter()
            .any(|effect| matches!(effect, LiveEffect::StartRecognition { .. }))
    );
    let missing_cleanup = missing_audio
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::CleanupCapture { correlation, .. } => Some(*correlation),
            _ => None,
        })
        .expect("capture failure without audio still requires cleanup");
    let cleanup_failed = reduce_live(
        missing_audio.state(),
        LiveInput::Event(LiveEvent::CaptureCleanupCompleted {
            correlation: missing_cleanup,
            audio_cancelled: false,
            audio_discarded: true,
            cancellation_cancelled: true,
        }),
    );
    let cleanup_failed_session = cleanup_failed.state().session().unwrap();
    assert_eq!(
        cleanup_failed_session.pending_cleanup(),
        Some(missing_cleanup)
    );
    assert_eq!(
        cleanup_failed_session.failure().unwrap().code(),
        FailureCode::DeviceFailure
    );
    assert!(cleanup_failed.effects().iter().any(|effect| matches!(
        effect,
        LiveEffect::RetryCaptureCleanup { correlation, .. } if *correlation == missing_cleanup
    )));

    let stopping_state = stopping(&state);
    let stopping_session = stopping_state.session().unwrap();
    let stopping_correlation = LiveCorrelation::new(
        stopping_session.session_id(),
        stopping_session.session_revision(),
        Phase::StoppingCapture,
    );
    let empty = reduce_live(
        &stopping_state,
        LiveInput::Event(LiveEvent::CaptureStoppedAt {
            correlation: stopping_correlation,
            audio: None,
            at: Timestamp::new(12),
        }),
    );
    assert_eq!(
        empty.state().session().unwrap().outcome(),
        Some(TerminalOutcome::Failed)
    );
    assert_eq!(
        empty.state().session().unwrap().failure().unwrap().code(),
        FailureCode::EmptyAudio
    );
    assert!(
        !empty
            .effects()
            .iter()
            .any(|effect| matches!(effect, LiveEffect::StartRecognition { .. }))
    );
    assert!(
        !empty
            .state()
            .session()
            .unwrap()
            .materials()
            .state(MaterialKind::RecordedAudio)
            .available()
    );
}

#[test]
fn post_capture_escape_preserves_material_and_cancels_remaining_work() {
    let recognizing_state = recognizing(&started());
    let correlation = recognition_correlation(&recognizing_state);
    let partial = reduce_live(
        &recognizing_state,
        LiveInput::Event(LiveEvent::RecognitionPartial {
            correlation,
            partial: voice_core::PartialTranscript::new("synthetic partial"),
        }),
    );
    let escaped = reduce_live(
        partial.state(),
        LiveInput::Command(LiveCommand::Escape(LiveCorrelation::new(
            partial.state().session().unwrap().session_id(),
            partial.state().session().unwrap().session_revision(),
            Phase::Recognizing,
        ))),
    );
    let session = escaped.state().session().unwrap();
    assert_eq!(session.outcome(), Some(TerminalOutcome::Cancelled));
    assert!(session.audio().is_some());
    assert!(session.partial().is_some());
    assert!(escaped.effects().iter().any(|effect| matches!(
        effect,
        LiveEffect::Cancel(token) if *token == session.cancellation_token()
    )));
    assert!(
        escaped
            .effects()
            .iter()
            .any(|effect| matches!(effect, LiveEffect::PersistRecord { .. }))
    );
}

#[test]
fn live_recognition_empty_cancellation_and_late_partial_are_correlated() {
    let state = recognizing(&started());
    let correlation = recognition_correlation(&state);
    let token = state.session().unwrap().cancellation_token();

    for event in [
        LiveEvent::RecognitionEmpty(correlation),
        LiveEvent::RecognitionCancelled(correlation),
    ] {
        let transition = reduce_live(&state, LiveInput::Event(event));
        let session = transition.state().session().unwrap();
        assert_eq!(session.outcome(), Some(TerminalOutcome::Failed));
        assert!(session.audio().is_some());
        assert!(
            session
                .materials()
                .state(MaterialKind::RecordedAudio)
                .available()
        );
        assert!(transition.effects().iter().any(|effect| matches!(
            effect,
            LiveEffect::Cancel(effect_token) if *effect_token == token
        )));
        assert!(
            transition
                .effects()
                .iter()
                .any(|effect| matches!(effect, LiveEffect::PersistRecord { .. }))
        );
    }

    let terminal = reduce_live(
        &state,
        LiveInput::Event(LiveEvent::RecognitionEmpty(correlation)),
    );
    let late = reduce_live(
        terminal.state(),
        LiveInput::Event(LiveEvent::RecognitionPartial {
            correlation,
            partial: voice_core::PartialTranscript::new("late partial"),
        }),
    );
    assert_eq!(
        late.disposition(),
        EventDisposition::Ignored(RejectReason::TerminalCallback)
    );
    assert_eq!(late.state(), terminal.state());
    assert!(late.effects().is_empty());
}

#[test]
fn partial_is_cleared_after_final_and_retained_on_timeout() {
    let state = recognizing(&started());
    let correlation = recognition_correlation(&state);
    let partial = reduce_live(
        &state,
        LiveInput::Event(LiveEvent::RecognitionPartial {
            correlation,
            partial: voice_core::PartialTranscript::new("synthetic partial"),
        }),
    );
    let final_state = reduce_live(
        partial.state(),
        LiveInput::Event(LiveEvent::RecognitionFinal {
            correlation,
            raw: RawTranscript::new("synthetic final"),
        }),
    );
    assert!(final_state.state().session().unwrap().partial().is_none());
    assert!(
        !final_state
            .state()
            .session()
            .unwrap()
            .materials()
            .state(MaterialKind::PartialTranscript)
            .available()
    );

    let failed = reduce_live(
        &partial.state().clone(),
        LiveInput::Event(LiveEvent::RecognitionTimedOut(correlation)),
    );
    let session = failed.state().session().unwrap();
    assert_eq!(session.outcome(), Some(TerminalOutcome::Failed));
    assert!(session.audio().is_some());
    assert!(
        session
            .materials()
            .state(MaterialKind::RecordedAudio)
            .available()
    );
    assert!(session.partial().is_some());
    assert!(
        session
            .warnings()
            .contains(&Warning::IncompletePartialRetained)
    );
}

#[test]
fn target_operation_is_exact_one_shot_and_cross_order_is_allowed() {
    let state = recognizing(&started());
    let target_operation = target_correlation(&state);
    let stale = TargetOperationCorrelation::new(
        target_operation.live(),
        voice_core::OperationId::new(target_operation.operation_id().get() + 1).unwrap(),
    );
    let ignored = reduce_live(
        &state,
        LiveInput::Event(LiveEvent::TargetResolvedForOperation {
            correlation: stale,
            resolution: target(),
        }),
    );
    assert!(matches!(
        ignored.disposition(),
        EventDisposition::Ignored(RejectReason::StaleRevision)
    ));
    assert_eq!(ignored.state(), &state);

    let processing = processing(&started());
    let accepted = reduce_live(
        &processing,
        LiveInput::Event(LiveEvent::TargetResolvedForOperation {
            correlation: target_operation,
            resolution: target(),
        }),
    );
    assert!(accepted.state().session().unwrap().target().is_some());
    let duplicate = reduce_live(
        accepted.state(),
        LiveInput::Event(LiveEvent::TargetResolvedForOperation {
            correlation: target_operation,
            resolution: TargetResolution::Ineligible,
        }),
    );
    assert!(matches!(
        duplicate.disposition(),
        EventDisposition::Ignored(RejectReason::StaleRevision)
    ));
    assert_eq!(
        duplicate.state().session().unwrap().target(),
        accepted.state().session().unwrap().target()
    );
}

#[test]
fn insertion_callbacks_require_exact_operation_and_uncertain_escape_preserves_text() {
    let state = recognizing(&started());
    let rec = recognition_correlation(&state);
    let with_target = reduce_live(
        &state,
        LiveInput::Event(LiveEvent::TargetResolvedForOperation {
            correlation: target_correlation(&state),
            resolution: target(),
        }),
    );
    let delivering = reduce_live(
        with_target.state(),
        LiveInput::Event(LiveEvent::RecognitionFinal {
            correlation: rec,
            raw: RawTranscript::new("synthetic raw"),
        }),
    );
    let processing_corr = LiveCorrelation::new(
        delivering.state().session().unwrap().session_id(),
        delivering.state().session().unwrap().session_revision(),
        Phase::Processing,
    );
    let delivering = reduce_live(
        delivering.state(),
        LiveInput::Event(LiveEvent::ProcessingSucceeded {
            correlation: processing_corr,
            result: ProcessingResult {
                processed_text: None,
                final_text: FinalText::new("synthetic raw"),
            },
        }),
    );
    let insertion = delivering
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::BeginInsertion { correlation, .. } => Some(*correlation),
            _ => None,
        })
        .unwrap();
    let fabricated = voice_core::DeliveryOperationCorrelation::new(
        insertion.live(),
        voice_core::OperationId::new(insertion.operation_id().get() + 1).unwrap(),
    );
    let ignored = reduce_live(
        delivering.state(),
        LiveInput::Event(LiveEvent::InsertionSucceededForOperation(fabricated)),
    );
    assert!(matches!(
        ignored.disposition(),
        EventDisposition::Ignored(RejectReason::StaleRevision)
    ));
    let marked = reduce_live(
        delivering.state(),
        LiveInput::Event(LiveEvent::InsertionStartedForOperation(insertion)),
    );
    assert!(marked.state().session().unwrap().delivery_irreversible());
    let marked_session = marked.state().session().unwrap();
    let escaped_after_marker = reduce_live(
        marked.state(),
        LiveInput::Command(LiveCommand::Escape(LiveCorrelation::new(
            marked_session.session_id(),
            marked_session.session_revision(),
            Phase::Delivering,
        ))),
    );
    assert_eq!(
        escaped_after_marker.state().session().unwrap().outcome(),
        Some(TerminalOutcome::DeliveryUncertain)
    );
    let started_uncertain = reduce_live(
        delivering.state(),
        LiveInput::Event(LiveEvent::InsertionUncertainForOperation(insertion)),
    );
    let manual = started_uncertain
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::PresentResultPanel { correlation, .. } => Some(*correlation),
            _ => None,
        })
        .unwrap();
    let escaped = reduce_live(
        started_uncertain.state(),
        LiveInput::Command(LiveCommand::Escape(manual.live())),
    );
    let session = escaped.state().session().unwrap();
    assert_eq!(session.outcome(), Some(TerminalOutcome::DeliveryUncertain));
    assert!(session.final_text().is_some());
    assert!(
        escaped
            .effects()
            .iter()
            .any(|effect| matches!(effect, LiveEffect::PresentResultPanel { .. }))
    );
}

#[test]
fn definite_insertion_failure_uses_panel_then_clipboard_and_both_fail() {
    let state = recognizing(&started());
    let rec = recognition_correlation(&state);
    let target_op = target_correlation(&state);
    let with_target = reduce_live(
        &state,
        LiveInput::Event(LiveEvent::TargetResolvedForOperation {
            correlation: target_op,
            resolution: target(),
        }),
    );
    let delivering = reduce_live(
        with_target.state(),
        LiveInput::Event(LiveEvent::RecognitionFinal {
            correlation: rec,
            raw: RawTranscript::new("synthetic raw"),
        }),
    );
    let processing_corr = LiveCorrelation::new(
        delivering.state().session().unwrap().session_id(),
        delivering.state().session().unwrap().session_revision(),
        Phase::Processing,
    );
    let delivering = reduce_live(
        delivering.state(),
        LiveInput::Event(LiveEvent::ProcessingSucceeded {
            correlation: processing_corr,
            result: ProcessingResult {
                processed_text: None,
                final_text: FinalText::new("synthetic raw"),
            },
        }),
    );
    let insertion = delivering
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::BeginInsertion { correlation, .. } => Some(*correlation),
            _ => None,
        })
        .unwrap();
    let failed = reduce_live(
        delivering.state(),
        LiveInput::Event(LiveEvent::InsertionFailedForOperation(insertion)),
    );
    let panel = failed
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::PresentResultPanel { correlation, .. } => Some(*correlation),
            _ => None,
        })
        .unwrap();
    let panel_failed = reduce_live(
        failed.state(),
        LiveInput::Event(LiveEvent::ResultPanelPresentedForOperation {
            correlation: panel,
            presented: false,
        }),
    );
    let clipboard = panel_failed
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::CopyToClipboard { correlation, .. } => Some(*correlation),
            _ => None,
        })
        .unwrap();
    let terminal = reduce_live(
        panel_failed.state(),
        LiveInput::Event(LiveEvent::ClipboardFallbackForOperation {
            correlation: clipboard,
            copied: false,
        }),
    );
    assert_eq!(
        terminal.state().session().unwrap().outcome(),
        Some(TerminalOutcome::Failed)
    );
    assert!(terminal.state().session().unwrap().audio().is_some());
    assert!(
        terminal
            .state()
            .session()
            .unwrap()
            .materials()
            .state(MaterialKind::RecordedAudio)
            .available()
    );
    assert!(terminal.state().session().unwrap().final_text().is_some());
}

#[test]
#[allow(clippy::too_many_lines)]
fn manual_panel_success_and_clipboard_success_preserve_exact_delivery_materials() {
    let (delivering, insertion) = delivering_with_insertion();
    let failed = reduce_live(
        &delivering,
        LiveInput::Event(LiveEvent::InsertionFailedForOperation(insertion)),
    );
    let panel = failed
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::PresentResultPanel { correlation, .. } => Some(*correlation),
            _ => None,
        })
        .expect("failed insertion must offer the result panel");
    let panel_success = reduce_live(
        failed.state(),
        LiveInput::Event(LiveEvent::ResultPanelPresentedForOperation {
            correlation: panel,
            presented: true,
        }),
    );
    let panel_session = panel_success.state().session().unwrap();
    assert_eq!(
        panel_session.outcome(),
        Some(TerminalOutcome::ManualDeliveryRequired)
    );
    assert!(panel_session.audio().is_some());
    assert!(
        panel_session
            .materials()
            .state(MaterialKind::RecordedAudio)
            .available()
    );
    assert!(
        panel_session
            .materials()
            .state(MaterialKind::ResultPanel)
            .available()
    );
    assert!(
        !panel_session
            .materials()
            .state(MaterialKind::ClipboardFallback)
            .available()
    );

    let failed_again = reduce_live(
        &delivering,
        LiveInput::Event(LiveEvent::InsertionFailedForOperation(insertion)),
    );
    let panel_again = failed_again
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::PresentResultPanel { correlation, .. } => Some(*correlation),
            _ => None,
        })
        .expect("failed insertion must offer the result panel");
    let clipboard_pending = reduce_live(
        failed_again.state(),
        LiveInput::Event(LiveEvent::ResultPanelPresentedForOperation {
            correlation: panel_again,
            presented: false,
        }),
    );
    let clipboard = clipboard_pending
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::CopyToClipboard { correlation, .. } => Some(*correlation),
            _ => None,
        })
        .expect("panel failure must offer clipboard fallback");
    let clipboard_success = reduce_live(
        clipboard_pending.state(),
        LiveInput::Event(LiveEvent::ClipboardFallbackForOperation {
            correlation: clipboard,
            copied: true,
        }),
    );
    let clipboard_session = clipboard_success.state().session().unwrap();
    assert_eq!(
        clipboard_session.outcome(),
        Some(TerminalOutcome::ManualDeliveryRequired)
    );
    assert!(clipboard_session.audio().is_some());
    assert!(
        clipboard_session
            .materials()
            .state(MaterialKind::RecordedAudio)
            .available()
    );
    assert!(
        clipboard_session
            .materials()
            .state(MaterialKind::ClipboardFallback)
            .available()
    );
    assert!(
        !clipboard_session
            .materials()
            .state(MaterialKind::ResultPanel)
            .available()
    );
}

#[test]
fn late_ineligible_target_resolution_starts_manual_preservation() {
    let recognizing_state = recognizing(&started());
    let target_operation = target_correlation(&recognizing_state);
    let rec = recognition_correlation(&recognizing_state);
    let processing_state = reduce_live(
        &recognizing_state,
        LiveInput::Event(LiveEvent::RecognitionFinal {
            correlation: rec,
            raw: RawTranscript::new("synthetic raw"),
        }),
    );
    let processing_session = processing_state.state().session().unwrap();
    let processing = reduce_live(
        processing_state.state(),
        LiveInput::Event(LiveEvent::ProcessingSucceeded {
            correlation: LiveCorrelation::new(
                processing_session.session_id(),
                processing_session.session_revision(),
                Phase::Processing,
            ),
            result: ProcessingResult {
                processed_text: None,
                final_text: FinalText::new("synthetic raw"),
            },
        }),
    );
    assert_eq!(processing.state().phase(), Phase::Delivering);
    assert!(processing.effects().is_empty());
    let resolved = reduce_live(
        processing.state(),
        LiveInput::Event(LiveEvent::TargetResolvedForOperation {
            correlation: target_operation,
            resolution: TargetResolution::Ineligible,
        }),
    );
    assert!(
        resolved
            .effects()
            .iter()
            .any(|effect| matches!(effect, LiveEffect::PresentResultPanel { .. }))
    );
}

#[test]
fn persistence_is_exact_and_recovery_owns_full_payload() {
    let state = recognizing(&started());
    let rec = recognition_correlation(&state);
    let failed = reduce_live(
        &state,
        LiveInput::Event(LiveEvent::RecognitionFailed {
            correlation: rec,
            code: FailureCode::RecognitionProvider,
        }),
    );
    let persist = failed
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::PersistRecord {
                correlation,
                operation_id,
                record,
                ..
            } => Some((*correlation, *operation_id, record.clone())),
            _ => None,
        })
        .unwrap();
    assert_eq!(persist.2.outcome(), Some(TerminalOutcome::Failed));
    let expected_recovery_id = failed.state().session().unwrap().recovery_id();
    let mismatch = reduce_live(
        failed.state(),
        LiveInput::Event(LiveEvent::PersistenceFailedForOperation {
            correlation: persist.0,
            operation_id: persist.1,
            recovery_id: voice_core::RecoveryId::new(expected_recovery_id.get() + 1).unwrap(),
        }),
    );
    assert_eq!(
        mismatch.disposition(),
        EventDisposition::Ignored(RejectReason::StaleRecovery)
    );
    assert_eq!(mismatch.state(), failed.state());
    assert!(mismatch.effects().is_empty());
    let recovery = reduce_live(
        failed.state(),
        LiveInput::Event(LiveEvent::PersistenceFailedForOperation {
            correlation: persist.0,
            operation_id: persist.1,
            recovery_id: expected_recovery_id,
        }),
    );
    let context = recovery.state().session().unwrap().recovery().unwrap();
    assert_eq!(context.record().outcome(), Some(TerminalOutcome::Failed));
    assert_eq!(context.record().warnings(), context.record().warnings());
    let stale = reduce_live(
        recovery.state(),
        LiveInput::Event(LiveEvent::PersistenceSucceededForOperation {
            correlation: persist.0,
            operation_id: persist.1,
            recovery_id: expected_recovery_id,
            report: PersistenceReport {
                durable_materials: context.materials().available_kinds(),
            },
        }),
    );
    assert!(matches!(
        stale.disposition(),
        EventDisposition::Ignored(RejectReason::UnexpectedPhase | RejectReason::TerminalCallback)
    ));
}

fn durable_record() -> DictationRecord {
    let mut record = DictationRecord::new(
        DictationRecordId::new(20).unwrap(),
        SessionId::new(21).unwrap(),
    );
    record.set_recorded_audio(voice_core::RecordedAudio::new(
        AudioReferenceId::new(22).unwrap(),
        true,
    ));
    record.set_outcome(TerminalOutcome::Failed);
    let mut materials = MaterialLedger::new();
    materials.set(
        MaterialKind::RecordedAudio,
        MaterialState::Available(Durability::Durable),
    );
    record.set_materials(materials);
    record.mark_durable();
    record
}

#[test]
fn retry_has_exact_subphases_and_preserves_original_on_result_persist_failure() {
    let state = RetryState::new(durable_record());
    let began = reduce_retry(
        &state,
        RetryInput::Command(RetryCommand::Begin {
            attempt_id: RecognitionAttemptId::new(23).unwrap(),
            configuration_id: ConfigurationId::new(24).unwrap(),
            timeout: DurationLimit::from_seconds(2).unwrap(),
            cancellation_token: CancellationTokenId::new(25).unwrap(),
            recovery_id: voice_core::RecoveryId::new(26).unwrap(),
            started_at: Timestamp::new(30),
        }),
    );
    let pending = began.state().active().unwrap();
    assert_eq!(
        pending.expected_retry_phase(),
        RetryPhase::PendingAttemptPersistence
    );
    assert!(
        began
            .effects()
            .iter()
            .any(|effect| matches!(effect, voice_core::RetryEffect::PersistAttempt { .. }))
    );
    let duplicate = reduce_retry(
        began.state(),
        RetryInput::Event(RetryEvent::RecognitionFinal {
            correlation: pending,
            raw: RawTranscript::new("too early"),
        }),
    );
    assert!(matches!(
        duplicate.disposition(),
        EventDisposition::Ignored(RejectReason::UnexpectedPhase)
    ));
    let recognizing = reduce_retry(
        began.state(),
        RetryInput::Event(RetryEvent::AttemptPersistenceSucceeded(pending)),
    );
    let active = recognizing.state().active().unwrap();
    assert_eq!(active.expected_retry_phase(), RetryPhase::Recognizing);
    let result = reduce_retry(
        recognizing.state(),
        RetryInput::Event(RetryEvent::RecognitionFinal {
            correlation: active,
            raw: RawTranscript::new("retry result"),
        }),
    );
    let result_corr = result.state().active().unwrap();
    assert_eq!(
        result_corr.expected_retry_phase(),
        RetryPhase::PendingResultPersistence
    );
    let failed = reduce_retry(
        result.state(),
        RetryInput::Event(RetryEvent::ResultPersistenceFailed(result_corr)),
    );
    assert!(failed.state().active().is_none());
    assert_eq!(failed.state().record().attempts().len(), 1);
    assert!(failed.state().record().is_durable());
    let pending_record = failed.state().pending_record().unwrap();
    assert!(!pending_record.is_durable());
    assert_eq!(pending_record.attempts().len(), 1);

    let persisted = reduce_retry(
        result.state(),
        RetryInput::Event(RetryEvent::ResultPersistenceSucceeded(result_corr)),
    );
    assert!(persisted.state().active().is_none());
    assert_eq!(persisted.state().record().attempts().len(), 1);
    assert!(persisted.state().record().is_durable());
}

#[test]
fn retry_attempt_persistence_failure_discards_only_pending_attempt() {
    let original = durable_record();
    let state = RetryState::new(original.clone());
    let began = reduce_retry(
        &state,
        RetryInput::Command(RetryCommand::Begin {
            attempt_id: RecognitionAttemptId::new(27).unwrap(),
            configuration_id: ConfigurationId::new(28).unwrap(),
            timeout: DurationLimit::from_seconds(2).unwrap(),
            cancellation_token: CancellationTokenId::new(29).unwrap(),
            recovery_id: voice_core::RecoveryId::new(30).unwrap(),
            started_at: Timestamp::new(30),
        }),
    );
    let pending = began.state().active().unwrap();
    assert_eq!(
        pending.expected_retry_phase(),
        RetryPhase::PendingAttemptPersistence
    );
    let failed = reduce_retry(
        began.state(),
        RetryInput::Event(RetryEvent::AttemptPersistenceFailed(pending)),
    );
    assert_eq!(failed.disposition(), EventDisposition::Applied);
    assert!(failed.state().active().is_none());
    assert!(failed.state().pending_record().is_none());
    assert_eq!(failed.state().record(), &original);
    assert!(failed.effects().is_empty());

    let late = reduce_retry(
        failed.state(),
        RetryInput::Event(RetryEvent::RecognitionFinal {
            correlation: pending,
            raw: RawTranscript::new("late retry"),
        }),
    );
    assert!(matches!(
        late.disposition(),
        EventDisposition::Ignored(RejectReason::NoActiveSession)
    ));
    assert_eq!(late.state(), failed.state());
    assert!(late.effects().is_empty());
}

#[test]
fn retry_rejects_every_mismatched_correlation_tuple_without_mutation_or_effects() {
    let state = RetryState::new(durable_record());
    let began = reduce_retry(
        &state,
        RetryInput::Command(RetryCommand::Begin {
            attempt_id: RecognitionAttemptId::new(127).unwrap(),
            configuration_id: ConfigurationId::new(128).unwrap(),
            timeout: DurationLimit::from_seconds(2).unwrap(),
            cancellation_token: CancellationTokenId::new(129).unwrap(),
            recovery_id: voice_core::RecoveryId::new(130).unwrap(),
            started_at: Timestamp::new(30),
        }),
    );
    let active = began.state().active().unwrap();
    let mismatches = [
        (
            voice_core::RetryCorrelation::new_with_retry_phase(
                DictationRecordId::new(active.record_id().get() + 1).unwrap(),
                active.originating_session_id(),
                active.attempt_id(),
                active.attempt_revision(),
                active.expected_phase(),
                active.expected_retry_phase(),
            ),
            RejectReason::StaleSessionId,
        ),
        (
            voice_core::RetryCorrelation::new_with_retry_phase(
                active.record_id(),
                SessionId::new(active.originating_session_id().get() + 1).unwrap(),
                active.attempt_id(),
                active.attempt_revision(),
                active.expected_phase(),
                active.expected_retry_phase(),
            ),
            RejectReason::StaleSessionId,
        ),
        (
            voice_core::RetryCorrelation::new_with_retry_phase(
                active.record_id(),
                active.originating_session_id(),
                RecognitionAttemptId::new(active.attempt_id().get() + 1).unwrap(),
                active.attempt_revision(),
                active.expected_phase(),
                active.expected_retry_phase(),
            ),
            RejectReason::StaleAttempt,
        ),
        (
            voice_core::RetryCorrelation::new_with_retry_phase(
                active.record_id(),
                active.originating_session_id(),
                active.attempt_id(),
                active.attempt_revision().next().unwrap(),
                active.expected_phase(),
                active.expected_retry_phase(),
            ),
            RejectReason::StaleAttempt,
        ),
        (
            voice_core::RetryCorrelation::new_with_retry_phase(
                active.record_id(),
                active.originating_session_id(),
                active.attempt_id(),
                active.attempt_revision(),
                Phase::Processing,
                active.expected_retry_phase(),
            ),
            RejectReason::StaleAttempt,
        ),
    ];
    for (correlation, reason) in mismatches {
        let transition = reduce_retry(
            began.state(),
            RetryInput::Event(RetryEvent::AttemptPersistenceSucceeded(correlation)),
        );
        assert_eq!(transition.disposition(), EventDisposition::Ignored(reason));
        assert_eq!(transition.state(), began.state());
        assert!(transition.effects().is_empty());
    }
}

#[test]
fn retry_incompatible_phase_events_are_unchanged_and_effect_free() {
    let state = RetryState::new(durable_record());
    let began = reduce_retry(
        &state,
        RetryInput::Command(RetryCommand::Begin {
            attempt_id: RecognitionAttemptId::new(131).unwrap(),
            configuration_id: ConfigurationId::new(132).unwrap(),
            timeout: DurationLimit::from_seconds(2).unwrap(),
            cancellation_token: CancellationTokenId::new(133).unwrap(),
            recovery_id: voice_core::RecoveryId::new(134).unwrap(),
            started_at: Timestamp::new(30),
        }),
    );
    let pending = began.state().active().unwrap();
    let pending_events = vec![
        RetryEvent::RecognitionPartial {
            correlation: pending,
            partial: voice_core::PartialTranscript::new("incompatible"),
        },
        RetryEvent::RecognitionFinal {
            correlation: pending,
            raw: RawTranscript::new("incompatible"),
        },
        RetryEvent::RecognitionEmpty(pending),
        RetryEvent::RecognitionFailed(pending),
        RetryEvent::RecognitionTimedOut(pending),
        RetryEvent::RecognitionCancelled(pending),
        RetryEvent::ResultPersistenceSucceeded(pending),
        RetryEvent::ResultPersistenceFailed(pending),
    ];
    for event in pending_events {
        let transition = reduce_retry(began.state(), RetryInput::Event(event));
        assert_eq!(
            transition.disposition(),
            EventDisposition::Ignored(RejectReason::UnexpectedPhase)
        );
        assert_eq!(transition.state(), began.state());
        assert!(transition.effects().is_empty());
    }

    let recognizing = reduce_retry(
        began.state(),
        RetryInput::Event(RetryEvent::AttemptPersistenceSucceeded(pending)),
    );
    let active = recognizing.state().active().unwrap();
    for event in [
        RetryEvent::AttemptPersistenceSucceeded(active),
        RetryEvent::AttemptPersistenceFailed(active),
        RetryEvent::ResultPersistenceSucceeded(active),
        RetryEvent::ResultPersistenceFailed(active),
    ] {
        let transition = reduce_retry(recognizing.state(), RetryInput::Event(event));
        assert_eq!(
            transition.disposition(),
            EventDisposition::Ignored(RejectReason::UnexpectedPhase)
        );
        assert_eq!(transition.state(), recognizing.state());
        assert!(transition.effects().is_empty());
    }

    let result = reduce_retry(
        recognizing.state(),
        RetryInput::Event(RetryEvent::RecognitionFinal {
            correlation: active,
            raw: RawTranscript::new("retry result"),
        }),
    );
    let pending_result = result.state().active().unwrap();
    for event in [
        RetryEvent::AttemptPersistenceSucceeded(pending_result),
        RetryEvent::AttemptPersistenceFailed(pending_result),
        RetryEvent::RecognitionPartial {
            correlation: pending_result,
            partial: voice_core::PartialTranscript::new("incompatible"),
        },
        RetryEvent::RecognitionFinal {
            correlation: pending_result,
            raw: RawTranscript::new("incompatible"),
        },
        RetryEvent::RecognitionEmpty(pending_result),
        RetryEvent::RecognitionFailed(pending_result),
        RetryEvent::RecognitionTimedOut(pending_result),
        RetryEvent::RecognitionCancelled(pending_result),
    ] {
        let transition = reduce_retry(result.state(), RetryInput::Event(event));
        assert_eq!(
            transition.disposition(),
            EventDisposition::Ignored(RejectReason::UnexpectedPhase)
        );
        assert_eq!(transition.state(), result.state());
        assert!(transition.effects().is_empty());
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn recovery_payload_refreshes_after_manual_preservation() {
    let recognizing_state = recognizing(&started());
    let rec = recognition_correlation(&recognizing_state);
    let processing = reduce_live(
        &recognizing_state,
        LiveInput::Event(LiveEvent::RecognitionFinal {
            correlation: rec,
            raw: RawTranscript::new("synthetic raw"),
        }),
    );
    let processing_session = processing.state().session().unwrap();
    let delivering = reduce_live(
        processing.state(),
        LiveInput::Event(LiveEvent::ProcessingSucceeded {
            correlation: LiveCorrelation::new(
                processing_session.session_id(),
                processing_session.session_revision(),
                Phase::Processing,
            ),
            result: ProcessingResult {
                processed_text: None,
                final_text: FinalText::new("synthetic raw"),
            },
        }),
    );
    let delivering_session = delivering.state().session().unwrap();
    let cancelled = reduce_live(
        delivering.state(),
        LiveInput::Command(LiveCommand::Escape(LiveCorrelation::new(
            delivering_session.session_id(),
            delivering_session.session_revision(),
            Phase::Delivering,
        ))),
    );
    let persist = cancelled
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::PersistRecord {
                correlation,
                operation_id,
                ..
            } => Some((*correlation, *operation_id)),
            _ => None,
        })
        .unwrap();
    let recovery = reduce_live(
        cancelled.state(),
        LiveInput::Event(LiveEvent::PersistenceFailedForOperation {
            correlation: persist.0,
            operation_id: persist.1,
            recovery_id: cancelled.state().session().unwrap().recovery_id(),
        }),
    );
    let panel = recovery
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::PresentResultPanel { correlation, .. } => Some(*correlation),
            _ => None,
        })
        .unwrap();
    let panel_failed = reduce_live(
        recovery.state(),
        LiveInput::Event(LiveEvent::ResultPanelPresentedForOperation {
            correlation: panel,
            presented: false,
        }),
    );
    let clipboard = panel_failed
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::CopyToClipboard { correlation, .. } => Some(*correlation),
            _ => None,
        })
        .expect("failed panel must offer clipboard fallback");
    let preservation_failed = reduce_live(
        panel_failed.state(),
        LiveInput::Event(LiveEvent::ClipboardFallbackForOperation {
            correlation: clipboard,
            copied: false,
        }),
    );
    let failed_session = preservation_failed.state().session().unwrap();
    assert_eq!(failed_session.outcome(), Some(TerminalOutcome::Cancelled));
    assert_eq!(
        failed_session.failure().unwrap().code(),
        FailureCode::ManualPreservationFailed
    );
    assert_eq!(
        failed_session.recovery().unwrap().record().outcome(),
        Some(TerminalOutcome::Cancelled)
    );
    let preserved = reduce_live(
        recovery.state(),
        LiveInput::Event(LiveEvent::ResultPanelPresentedForOperation {
            correlation: panel,
            presented: true,
        }),
    );
    let context = preserved.state().session().unwrap().recovery().unwrap();
    assert!(
        context
            .record()
            .materials()
            .state(MaterialKind::ResultPanel)
            .available()
    );
    let recovery = context.id();
    let closed = reduce_live(
        preserved.state(),
        LiveInput::Event(LiveEvent::RecoveryPersistenceSucceeded {
            recovery: voice_core::RecoveryCorrelation::new(
                recovery,
                context.record_id(),
                context.session_id(),
            ),
            report: PersistenceReport {
                durable_materials: context.materials().available_kinds(),
            },
        }),
    );
    let late = reduce_live(
        closed.state(),
        LiveInput::Event(LiveEvent::ResultPanelPresentedForOperation {
            correlation: panel,
            presented: true,
        }),
    );
    assert_eq!(
        late.disposition(),
        EventDisposition::Ignored(RejectReason::StaleRecovery)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn failure_metadata_survives_target_fallback_and_processing_fallback_delivery() {
    let recognizing_state = recognizing(&started());
    let target_operation = target_correlation(&recognizing_state);
    let recognition = recognition_correlation(&recognizing_state);
    let processing_state = reduce_live(
        &recognizing_state,
        LiveInput::Event(LiveEvent::RecognitionFinal {
            correlation: recognition,
            raw: RawTranscript::new("synthetic raw"),
        }),
    );
    let ineligible = reduce_live(
        processing_state.state(),
        LiveInput::Event(LiveEvent::TargetResolvedForOperation {
            correlation: target_operation,
            resolution: TargetResolution::Ineligible,
        }),
    );
    let ineligible_session = ineligible.state().session().unwrap();
    assert_eq!(
        ineligible_session.failure().unwrap().code(),
        FailureCode::TargetUnavailable
    );

    let processing = reduce_live(
        ineligible.state(),
        LiveInput::Event(LiveEvent::ProcessingFailed {
            correlation: LiveCorrelation::new(
                ineligible_session.session_id(),
                ineligible_session.session_revision(),
                Phase::Processing,
            ),
            code: FailureCode::ProcessingStep,
        }),
    );
    let manual = processing
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::PresentResultPanel { correlation, .. } => Some(*correlation),
            _ => None,
        })
        .unwrap();
    let terminal = reduce_live(
        processing.state(),
        LiveInput::Event(LiveEvent::ResultPanelPresentedForOperation {
            correlation: manual,
            presented: true,
        }),
    );
    let session = terminal.state().session().unwrap();
    assert_eq!(
        session.failure().unwrap().code(),
        FailureCode::TargetUnavailable
    );

    let with_target = reduce_live(
        processing_state.state(),
        LiveInput::Event(LiveEvent::TargetResolvedForOperation {
            correlation: target_operation,
            resolution: target(),
        }),
    );
    let processing_session = with_target.state().session().unwrap();
    let fallback = reduce_live(
        with_target.state(),
        LiveInput::Event(LiveEvent::ProcessingFailed {
            correlation: LiveCorrelation::new(
                processing_session.session_id(),
                processing_session.session_revision(),
                Phase::Processing,
            ),
            code: FailureCode::ProcessingStep,
        }),
    );
    let insertion = fallback
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::BeginInsertion { correlation, .. } => Some(*correlation),
            _ => None,
        })
        .unwrap();
    let delivered = reduce_live(
        fallback.state(),
        LiveInput::Event(LiveEvent::InsertionSucceededForOperation(insertion)),
    );
    let session = delivered.state().session().unwrap();
    assert_eq!(
        session.outcome(),
        Some(TerminalOutcome::DeliveredAutomatically)
    );
    assert_eq!(
        session.failure().unwrap().code(),
        FailureCode::ProcessingStep
    );
    assert!(session.audio().is_some());
    assert!(
        session
            .materials()
            .state(MaterialKind::RecordedAudio)
            .available()
    );
    assert!(session.warnings().contains(&Warning::ProcessingFallback));
}

#[test]
fn processing_timeout_falls_back_to_raw_and_cancels_remaining_work_first() {
    let recognizing_state = recognizing(&started());
    let target_operation = target_correlation(&recognizing_state);
    let processing_state = reduce_live(
        &recognizing_state,
        LiveInput::Event(LiveEvent::RecognitionFinal {
            correlation: recognition_correlation(&recognizing_state),
            raw: RawTranscript::new("synthetic raw"),
        }),
    );
    let with_target = reduce_live(
        processing_state.state(),
        LiveInput::Event(LiveEvent::TargetResolvedForOperation {
            correlation: target_operation,
            resolution: target(),
        }),
    );
    let processing_session = with_target.state().session().unwrap();
    let fallback = reduce_live(
        with_target.state(),
        LiveInput::Event(LiveEvent::ProcessingFailed {
            correlation: LiveCorrelation::new(
                processing_session.session_id(),
                processing_session.session_revision(),
                Phase::Processing,
            ),
            code: FailureCode::ProcessingTimeout,
        }),
    );
    assert!(matches!(
        fallback.effects().first(),
        Some(LiveEffect::Cancel(token)) if *token == processing_session.cancellation_token()
    ));
    assert!(
        fallback
            .effects()
            .iter()
            .any(|effect| matches!(effect, LiveEffect::BeginInsertion { .. }))
    );
    let session = fallback.state().session().unwrap();
    assert_eq!(
        session.raw().map(RawTranscript::as_str),
        Some("synthetic raw")
    );
    assert_eq!(
        session.final_text().map(FinalText::as_str),
        Some("synthetic raw")
    );
    assert!(session.processed().is_none());
    assert!(session.warnings().contains(&Warning::ProcessingFallback));
    assert_eq!(
        session.failure().unwrap().code(),
        FailureCode::ProcessingTimeout
    );
    assert!(session.audio().is_some());
    assert!(
        session
            .materials()
            .state(MaterialKind::RecordedAudio)
            .available()
    );
}

#[test]
fn disabled_or_unavailable_llm_is_skipped_and_later_local_step_runs() {
    let plan = ProcessingPlan::new(
        RawTranscript::new("synthetic raw"),
        vec![
            ProcessingStep::LanguageModel {
                configuration_id: None,
                enabled: true,
            },
            ProcessingStep::BuiltIn {
                rule_id: ConfigurationId::new(31).unwrap(),
                enabled: true,
            },
        ],
    )
    .unwrap();
    assert!(!plan.steps()[0].is_enabled());
    assert!(plan.steps()[1].is_enabled());
}

#[test]
fn processing_success_advances_revision_and_rejects_late_failure_without_mutation() {
    let state = processing(&started());
    let session = state.session().unwrap();
    let successful = reduce_live(
        &state,
        LiveInput::Event(LiveEvent::ProcessingSucceeded {
            correlation: LiveCorrelation::new(
                session.session_id(),
                session.session_revision(),
                Phase::Processing,
            ),
            result: ProcessingResult {
                processed_text: Some(voice_core::ProcessedText::new("synthetic processed")),
                final_text: FinalText::new("synthetic processed"),
            },
        }),
    );
    let processed = successful.state().session().unwrap();
    assert_eq!(
        processed.processed().map(voice_core::ProcessedText::as_str),
        Some("synthetic processed")
    );
    assert_eq!(
        processed.final_text().map(FinalText::as_str),
        Some("synthetic processed")
    );
    // A later callback for the already advanced processing operation is stale;
    // it cannot replace the committed processing result.
    let late = reduce_live(
        successful.state(),
        LiveInput::Event(LiveEvent::ProcessingFailed {
            correlation: LiveCorrelation::new(
                processed.session_id(),
                processed.session_revision(),
                Phase::Processing,
            ),
            code: FailureCode::ProcessingStep,
        }),
    );
    assert_eq!(late.state(), successful.state());
    assert!(late.effects().is_empty());
}

#[test]
#[allow(clippy::too_many_lines)]
fn persistence_callbacks_are_one_shot_and_recovery_failure_stays_retryable() {
    let state = recognizing(&started());
    let failed = reduce_live(
        &state,
        LiveInput::Event(LiveEvent::RecognitionFailed {
            correlation: recognition_correlation(&state),
            code: FailureCode::RecognitionProvider,
        }),
    );
    let (correlation, operation_id) = failed
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::PersistRecord {
                correlation,
                operation_id,
                ..
            } => Some((*correlation, *operation_id)),
            _ => None,
        })
        .unwrap();
    let persistence_failed = reduce_live(
        failed.state(),
        LiveInput::Event(LiveEvent::PersistenceFailedForOperation {
            correlation,
            operation_id,
            recovery_id: failed.state().session().unwrap().recovery_id(),
        }),
    );
    let recovery = persistence_failed
        .state()
        .session()
        .unwrap()
        .recovery()
        .unwrap();
    assert!(recovery.record().recorded_audio().is_some());
    assert!(
        recovery
            .record()
            .materials()
            .state(MaterialKind::RecordedAudio)
            .available()
    );
    let retryable = reduce_live(
        persistence_failed.state(),
        LiveInput::Event(LiveEvent::RecoveryPersistenceFailed(
            voice_core::RecoveryCorrelation::new(
                recovery.id(),
                recovery.record_id(),
                recovery.session_id(),
            ),
        )),
    );
    assert_eq!(retryable.disposition(), EventDisposition::Applied);
    assert!(
        !retryable
            .state()
            .session()
            .unwrap()
            .recovery()
            .unwrap()
            .is_closed()
    );

    let duplicate = reduce_live(
        persistence_failed.state(),
        LiveInput::Event(LiveEvent::PersistenceFailedForOperation {
            correlation,
            operation_id,
            recovery_id: persistence_failed.state().session().unwrap().recovery_id(),
        }),
    );
    assert_eq!(
        duplicate.disposition(),
        EventDisposition::Ignored(RejectReason::TerminalCallback)
    );

    let closed = reduce_live(
        persistence_failed.state(),
        LiveInput::Event(LiveEvent::RecoveryPersistenceSucceeded {
            recovery: voice_core::RecoveryCorrelation::new(
                recovery.id(),
                recovery.record_id(),
                recovery.session_id(),
            ),
            report: PersistenceReport {
                durable_materials: recovery.materials().available_kinds(),
            },
        }),
    );
    let late = reduce_live(
        closed.state(),
        LiveInput::Event(LiveEvent::RecoveryPersistenceFailed(
            voice_core::RecoveryCorrelation::new(
                recovery.id(),
                recovery.record_id(),
                recovery.session_id(),
            ),
        )),
    );
    assert_eq!(
        late.disposition(),
        EventDisposition::Ignored(RejectReason::StaleRecovery)
    );
}

#[test]
fn retry_result_persistence_failure_archives_the_non_durable_pending_record() {
    let state = RetryState::new(durable_record());
    let began = reduce_retry(
        &state,
        RetryInput::Command(RetryCommand::Begin {
            attempt_id: RecognitionAttemptId::new(101).unwrap(),
            configuration_id: ConfigurationId::new(102).unwrap(),
            timeout: DurationLimit::from_seconds(2).unwrap(),
            cancellation_token: CancellationTokenId::new(103).unwrap(),
            recovery_id: voice_core::RecoveryId::new(104).unwrap(),
            started_at: Timestamp::new(30),
        }),
    );
    let pending = began.state().active().unwrap();
    let recognizing = reduce_retry(
        began.state(),
        RetryInput::Event(RetryEvent::AttemptPersistenceSucceeded(pending)),
    );
    let active = recognizing.state().active().unwrap();
    let result = reduce_retry(
        recognizing.state(),
        RetryInput::Event(RetryEvent::RecognitionFinal {
            correlation: active,
            raw: RawTranscript::new("retry result"),
        }),
    );
    let result_corr = result.state().active().unwrap();
    let failed = reduce_retry(
        result.state(),
        RetryInput::Event(RetryEvent::ResultPersistenceFailed(result_corr)),
    );
    assert!(failed.state().active().is_none());
    assert_eq!(failed.state().record().attempts().len(), 1);
    assert!(failed.state().record().is_durable());
    assert_eq!(failed.effects().len(), 1);
    assert!(matches!(
        &failed.effects()[0],
        voice_core::RetryEffect::ArchiveRecovery { record, .. } if record.attempts().len() == 1
    ));
    let restarted = reduce_retry(
        failed.state(),
        RetryInput::Command(RetryCommand::Begin {
            attempt_id: RecognitionAttemptId::new(105).unwrap(),
            configuration_id: ConfigurationId::new(106).unwrap(),
            timeout: DurationLimit::from_seconds(2).unwrap(),
            cancellation_token: CancellationTokenId::new(107).unwrap(),
            recovery_id: voice_core::RecoveryId::new(108).unwrap(),
            started_at: Timestamp::new(40),
        }),
    );
    assert_eq!(
        restarted.state().active().unwrap().attempt_revision().get(),
        result_corr.attempt_revision().get() + 1
    );
    assert!(matches!(
        restarted.effects().first(),
        Some(voice_core::RetryEffect::PersistAttempt { record, attempt, .. })
            if record.attempts().len() == 2
                && attempt.revision() == restarted.state().active().unwrap().attempt_revision()
    ));
}

#[test]
fn retry_timeout_cancellation_provider_failure_and_stale_callbacks_are_correlated() {
    let state = RetryState::new(durable_record());
    let began = reduce_retry(
        &state,
        RetryInput::Command(RetryCommand::Begin {
            attempt_id: RecognitionAttemptId::new(111).unwrap(),
            configuration_id: ConfigurationId::new(112).unwrap(),
            timeout: DurationLimit::from_seconds(2).unwrap(),
            cancellation_token: CancellationTokenId::new(113).unwrap(),
            recovery_id: voice_core::RecoveryId::new(114).unwrap(),
            started_at: Timestamp::new(30),
        }),
    );
    let pending = began.state().active().unwrap();
    let recognizing = reduce_retry(
        began.state(),
        RetryInput::Event(RetryEvent::AttemptPersistenceSucceeded(pending)),
    );
    let active = recognizing.state().active().unwrap();
    let stale = voice_core::RetryCorrelation::new_with_retry_phase(
        active.record_id(),
        active.originating_session_id(),
        active.attempt_id(),
        active.attempt_revision().next().unwrap(),
        Phase::Recognizing,
        RetryPhase::Recognizing,
    );
    let stale_transition = reduce_retry(
        recognizing.state(),
        RetryInput::Event(RetryEvent::RecognitionFinal {
            correlation: stale,
            raw: RawTranscript::new("stale"),
        }),
    );
    assert_eq!(
        stale_transition.disposition(),
        EventDisposition::Ignored(RejectReason::StaleAttempt)
    );

    let timed_out = reduce_retry(
        recognizing.state(),
        RetryInput::Event(RetryEvent::RecognitionTimedOut(active)),
    );
    assert!(timed_out.effects().iter().any(|effect| matches!(
        effect,
        voice_core::RetryEffect::Cancel(token) if *token == CancellationTokenId::new(113).unwrap()
    )));
    let result_corr = timed_out.state().active().unwrap();
    assert_eq!(
        timed_out
            .state()
            .pending_attempt()
            .unwrap()
            .failure()
            .unwrap()
            .code(),
        FailureCode::RetryTimeout
    );

    let cancelled = reduce_retry(
        recognizing.state(),
        RetryInput::Event(RetryEvent::RecognitionCancelled(active)),
    );
    assert_eq!(
        cancelled
            .state()
            .pending_attempt()
            .unwrap()
            .failure()
            .unwrap()
            .code(),
        FailureCode::RetryCancelled
    );

    let provider_failed = reduce_retry(
        recognizing.state(),
        RetryInput::Event(RetryEvent::RecognitionFailed(active)),
    );
    assert_eq!(
        provider_failed
            .state()
            .pending_attempt()
            .unwrap()
            .failure()
            .unwrap()
            .code(),
        FailureCode::RetryProvider
    );
    assert_eq!(
        reduce_retry(
            timed_out.state(),
            RetryInput::Event(RetryEvent::RecognitionFinal {
                correlation: result_corr,
                raw: RawTranscript::new("late"),
            }),
        )
        .disposition(),
        EventDisposition::Ignored(RejectReason::UnexpectedPhase)
    );
}

#[test]
fn retry_ineligible_record_emits_no_effect_and_releases_work() {
    let record = DictationRecord::new(
        DictationRecordId::new(115).unwrap(),
        SessionId::new(116).unwrap(),
    );
    let state = RetryState::new(record.clone());
    let transition = reduce_retry(
        &state,
        RetryInput::Command(RetryCommand::Begin {
            attempt_id: RecognitionAttemptId::new(117).unwrap(),
            configuration_id: ConfigurationId::new(118).unwrap(),
            timeout: DurationLimit::from_seconds(2).unwrap(),
            cancellation_token: CancellationTokenId::new(119).unwrap(),
            recovery_id: voice_core::RecoveryId::new(120).unwrap(),
            started_at: Timestamp::new(30),
        }),
    );
    assert_eq!(
        transition.disposition(),
        EventDisposition::Ignored(RejectReason::CompetingWork)
    );
    assert_eq!(transition.state(), &state);
    assert!(transition.effects().is_empty());
}

#[test]
fn capture_cleanup_is_fail_closed_and_duplicate_callback_is_stale() {
    let state = started();
    let session = state.session().unwrap();
    let escaped = reduce_live(
        &state,
        LiveInput::Command(LiveCommand::Escape(LiveCorrelation::new(
            session.session_id(),
            session.session_revision(),
            Phase::Capturing,
        ))),
    );
    let cleanup = escaped
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::CleanupCapture { correlation, .. } => Some(*correlation),
            _ => None,
        })
        .unwrap();
    let failed = reduce_live(
        escaped.state(),
        LiveInput::Event(LiveEvent::CaptureCleanupCompleted {
            correlation: cleanup,
            audio_cancelled: true,
            audio_discarded: false,
            cancellation_cancelled: true,
        }),
    );
    let session = failed.state().session().unwrap();
    assert_eq!(session.outcome(), Some(TerminalOutcome::Cancelled));
    assert!(session.pending_cleanup().is_some());
    assert!(
        failed
            .effects()
            .iter()
            .any(|effect| matches!(effect, LiveEffect::RetryCaptureCleanup { .. }))
    );
    assert_eq!(
        session.failure().unwrap().code(),
        FailureCode::CaptureCleanupFailed
    );
    let repeated_failure = reduce_live(
        failed.state(),
        LiveInput::Event(LiveEvent::CaptureCleanupCompleted {
            correlation: cleanup,
            audio_cancelled: true,
            audio_discarded: false,
            cancellation_cancelled: true,
        }),
    );
    assert_eq!(repeated_failure.disposition(), EventDisposition::Applied);
    assert_eq!(
        repeated_failure
            .state()
            .session()
            .unwrap()
            .pending_cleanup(),
        Some(cleanup)
    );
    let completed = reduce_live(
        repeated_failure.state(),
        LiveInput::Event(LiveEvent::CaptureCleanupCompleted {
            correlation: cleanup,
            audio_cancelled: true,
            audio_discarded: true,
            cancellation_cancelled: true,
        }),
    );
    assert!(
        completed
            .state()
            .session()
            .unwrap()
            .pending_cleanup()
            .is_none()
    );
    let duplicate = reduce_live(
        completed.state(),
        LiveInput::Event(LiveEvent::CaptureCleanupCompleted {
            correlation: cleanup,
            audio_cancelled: true,
            audio_discarded: true,
            cancellation_cancelled: true,
        }),
    );
    assert_eq!(
        duplicate.disposition(),
        EventDisposition::Ignored(RejectReason::StaleRevision)
    );
    assert_eq!(duplicate.state(), completed.state());
}

#[test]
fn partial_persistence_remains_open_until_all_recovery_material_is_durable() {
    let failed = reduce_live(
        &recognizing(&started()),
        LiveInput::Event(LiveEvent::RecognitionFailed {
            correlation: recognition_correlation(&recognizing(&started())),
            code: FailureCode::RecognitionProvider,
        }),
    );
    let (correlation, operation_id) = failed
        .effects()
        .iter()
        .find_map(|effect| match effect {
            LiveEffect::PersistRecord {
                correlation,
                operation_id,
                ..
            } => Some((*correlation, *operation_id)),
            _ => None,
        })
        .unwrap();
    let partial = reduce_live(
        failed.state(),
        LiveInput::Event(LiveEvent::PersistenceSucceededForOperation {
            correlation,
            operation_id,
            recovery_id: failed.state().session().unwrap().recovery_id(),
            report: PersistenceReport {
                durable_materials: Vec::new(),
            },
        }),
    );
    let context = partial.state().session().unwrap().recovery().unwrap();
    assert_eq!(partial.state().phase(), Phase::Recovery);
    assert!(!context.is_closed());
    let recovery = voice_core::RecoveryCorrelation::new(
        context.id(),
        context.record_id(),
        context.session_id(),
    );
    let completed = reduce_live(
        partial.state(),
        LiveInput::Event(LiveEvent::RecoveryPersistenceSucceeded {
            recovery,
            report: PersistenceReport {
                durable_materials: context.materials().available_kinds(),
            },
        }),
    );
    assert!(
        completed
            .state()
            .session()
            .unwrap()
            .recovery()
            .unwrap()
            .is_closed()
    );
    assert_eq!(completed.state().phase(), Phase::Completed);
}

#[test]
fn focus_change_invalidates_target_before_callback_without_reactivation() {
    let recognizing_state = recognizing(&started());
    let session = recognizing_state.session().unwrap();
    let focus = reduce_live(
        &recognizing_state,
        LiveInput::Event(LiveEvent::FocusChanged(LiveCorrelation::new(
            session.session_id(),
            session.session_revision(),
            Phase::Recognizing,
        ))),
    );
    let focus_session = focus.state().session().unwrap();
    assert!(focus_session.target_operation().is_none());
    assert!(focus_session.target_invalidated());
    let late_target = reduce_live(
        focus.state(),
        LiveInput::Event(LiveEvent::TargetResolvedForOperation {
            correlation: target_correlation(&recognizing_state),
            resolution: target(),
        }),
    );
    assert_eq!(
        late_target.disposition(),
        EventDisposition::Ignored(RejectReason::StaleRevision)
    );
    let recognition = recognition_correlation(focus.state());
    let processing = reduce_live(
        focus.state(),
        LiveInput::Event(LiveEvent::RecognitionFinal {
            correlation: recognition,
            raw: RawTranscript::new("synthetic raw"),
        }),
    );
    let processing_session = processing.state().session().unwrap();
    let processing_focus = reduce_live(
        processing.state(),
        LiveInput::Event(LiveEvent::FocusChanged(LiveCorrelation::new(
            processing_session.session_id(),
            processing_session.session_revision(),
            Phase::Processing,
        ))),
    );
    let processing_corr = LiveCorrelation::new(
        processing_session.session_id(),
        processing_session.session_revision(),
        Phase::Processing,
    );
    let delivered = reduce_live(
        processing_focus.state(),
        LiveInput::Event(LiveEvent::ProcessingSucceeded {
            correlation: processing_corr,
            result: ProcessingResult {
                processed_text: None,
                final_text: FinalText::new("synthetic raw"),
            },
        }),
    );
    assert!(
        delivered
            .effects()
            .iter()
            .any(|effect| matches!(effect, LiveEffect::PresentResultPanel { .. }))
    );
}

#[test]
fn processing_plan_rejects_multiple_language_model_steps_and_attempt_status_codes_round_trip() {
    let result = ProcessingPlan::try_new(
        RawTranscript::new("synthetic raw"),
        vec![
            ProcessingStep::LanguageModel {
                configuration_id: None,
                enabled: false,
            },
            ProcessingStep::LanguageModel {
                configuration_id: None,
                enabled: false,
            },
        ],
    );
    assert!(matches!(
        result,
        Err(voice_core::ProcessingPlanError::MultipleLanguageModelSteps)
    ));
    for status in [
        voice_core::AttemptStatus::Pending,
        voice_core::AttemptStatus::Succeeded,
        voice_core::AttemptStatus::Failed,
    ] {
        assert_eq!(
            voice_core::AttemptStatus::from_code(status.code()),
            Some(status)
        );
    }
    assert!(voice_core::AttemptStatus::from_code("unknown").is_none());
}

#[test]
fn wire_codes_round_trip_for_phase_outcome_mode_and_failures() {
    for value in [
        Phase::Idle,
        Phase::Capturing,
        Phase::StoppingCapture,
        Phase::Recognizing,
        Phase::Processing,
        Phase::Delivering,
        Phase::Completed,
        Phase::Recovery,
    ] {
        assert_eq!(Phase::from_code(value.code()), Some(value));
    }
    for value in [
        TerminalOutcome::DeliveredAutomatically,
        TerminalOutcome::ManualDeliveryRequired,
        TerminalOutcome::DeliveryUncertain,
        TerminalOutcome::Cancelled,
        TerminalOutcome::Failed,
    ] {
        assert_eq!(TerminalOutcome::from_code(value.code()), Some(value));
    }
    for value in [StartMode::PushToTalk, StartMode::Toggle] {
        assert_eq!(StartMode::from_code(value.code()), Some(value));
    }
    for value in [
        FailureStage::Shortcut,
        FailureStage::Credential,
        FailureStage::ModelManagement,
        FailureStage::Capture,
        FailureStage::Recognition,
        FailureStage::Processing,
        FailureStage::Targeting,
        FailureStage::Delivery,
        FailureStage::Persistence,
        FailureStage::Recovery,
        FailureStage::Retry,
    ] {
        assert_eq!(FailureStage::from_code(value.code()), Some(value));
    }
    for value in [
        FailureCode::ShortcutRegistration,
        FailureCode::CredentialMissing,
        FailureCode::CredentialUnavailable,
        FailureCode::ModelMissing,
        FailureCode::ModelInvalid,
        FailureCode::ModelManagement,
        FailureCode::EmptyAudio,
        FailureCode::DeviceFailure,
        FailureCode::CaptureCleanupFailed,
        FailureCode::RecognitionEmpty,
        FailureCode::RecognitionProvider,
        FailureCode::RecognitionTimeout,
        FailureCode::RecognitionCancelled,
        FailureCode::ProcessingStep,
        FailureCode::ProcessingTimeout,
        FailureCode::TargetUnavailable,
        FailureCode::TargetInvalid,
        FailureCode::InjectionFailed,
        FailureCode::ManualPreservationFailed,
        FailureCode::InsertionUncertain,
        FailureCode::PersistenceUnavailable,
        FailureCode::RecoveryUnavailable,
        FailureCode::RetryIneligible,
        FailureCode::RetryProvider,
        FailureCode::RetryTimeout,
        FailureCode::RetryCancelled,
        FailureCode::RetryEmpty,
    ] {
        assert_eq!(FailureCode::from_code(value.code()), Some(value));
    }
    assert!(Phase::from_code("unknown").is_none());
    assert!(FailureCode::from_code("unknown").is_none());
}

#[test]
fn wire_codes_round_trip_for_retry_delivery_warnings_and_materials() {
    for value in [
        voice_core::RetryMeaning::Retryable,
        voice_core::RetryMeaning::NotRetryable,
        voice_core::RetryMeaning::NoAutomaticRetry,
    ] {
        assert_eq!(
            voice_core::RetryMeaning::from_code(value.code()),
            Some(value)
        );
    }
    for value in [
        voice_core::DeliveryCertainty::NotApplicable,
        voice_core::DeliveryCertainty::Confirmed,
        voice_core::DeliveryCertainty::DefiniteFailure,
        voice_core::DeliveryCertainty::Uncertain,
    ] {
        assert_eq!(
            voice_core::DeliveryCertainty::from_code(value.code()),
            Some(value)
        );
    }
    for value in [
        Warning::MaximumDurationReached,
        Warning::ProcessingFallback,
        Warning::PersistenceUnsaved,
        Warning::IncompletePartialRetained,
        Warning::TargetChanged,
        Warning::LowVolume,
    ] {
        assert_eq!(Warning::from_code(value.code()), Some(value));
    }
    for value in [Durability::NonDurable, Durability::Durable] {
        assert_eq!(Durability::from_code(value.code()), Some(value));
    }
    for value in [
        MaterialKind::RecordedAudio,
        MaterialKind::PartialTranscript,
        MaterialKind::RawTranscript,
        MaterialKind::ProcessedText,
        MaterialKind::FinalText,
        MaterialKind::ResultPanel,
        MaterialKind::ClipboardFallback,
    ] {
        assert_eq!(MaterialKind::from_code(value.code()), Some(value));
    }
    for value in [
        RetryPhase::PendingAttemptPersistence,
        RetryPhase::Recognizing,
        RetryPhase::PendingResultPersistence,
    ] {
        assert_eq!(RetryPhase::from_code(value.code()), Some(value));
    }
    assert!(MaterialKind::from_code("unknown").is_none());
}
