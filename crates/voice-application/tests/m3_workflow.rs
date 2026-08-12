use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use voice_application::{
    ApplicationPorts, ApplicationSupervisor, LiveStartConfig, RetryStartConfig,
};
use voice_core::{
    AudioReferenceId, CancellationTokenId, CaptureLimit, ConfigurationId, DeliveryCertainty,
    DictationRecord, DictationRecordId, DurationLimit, EventDisposition, FailureCode, FailureStage,
    FinalText, InsertionTarget, LiveCommand, LiveCorrelation, LiveEvent, LiveInput, Phase,
    ProcessingPlan, ProcessingResult, ProcessingStep, RawTranscript, RetryCommand, RetryEvent,
    RetryMeaning, SanitizedFailure, SessionId, StartMode, StartRequest, TargetId, TargetResolution,
    TargetToken, TerminalOutcome, Timestamp,
};
use voice_ports::{
    AudioCapturePort, AudioStartRequest, AudioStopRequest, CancellationPort, ClockPort,
    DeterministicCancellation, DeterministicIdentifierSource, FakeAudioCapture, FakeClipboard,
    FakeCredentialStore, FakeHistoryStore, FakeModelManager, FakeRecognitionEngine,
    FakeResultPanel, FakeShortcutRegistry, FakeTargetResolver, FakeTargetValidator,
    FakeTextInjector, FakeTextProcessor, InjectionDisposition, PortCall, PortResult,
    ProcessingRequest, RecognitionCorrelationEnvelope, RecognitionEnginePort, RecognitionRequest,
    TextProcessorPort,
};

#[derive(Clone)]
struct SharedClock(Arc<Mutex<Timestamp>>);
impl ClockPort for SharedClock {
    fn now(&self) -> Timestamp {
        *self.0.lock().expect("clock lock")
    }
}
impl SharedClock {
    fn set(&self, now: Timestamp) {
        *self.0.lock().expect("clock lock") = now;
    }
}

fn config() -> LiveStartConfig {
    LiveStartConfig {
        max_duration: CaptureLimit::from_seconds(60).unwrap(),
        recognition_timeout: DurationLimit::from_seconds(2).unwrap(),
        recognition_configuration_id: ConfigurationId::new(1).unwrap(),
        processing_plan: ProcessingPlan::new(
            RawTranscript::new("synthetic raw"),
            vec![ProcessingStep::BuiltIn {
                rule_id: ConfigurationId::new(2).unwrap(),
                enabled: true,
            }],
        )
        .unwrap(),
    }
}

fn ports(history: FakeHistoryStore, audio: FakeAudioCapture) -> ApplicationPorts {
    ApplicationPorts {
        audio: Box::new(audio),
        shortcuts: Box::new(FakeShortcutRegistry::default()),
        recognition: Box::new(FakeRecognitionEngine::default()),
        processor: Box::new(FakeTextProcessor::default()),
        target_resolver: Box::new(FakeTargetResolver {
            results: VecDeque::from([Ok(TargetResolution::Ineligible)]),
            ..FakeTargetResolver::default()
        }),
        target_validator: Box::new(FakeTargetValidator::default()),
        injector: Box::new(FakeTextInjector::default()),
        result_panel: Box::new(FakeResultPanel::default()),
        clipboard: Box::new(FakeClipboard::default()),
        credentials: Box::new(FakeCredentialStore::default()),
        history: Box::new(history),
        models: Box::new(FakeModelManager::default()),
    }
}

fn supervisor(
    history: FakeHistoryStore,
    audio: FakeAudioCapture,
    clock: SharedClock,
) -> ApplicationSupervisor {
    ApplicationSupervisor::new(
        ports(history, audio),
        Box::new(DeterministicIdentifierSource::default()),
        Box::new(clock),
        Box::new(DeterministicCancellation::default()),
    )
}

struct RecordingAudio {
    calls: Arc<Mutex<Vec<PortCall>>>,
    stop_results: VecDeque<PortResult<()>>,
    cancel_results: VecDeque<PortResult<()>>,
    discard_results: VecDeque<PortResult<()>>,
}

impl AudioCapturePort for RecordingAudio {
    fn start(&mut self, request: AudioStartRequest) -> PortResult<()> {
        self.calls
            .lock()
            .expect("audio calls lock")
            .push(PortCall::AudioStart(request.session_id));
        Ok(())
    }

    fn stop(&mut self, request: AudioStopRequest) -> PortResult<()> {
        self.calls
            .lock()
            .expect("audio calls lock")
            .push(PortCall::AudioStop(request.session_id));
        self.stop_results.pop_front().unwrap_or(Ok(()))
    }

    fn delete(&mut self, reference: AudioReferenceId) -> PortResult<()> {
        self.calls
            .lock()
            .expect("audio calls lock")
            .push(PortCall::AudioDelete(reference));
        Ok(())
    }

    fn cancel(&mut self, request: AudioStopRequest) -> PortResult<()> {
        self.calls
            .lock()
            .expect("audio calls lock")
            .push(PortCall::AudioCancel(request.session_id));
        self.cancel_results.pop_front().unwrap_or(Ok(()))
    }

    fn discard(&mut self, session_id: SessionId) -> PortResult<()> {
        self.calls
            .lock()
            .expect("audio calls lock")
            .push(PortCall::AudioDiscard(session_id));
        self.discard_results.pop_front().unwrap_or(Ok(()))
    }
}

struct RecordingRecognition {
    calls: Arc<Mutex<Vec<PortCall>>>,
}

impl RecognitionEnginePort for RecordingRecognition {
    fn recognize(&mut self, request: RecognitionRequest) -> PortResult<()> {
        let correlation = match request {
            RecognitionRequest::Live { correlation, .. } => {
                RecognitionCorrelationEnvelope::Live(correlation)
            }
            RecognitionRequest::Retry { correlation, .. } => {
                RecognitionCorrelationEnvelope::Retry(correlation)
            }
        };
        self.calls
            .lock()
            .expect("recognition calls lock")
            .push(PortCall::RecognitionStart(correlation));
        Ok(())
    }

    fn cancel(&mut self, token: CancellationTokenId) -> PortResult<()> {
        self.calls
            .lock()
            .expect("recognition calls lock")
            .push(PortCall::RecognitionCancel(token));
        Ok(())
    }
}

struct RecordingCancellation {
    calls: Arc<Mutex<Vec<PortCall>>>,
    token: CancellationTokenId,
    allocated: bool,
}

impl CancellationPort for RecordingCancellation {
    fn allocate(&mut self) -> Result<CancellationTokenId, voice_ports::AllocationError> {
        if self.allocated {
            return Err(voice_ports::AllocationError::Exhausted);
        }
        self.allocated = true;
        Ok(self.token)
    }

    fn cancel(&mut self, token: CancellationTokenId) -> PortResult<()> {
        self.calls
            .lock()
            .expect("cancellation calls lock")
            .push(PortCall::RecognitionCancel(token));
        Ok(())
    }

    fn is_cancelled(&self, _token: CancellationTokenId) -> bool {
        false
    }
}

fn supervisor_with_injection_result(
    history: FakeHistoryStore,
    audio: FakeAudioCapture,
    clock: SharedClock,
    result: voice_ports::PortResult<InjectionDisposition>,
) -> ApplicationSupervisor {
    let target = TargetResolution::Eligible(InsertionTarget::new(
        TargetId::new(50).unwrap(),
        TargetToken::new("synthetic-target"),
        None,
    ));
    let mut resolver = FakeTargetResolver::default();
    resolver.results.push_back(Ok(target));
    let mut injector = FakeTextInjector::default();
    injector.results.push_back(result);
    ApplicationSupervisor::new(
        ApplicationPorts {
            audio: Box::new(audio),
            shortcuts: Box::new(FakeShortcutRegistry::default()),
            recognition: Box::new(FakeRecognitionEngine::default()),
            processor: Box::new(FakeTextProcessor::default()),
            target_resolver: Box::new(resolver),
            target_validator: Box::new(FakeTargetValidator::default()),
            injector: Box::new(injector),
            result_panel: Box::new(FakeResultPanel::default()),
            clipboard: Box::new(FakeClipboard::default()),
            credentials: Box::new(FakeCredentialStore::default()),
            history: Box::new(history),
            models: Box::new(FakeModelManager::default()),
        },
        Box::new(DeterministicIdentifierSource::default()),
        Box::new(clock),
        Box::new(DeterministicCancellation::default()),
    )
}

struct RecordingProcessor {
    process_tokens: Arc<Mutex<Vec<CancellationTokenId>>>,
    cancel_tokens: Arc<Mutex<Vec<CancellationTokenId>>>,
    failure: Option<SanitizedFailure>,
}

impl TextProcessorPort for RecordingProcessor {
    fn process(&mut self, request: ProcessingRequest) -> PortResult<ProcessingResult> {
        self.process_tokens
            .lock()
            .expect("processor process lock")
            .push(request.cancellation_token);
        match self.failure {
            Some(failure) => Err(failure),
            None => Ok(ProcessingResult {
                processed_text: None,
                final_text: FinalText::new("synthetic processed"),
            }),
        }
    }

    fn cancel(&mut self, token: CancellationTokenId) -> PortResult<()> {
        self.cancel_tokens
            .lock()
            .expect("processor cancel lock")
            .push(token);
        Ok(())
    }
}

fn supervisor_with_processor(
    history: FakeHistoryStore,
    audio: FakeAudioCapture,
    clock: SharedClock,
    processor: Box<dyn TextProcessorPort>,
) -> ApplicationSupervisor {
    ApplicationSupervisor::new(
        ApplicationPorts {
            audio: Box::new(audio),
            shortcuts: Box::new(FakeShortcutRegistry::default()),
            recognition: Box::new(FakeRecognitionEngine::default()),
            processor,
            target_resolver: Box::new(FakeTargetResolver {
                results: VecDeque::from([Ok(TargetResolution::Eligible(InsertionTarget::new(
                    TargetId::new(90).unwrap(),
                    TargetToken::new("synthetic-processing-target"),
                    None,
                )))]),
                ..FakeTargetResolver::default()
            }),
            target_validator: Box::new(FakeTargetValidator::default()),
            injector: Box::new(FakeTextInjector::default()),
            result_panel: Box::new(FakeResultPanel::default()),
            clipboard: Box::new(FakeClipboard::default()),
            credentials: Box::new(FakeCredentialStore::default()),
            history: Box::new(history),
            models: Box::new(FakeModelManager::default()),
        },
        Box::new(DeterministicIdentifierSource::default()),
        Box::new(clock),
        Box::new(DeterministicCancellation::default()),
    )
}

fn drive_to_recognizing(supervisor: &mut ApplicationSupervisor) {
    assert_eq!(
        supervisor.start_live(StartMode::Toggle, config()),
        EventDisposition::Applied
    );
    let session = supervisor.live_state().session().unwrap();
    let capturing = LiveCorrelation::new(
        session.session_id(),
        session.session_revision(),
        Phase::Capturing,
    );
    assert_eq!(
        supervisor.submit_live(LiveInput::Command(LiveCommand::StopToggle(capturing))),
        EventDisposition::Applied
    );
    let session = supervisor.live_state().session().unwrap();
    let stopping = LiveCorrelation::new(
        session.session_id(),
        session.session_revision(),
        Phase::StoppingCapture,
    );
    assert_eq!(
        supervisor.submit_live(LiveInput::Event(LiveEvent::CaptureStoppedAt {
            correlation: stopping,
            audio: Some(voice_core::RecordedAudio::new(
                AudioReferenceId::new(10).unwrap(),
                true,
            )),
            at: Timestamp::new(5),
        })),
        EventDisposition::Applied
    );
}

#[test]
fn immediate_capture_start_failure_releases_guard() {
    let failure = voice_core::SanitizedFailure::new(
        voice_core::FailureStage::Capture,
        FailureCode::DeviceFailure,
        voice_core::RetryMeaning::Retryable,
        voice_core::DeliveryCertainty::NotApplicable,
    )
    .unwrap();
    let mut audio = FakeAudioCapture::default();
    audio.start_results.push_back(Err(failure));
    let clock = SharedClock(Arc::new(Mutex::new(Timestamp::new(0))));
    let mut supervisor = supervisor(FakeHistoryStore::default(), audio, clock);
    assert_eq!(
        supervisor.start_live(StartMode::Toggle, config()),
        EventDisposition::Applied
    );
    assert!(!supervisor.work_guard().live());
    assert_eq!(
        supervisor.live_state().session().unwrap().outcome(),
        Some(TerminalOutcome::Failed)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn capture_stop_failure_cleans_up_before_releasing_guard() {
    let failure = voice_core::SanitizedFailure::new(
        FailureStage::Capture,
        FailureCode::DeviceFailure,
        voice_core::RetryMeaning::Retryable,
        voice_core::DeliveryCertainty::NotApplicable,
    )
    .unwrap();
    let audio_calls = Arc::new(Mutex::new(Vec::new()));
    let cleanup_failure = voice_core::SanitizedFailure::new(
        FailureStage::Capture,
        FailureCode::CaptureCleanupFailed,
        voice_core::RetryMeaning::Retryable,
        voice_core::DeliveryCertainty::NotApplicable,
    )
    .unwrap();
    let audio = RecordingAudio {
        calls: audio_calls.clone(),
        stop_results: VecDeque::from([Err(failure)]),
        cancel_results: VecDeque::from([Ok(()), Ok(())]),
        discard_results: VecDeque::from([Err(cleanup_failure), Ok(())]),
    };
    let recognition_calls = Arc::new(Mutex::new(Vec::new()));
    let clock = SharedClock(Arc::new(Mutex::new(Timestamp::new(0))));
    let mut supervisor = ApplicationSupervisor::new(
        ApplicationPorts {
            audio: Box::new(audio),
            shortcuts: Box::new(FakeShortcutRegistry::default()),
            recognition: Box::new(RecordingRecognition {
                calls: recognition_calls.clone(),
            }),
            processor: Box::new(FakeTextProcessor::default()),
            target_resolver: Box::new(FakeTargetResolver::default()),
            target_validator: Box::new(FakeTargetValidator::default()),
            injector: Box::new(FakeTextInjector::default()),
            result_panel: Box::new(FakeResultPanel::default()),
            clipboard: Box::new(FakeClipboard::default()),
            credentials: Box::new(FakeCredentialStore::default()),
            history: Box::new(FakeHistoryStore::default()),
            models: Box::new(FakeModelManager::default()),
        },
        Box::new(DeterministicIdentifierSource::default()),
        Box::new(clock),
        Box::new(RecordingCancellation {
            calls: audio_calls.clone(),
            token: CancellationTokenId::new(1).unwrap(),
            allocated: false,
        }),
    );

    assert_eq!(
        supervisor.start_live(StartMode::Toggle, config()),
        EventDisposition::Applied
    );
    let session = supervisor.live_state().session().unwrap();
    let correlation = LiveCorrelation::new(
        session.session_id(),
        session.session_revision(),
        Phase::Capturing,
    );
    assert_eq!(
        supervisor.submit_live(LiveInput::Command(LiveCommand::StopToggle(correlation))),
        EventDisposition::Applied
    );

    let failed = supervisor.live_state().session().unwrap();
    assert_eq!(failed.outcome(), Some(TerminalOutcome::Failed));
    assert!(failed.audio().is_none());
    assert!(
        !failed
            .materials()
            .state(voice_core::MaterialKind::RecordedAudio)
            .available()
    );
    assert!(recognition_calls.lock().unwrap().is_empty());
    assert!(failed.pending_cleanup().is_some());
    assert_eq!(failed.failure().unwrap().code(), FailureCode::DeviceFailure);
    assert!(supervisor.work_guard().live());
    assert_eq!(
        supervisor.start_live(StartMode::Toggle, config()),
        EventDisposition::Ignored(voice_core::RejectReason::CompetingWork)
    );
    assert_eq!(
        supervisor.retry_capture_cleanup(),
        EventDisposition::Applied
    );
    assert!(
        supervisor
            .live_state()
            .session()
            .unwrap()
            .pending_cleanup()
            .is_none()
    );
    assert!(!supervisor.work_guard().live());
    assert_eq!(
        audio_calls.lock().unwrap().as_slice(),
        &[
            PortCall::AudioStart(SessionId::new(1).unwrap()),
            PortCall::AudioStop(SessionId::new(1).unwrap()),
            PortCall::AudioCancel(SessionId::new(1).unwrap()),
            PortCall::AudioDiscard(SessionId::new(1).unwrap()),
            PortCall::RecognitionCancel(CancellationTokenId::new(1).unwrap()),
            PortCall::AudioCancel(SessionId::new(1).unwrap()),
            PortCall::AudioDiscard(SessionId::new(1).unwrap()),
            PortCall::RecognitionCancel(CancellationTokenId::new(1).unwrap()),
        ]
    );
}

#[test]
fn clock_drives_capture_and_recognition_deadlines() {
    let clock = SharedClock(Arc::new(Mutex::new(Timestamp::new(0))));
    let mut supervisor = supervisor(
        FakeHistoryStore::default(),
        FakeAudioCapture::default(),
        clock.clone(),
    );
    assert_eq!(
        supervisor.start_live(StartMode::Toggle, config()),
        EventDisposition::Applied
    );
    clock.set(Timestamp::new(60_000));
    assert_eq!(supervisor.tick(), EventDisposition::Applied);
    assert_eq!(supervisor.live_state().phase(), Phase::StoppingCapture);
    let session = supervisor.live_state().session().unwrap();
    let stopping = LiveCorrelation::new(
        session.session_id(),
        session.session_revision(),
        Phase::StoppingCapture,
    );
    assert_eq!(
        supervisor.submit_live(LiveInput::Event(LiveEvent::CaptureStoppedAt {
            correlation: stopping,
            audio: Some(voice_core::RecordedAudio::new(
                AudioReferenceId::new(11).unwrap(),
                true,
            )),
            at: Timestamp::new(60_000),
        })),
        EventDisposition::Applied
    );
    let session = supervisor.live_state().session().unwrap();
    let recognition = voice_core::RecognitionCorrelation::new(
        LiveCorrelation::new(
            session.session_id(),
            session.session_revision(),
            Phase::Recognizing,
        ),
        session.attempt_id(),
        session.attempt_revision(),
    );
    clock.set(Timestamp::new(62_000));
    assert_eq!(supervisor.tick(), EventDisposition::Applied);
    assert_eq!(
        supervisor.live_state().session().unwrap().outcome(),
        Some(TerminalOutcome::Failed)
    );
    let late = supervisor.submit_live(LiveInput::Event(LiveEvent::RecognitionFinal {
        correlation: recognition,
        raw: RawTranscript::new("late"),
    }));
    assert!(matches!(
        late,
        EventDisposition::Ignored(
            voice_core::RejectReason::TerminalCallback | voice_core::RejectReason::StaleRevision,
        )
    ));
}

#[test]
fn processing_request_carries_session_token_and_timeout_cancels_processor() {
    let process_tokens = Arc::new(Mutex::new(Vec::new()));
    let cancel_tokens = Arc::new(Mutex::new(Vec::new()));
    let processing_failure = SanitizedFailure::new(
        FailureStage::Processing,
        FailureCode::ProcessingTimeout,
        RetryMeaning::Retryable,
        DeliveryCertainty::NotApplicable,
    )
    .unwrap();
    let processor = RecordingProcessor {
        process_tokens: process_tokens.clone(),
        cancel_tokens: cancel_tokens.clone(),
        failure: Some(processing_failure),
    };
    let clock = SharedClock(Arc::new(Mutex::new(Timestamp::new(0))));
    let mut supervisor = supervisor_with_processor(
        FakeHistoryStore::default(),
        FakeAudioCapture::default(),
        clock,
        Box::new(processor),
    );
    drive_to_recognizing(&mut supervisor);
    let session = supervisor.live_state().session().unwrap();
    let token = session.cancellation_token();
    let recognition = voice_core::RecognitionCorrelation::new(
        LiveCorrelation::new(
            session.session_id(),
            session.session_revision(),
            Phase::Recognizing,
        ),
        session.attempt_id(),
        session.attempt_revision(),
    );
    assert_eq!(
        supervisor.submit_live(LiveInput::Event(LiveEvent::RecognitionFinal {
            correlation: recognition,
            raw: RawTranscript::new("synthetic raw"),
        })),
        EventDisposition::Applied
    );
    assert_eq!(process_tokens.lock().unwrap().as_slice(), &[token]);
    assert_eq!(cancel_tokens.lock().unwrap().as_slice(), &[token]);
    let session = supervisor.live_state().session().unwrap();
    assert_eq!(
        session.raw().map(RawTranscript::as_str),
        Some("synthetic raw")
    );
    assert_eq!(
        session.final_text().map(FinalText::as_str),
        Some("synthetic raw")
    );
    assert!(session.processed().is_none());
    assert!(
        session
            .warnings()
            .contains(&voice_core::Warning::ProcessingFallback)
    );
    assert_eq!(
        session.failure().unwrap().code(),
        FailureCode::ProcessingTimeout
    );
}

#[test]
fn recovery_payload_survives_archiving_and_exact_success() {
    let failure = voice_core::SanitizedFailure::new(
        voice_core::FailureStage::Persistence,
        FailureCode::PersistenceUnavailable,
        voice_core::RetryMeaning::Retryable,
        voice_core::DeliveryCertainty::NotApplicable,
    )
    .unwrap();
    let mut history = FakeHistoryStore::default();
    history.persist_results.push_back(Err(failure));
    let clock = SharedClock(Arc::new(Mutex::new(Timestamp::new(0))));
    let mut supervisor = supervisor(history, FakeAudioCapture::default(), clock);
    drive_to_recognizing(&mut supervisor);
    let session = supervisor.live_state().session().unwrap();
    let recognition = voice_core::RecognitionCorrelation::new(
        LiveCorrelation::new(
            session.session_id(),
            session.session_revision(),
            Phase::Recognizing,
        ),
        session.attempt_id(),
        session.attempt_revision(),
    );
    supervisor.submit_live(LiveInput::Event(LiveEvent::RecognitionFinal {
        correlation: recognition,
        raw: RawTranscript::new("synthetic final"),
    }));
    let recovery = supervisor
        .live_state()
        .session()
        .unwrap()
        .recovery()
        .unwrap()
        .clone();
    assert!(recovery.record().final_text().is_some());
    assert_eq!(
        supervisor.start_live(StartMode::PushToTalk, config()),
        EventDisposition::Applied
    );
    let correlation = voice_core::RecoveryCorrelation::new(
        recovery.id(),
        recovery.record_id(),
        recovery.session_id(),
    );
    assert_eq!(
        supervisor.persist_recovery(correlation),
        EventDisposition::Applied
    );
    assert!(
        supervisor
            .recoveries()
            .iter()
            .find(|context| context.id() == recovery.id())
            .unwrap()
            .is_closed()
    );
}

#[test]
fn retry_uses_pending_and_terminal_persistence_without_delivery_side_effects() {
    let mut record = DictationRecord::new(
        DictationRecordId::new(40).unwrap(),
        SessionId::new(41).unwrap(),
    );
    record.set_recorded_audio(voice_core::RecordedAudio::new(
        AudioReferenceId::new(42).unwrap(),
        true,
    ));
    record.set_outcome(TerminalOutcome::Failed);
    let mut materials = voice_core::MaterialLedger::new();
    materials.set(
        voice_core::MaterialKind::RecordedAudio,
        voice_core::MaterialState::durable(),
    );
    record.set_materials(materials);
    record.mark_durable();

    let clock = SharedClock(Arc::new(Mutex::new(Timestamp::new(0))));
    let mut supervisor = supervisor(
        FakeHistoryStore::default(),
        FakeAudioCapture::default(),
        clock,
    );
    assert_eq!(
        supervisor.retry_recognition(
            record,
            RetryStartConfig {
                recognition_configuration_id: ConfigurationId::new(43).unwrap(),
                timeout: DurationLimit::from_seconds(2).unwrap(),
            },
        ),
        EventDisposition::Applied
    );
    let correlation = supervisor.retry_state().unwrap().active().unwrap();
    assert_eq!(
        correlation.expected_retry_phase(),
        voice_core::RetryPhase::Recognizing
    );
    assert_eq!(
        supervisor.submit_retry(voice_core::RetryInput::Event(
            RetryEvent::RecognitionFinal {
                correlation,
                raw: RawTranscript::new("retry text"),
            }
        )),
        EventDisposition::Applied
    );
    assert!(supervisor.retry_state().unwrap().active().is_none());
    assert_eq!(
        supervisor.retry_state().unwrap().record().attempts().len(),
        1
    );
}

#[test]
fn retry_result_persistence_failure_archives_full_pending_record_and_allows_new_live_work() {
    let mut record = DictationRecord::new(
        DictationRecordId::new(80).unwrap(),
        SessionId::new(81).unwrap(),
    );
    record.set_recorded_audio(voice_core::RecordedAudio::new(
        AudioReferenceId::new(82).unwrap(),
        true,
    ));
    record.set_outcome(TerminalOutcome::Failed);
    let mut materials = voice_core::MaterialLedger::new();
    materials.set(
        voice_core::MaterialKind::RecordedAudio,
        voice_core::MaterialState::durable(),
    );
    record.set_materials(materials);
    record.mark_durable();

    let persistence_failure = SanitizedFailure::new(
        FailureStage::Persistence,
        FailureCode::PersistenceUnavailable,
        RetryMeaning::Retryable,
        DeliveryCertainty::NotApplicable,
    )
    .unwrap();
    let history = FakeHistoryStore {
        retry_results: VecDeque::from([Ok(()), Err(persistence_failure)]),
        ..FakeHistoryStore::default()
    };
    let clock = SharedClock(Arc::new(Mutex::new(Timestamp::new(0))));
    let mut supervisor = supervisor(history, FakeAudioCapture::default(), clock);
    assert_eq!(
        supervisor.retry_recognition(
            record,
            RetryStartConfig {
                recognition_configuration_id: ConfigurationId::new(83).unwrap(),
                timeout: DurationLimit::from_seconds(2).unwrap(),
            },
        ),
        EventDisposition::Applied
    );
    let correlation = supervisor.retry_state().unwrap().active().unwrap();
    assert_eq!(
        supervisor.submit_retry(voice_core::RetryInput::Event(
            RetryEvent::RecognitionFinal {
                correlation,
                raw: RawTranscript::new("retry text"),
            },
        )),
        EventDisposition::Applied
    );
    assert!(!supervisor.work_guard().retry());
    assert_eq!(supervisor.recoveries().len(), 1);
    let recovery = supervisor.recoveries()[0].clone();
    assert_eq!(recovery.record().attempts().len(), 1);
    assert!(!recovery.record().is_durable());
    let durable_pending = supervisor.retry_state().unwrap().record().clone();
    assert_eq!(durable_pending.attempts().len(), 1);
    assert!(durable_pending.is_durable());
    assert_eq!(
        supervisor.persist_recovery(voice_core::RecoveryCorrelation::new(
            recovery.id(),
            recovery.record_id(),
            recovery.session_id(),
        )),
        EventDisposition::Applied
    );
    assert!(supervisor.recoveries()[0].is_closed());
    assert_eq!(
        supervisor.retry_recognition(
            durable_pending,
            RetryStartConfig {
                recognition_configuration_id: ConfigurationId::new(84).unwrap(),
                timeout: DurationLimit::from_seconds(2).unwrap(),
            },
        ),
        EventDisposition::Applied
    );
    let second = supervisor.retry_state().unwrap().active().unwrap();
    assert_eq!(
        second.attempt_revision().get(),
        correlation.attempt_revision().get() + 1
    );
    assert_eq!(
        supervisor.submit_retry(voice_core::RetryInput::Event(
            RetryEvent::RecognitionFinal {
                correlation: second,
                raw: RawTranscript::new("second retry text"),
            },
        )),
        EventDisposition::Applied
    );
    assert_eq!(
        supervisor.retry_state().unwrap().record().attempts().len(),
        2
    );
    assert_eq!(
        supervisor.start_live(StartMode::Toggle, config()),
        EventDisposition::Applied
    );
}

#[test]
fn public_submission_guard_blocks_direct_competing_commands() {
    let clock = SharedClock(Arc::new(Mutex::new(Timestamp::new(0))));
    let mut live_supervisor = supervisor(
        FakeHistoryStore::default(),
        FakeAudioCapture::default(),
        clock.clone(),
    );
    drive_to_recognizing(&mut live_supervisor);
    let retry = RetryCommand::Begin {
        attempt_id: voice_core::RecognitionAttemptId::new(60).unwrap(),
        configuration_id: ConfigurationId::new(61).unwrap(),
        timeout: DurationLimit::from_seconds(2).unwrap(),
        cancellation_token: CancellationTokenId::new(62).unwrap(),
        recovery_id: voice_core::RecoveryId::new(73).unwrap(),
        started_at: Timestamp::new(0),
    };
    assert_eq!(
        live_supervisor.submit_retry(voice_core::RetryInput::Command(retry)),
        EventDisposition::Ignored(voice_core::RejectReason::CompetingWork)
    );

    let mut record = DictationRecord::new(
        DictationRecordId::new(63).unwrap(),
        SessionId::new(64).unwrap(),
    );
    record.set_recorded_audio(voice_core::RecordedAudio::new(
        AudioReferenceId::new(65).unwrap(),
        true,
    ));
    record.set_outcome(TerminalOutcome::Failed);
    let mut materials = voice_core::MaterialLedger::new();
    materials.set(
        voice_core::MaterialKind::RecordedAudio,
        voice_core::MaterialState::durable(),
    );
    record.set_materials(materials);
    record.mark_durable();
    let mut retry_supervisor = supervisor(
        FakeHistoryStore::default(),
        FakeAudioCapture::default(),
        clock,
    );
    assert_eq!(
        retry_supervisor.retry_recognition(
            record,
            RetryStartConfig {
                recognition_configuration_id: ConfigurationId::new(66).unwrap(),
                timeout: DurationLimit::from_seconds(2).unwrap(),
            },
        ),
        EventDisposition::Applied
    );
    let start = StartRequest {
        session_id: SessionId::new(67).unwrap(),
        record_id: DictationRecordId::new(68).unwrap(),
        max_duration: CaptureLimit::from_seconds(60).unwrap(),
        recognition_timeout: DurationLimit::from_seconds(2).unwrap(),
        started_at: Timestamp::new(0),
        cancellation_token: CancellationTokenId::new(69).unwrap(),
        recovery_id: voice_core::RecoveryId::new(72).unwrap(),
        recognition_attempt_id: voice_core::RecognitionAttemptId::new(70).unwrap(),
        recognition_configuration_id: ConfigurationId::new(71).unwrap(),
        processing_plan: ProcessingPlan::new(RawTranscript::new("synthetic raw"), Vec::new())
            .unwrap(),
    };
    assert_eq!(
        retry_supervisor.submit_live(LiveInput::Command(LiveCommand::Start {
            mode: StartMode::Toggle,
            request: start,
        })),
        EventDisposition::Ignored(voice_core::RejectReason::CompetingWork)
    );
}

#[test]
fn uncertain_injector_failure_preserves_delivery_uncertainty() {
    let failure = SanitizedFailure::new(
        FailureStage::Delivery,
        FailureCode::InsertionUncertain,
        RetryMeaning::NoAutomaticRetry,
        DeliveryCertainty::Uncertain,
    )
    .unwrap();
    let clock = SharedClock(Arc::new(Mutex::new(Timestamp::new(0))));
    let mut supervisor = supervisor_with_injection_result(
        FakeHistoryStore::default(),
        FakeAudioCapture::default(),
        clock,
        Err(failure),
    );
    drive_to_recognizing(&mut supervisor);
    let session = supervisor.live_state().session().unwrap();
    let recognition = voice_core::RecognitionCorrelation::new(
        LiveCorrelation::new(
            session.session_id(),
            session.session_revision(),
            Phase::Recognizing,
        ),
        session.attempt_id(),
        session.attempt_revision(),
    );
    assert_eq!(
        supervisor.submit_live(LiveInput::Event(LiveEvent::RecognitionFinal {
            correlation: recognition,
            raw: RawTranscript::new("synthetic final"),
        })),
        EventDisposition::Applied
    );
    assert_eq!(
        supervisor.live_state().session().unwrap().outcome(),
        Some(TerminalOutcome::DeliveryUncertain)
    );
}

#[test]
fn capture_escape_dispatches_cancel_discard_and_cancellation_in_order() {
    let clock = SharedClock(Arc::new(Mutex::new(Timestamp::new(0))));
    let mut supervisor = supervisor(
        FakeHistoryStore::default(),
        FakeAudioCapture::default(),
        clock,
    );
    assert_eq!(
        supervisor.start_live(StartMode::Toggle, config()),
        EventDisposition::Applied
    );
    let session = supervisor.live_state().session().unwrap();
    let correlation = LiveCorrelation::new(
        session.session_id(),
        session.session_revision(),
        Phase::Capturing,
    );
    assert_eq!(
        supervisor.submit_live(LiveInput::Command(LiveCommand::Escape(correlation))),
        EventDisposition::Applied
    );
    assert_eq!(
        supervisor.live_state().session().unwrap().outcome(),
        Some(TerminalOutcome::Cancelled)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn capture_cleanup_failure_is_bounded_retriable_and_blocks_replacement_work() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let cleanup_failure = SanitizedFailure::new(
        FailureStage::Capture,
        FailureCode::DeviceFailure,
        RetryMeaning::Retryable,
        DeliveryCertainty::NotApplicable,
    )
    .unwrap();
    let audio = RecordingAudio {
        calls: calls.clone(),
        stop_results: VecDeque::new(),
        cancel_results: VecDeque::from([Ok(()), Ok(())]),
        discard_results: VecDeque::from([Err(cleanup_failure), Err(cleanup_failure), Ok(())]),
    };
    let cancellation = RecordingCancellation {
        calls: calls.clone(),
        token: CancellationTokenId::new(900).unwrap(),
        allocated: false,
    };
    let clock = SharedClock(Arc::new(Mutex::new(Timestamp::new(0))));
    let mut supervisor = ApplicationSupervisor::new(
        ApplicationPorts {
            audio: Box::new(audio),
            shortcuts: Box::new(FakeShortcutRegistry::default()),
            recognition: Box::new(FakeRecognitionEngine::default()),
            processor: Box::new(FakeTextProcessor::default()),
            target_resolver: Box::new(FakeTargetResolver::default()),
            target_validator: Box::new(FakeTargetValidator::default()),
            injector: Box::new(FakeTextInjector::default()),
            result_panel: Box::new(FakeResultPanel::default()),
            clipboard: Box::new(FakeClipboard::default()),
            credentials: Box::new(FakeCredentialStore::default()),
            history: Box::new(FakeHistoryStore::default()),
            models: Box::new(FakeModelManager::default()),
        },
        Box::new(DeterministicIdentifierSource::default()),
        Box::new(clock),
        Box::new(cancellation),
    );
    assert_eq!(
        supervisor.start_live(StartMode::Toggle, config()),
        EventDisposition::Applied
    );
    let session = supervisor.live_state().session().unwrap();
    let correlation = LiveCorrelation::new(
        session.session_id(),
        session.session_revision(),
        Phase::Capturing,
    );
    assert_eq!(
        supervisor.submit_live(LiveInput::Command(LiveCommand::Escape(correlation))),
        EventDisposition::Applied
    );
    let failed_session = supervisor.live_state().session().unwrap();
    assert_eq!(failed_session.outcome(), Some(TerminalOutcome::Cancelled));
    assert!(failed_session.pending_cleanup().is_some());
    assert!(supervisor.recoveries().is_empty());
    assert!(supervisor.work_guard().live());
    assert_eq!(
        supervisor.start_live(StartMode::Toggle, config()),
        EventDisposition::Ignored(voice_core::RejectReason::CompetingWork)
    );
    assert_eq!(
        supervisor.retry_recognition(
            DictationRecord::new(
                DictationRecordId::new(901).unwrap(),
                SessionId::new(902).unwrap()
            ),
            RetryStartConfig {
                recognition_configuration_id: ConfigurationId::new(903).unwrap(),
                timeout: DurationLimit::from_seconds(2).unwrap(),
            },
        ),
        EventDisposition::Ignored(voice_core::RejectReason::CompetingWork)
    );
    assert_eq!(
        supervisor.retry_capture_cleanup(),
        EventDisposition::Applied
    );
    assert!(
        supervisor
            .live_state()
            .session()
            .unwrap()
            .pending_cleanup()
            .is_some()
    );
    assert!(supervisor.work_guard().live());
    assert_eq!(
        supervisor.retry_capture_cleanup(),
        EventDisposition::Applied
    );
    assert!(
        supervisor
            .live_state()
            .session()
            .unwrap()
            .pending_cleanup()
            .is_none()
    );
    assert!(!supervisor.work_guard().live());
    assert_eq!(
        supervisor.retry_capture_cleanup(),
        EventDisposition::Ignored(voice_core::RejectReason::StaleRevision)
    );
    let calls = calls.lock().unwrap().clone();
    assert_eq!(
        calls,
        vec![
            PortCall::AudioStart(SessionId::new(1).unwrap()),
            PortCall::AudioCancel(SessionId::new(1).unwrap()),
            PortCall::AudioDiscard(SessionId::new(1).unwrap()),
            PortCall::RecognitionCancel(CancellationTokenId::new(900).unwrap()),
            PortCall::AudioCancel(SessionId::new(1).unwrap()),
            PortCall::AudioDiscard(SessionId::new(1).unwrap()),
            PortCall::RecognitionCancel(CancellationTokenId::new(900).unwrap()),
            PortCall::AudioCancel(SessionId::new(1).unwrap()),
            PortCall::AudioDiscard(SessionId::new(1).unwrap()),
            PortCall::RecognitionCancel(CancellationTokenId::new(900).unwrap()),
        ]
    );
}

#[test]
fn start_allocations_fail_closed_without_creating_live_work() {
    let clock = SharedClock(Arc::new(Mutex::new(Timestamp::new(0))));
    let mut supervisor = ApplicationSupervisor::new(
        ports(FakeHistoryStore::default(), FakeAudioCapture::default()),
        Box::new(DeterministicIdentifierSource::new(u64::MAX)),
        Box::new(clock),
        Box::new(DeterministicCancellation::default()),
    );
    assert_eq!(
        supervisor.start_live(StartMode::Toggle, config()),
        EventDisposition::Ignored(voice_core::RejectReason::AllocationExhausted)
    );
    assert_eq!(supervisor.live_state().phase(), Phase::Idle);
    assert!(!supervisor.work_guard().live());
}

#[test]
fn late_start_allocation_failure_creates_no_live_work_or_effects() {
    let clock = SharedClock(Arc::new(Mutex::new(Timestamp::new(0))));
    let mut cancellation = DeterministicCancellation::new(u64::MAX);
    assert!(cancellation.allocate().is_ok());
    let mut supervisor = ApplicationSupervisor::new(
        ports(FakeHistoryStore::default(), FakeAudioCapture::default()),
        Box::new(DeterministicIdentifierSource::default()),
        Box::new(clock),
        Box::new(cancellation),
    );
    assert_eq!(
        supervisor.start_live(StartMode::Toggle, config()),
        EventDisposition::Ignored(voice_core::RejectReason::AllocationExhausted)
    );
    assert_eq!(supervisor.live_state().phase(), Phase::Idle);
    assert!(!supervisor.work_guard().live());
    assert!(supervisor.pending_live_effects().is_empty());
}

#[test]
fn retry_cancellation_allocation_failure_creates_no_retry_work() {
    let mut record = DictationRecord::new(
        DictationRecordId::new(121).unwrap(),
        SessionId::new(122).unwrap(),
    );
    record.set_recorded_audio(voice_core::RecordedAudio::new(
        AudioReferenceId::new(123).unwrap(),
        true,
    ));
    record.set_outcome(TerminalOutcome::Failed);
    let mut materials = voice_core::MaterialLedger::new();
    materials.set(
        voice_core::MaterialKind::RecordedAudio,
        voice_core::MaterialState::durable(),
    );
    record.set_materials(materials);
    record.mark_durable();
    let clock = SharedClock(Arc::new(Mutex::new(Timestamp::new(0))));
    let mut cancellation = DeterministicCancellation::new(u64::MAX);
    assert!(cancellation.allocate().is_ok());
    let mut supervisor = ApplicationSupervisor::new(
        ports(FakeHistoryStore::default(), FakeAudioCapture::default()),
        Box::new(DeterministicIdentifierSource::default()),
        Box::new(clock),
        Box::new(cancellation),
    );
    assert_eq!(
        supervisor.retry_recognition(
            record,
            RetryStartConfig {
                recognition_configuration_id: ConfigurationId::new(124).unwrap(),
                timeout: DurationLimit::from_seconds(2).unwrap(),
            },
        ),
        EventDisposition::Ignored(voice_core::RejectReason::AllocationExhausted)
    );
    assert!(supervisor.retry_state().is_none());
    assert!(!supervisor.work_guard().retry());
    assert!(supervisor.pending_retry_effects().is_empty());
}

#[test]
fn archived_partial_recovery_stays_open_until_follow_up_success() {
    let failure = SanitizedFailure::new(
        FailureStage::Persistence,
        FailureCode::PersistenceUnavailable,
        RetryMeaning::Retryable,
        DeliveryCertainty::NotApplicable,
    )
    .unwrap();
    let mut history = FakeHistoryStore::default();
    history.persist_results.push_back(Err(failure));
    history.recovery_results.extend([
        Ok(voice_core::PersistenceReport {
            durable_materials: Vec::new(),
        }),
        Ok(voice_core::PersistenceReport {
            durable_materials: vec![
                voice_core::MaterialKind::RecordedAudio,
                voice_core::MaterialKind::RawTranscript,
                voice_core::MaterialKind::FinalText,
                voice_core::MaterialKind::ResultPanel,
            ],
        }),
    ]);
    let clock = SharedClock(Arc::new(Mutex::new(Timestamp::new(0))));
    let mut supervisor = supervisor(history, FakeAudioCapture::default(), clock);
    drive_to_recognizing(&mut supervisor);
    let session = supervisor.live_state().session().unwrap();
    let recognition = voice_core::RecognitionCorrelation::new(
        LiveCorrelation::new(
            session.session_id(),
            session.session_revision(),
            Phase::Recognizing,
        ),
        session.attempt_id(),
        session.attempt_revision(),
    );
    assert_eq!(
        supervisor.submit_live(LiveInput::Event(LiveEvent::RecognitionFinal {
            correlation: recognition,
            raw: RawTranscript::new("synthetic final"),
        })),
        EventDisposition::Applied
    );
    let recovery = supervisor
        .live_state()
        .session()
        .unwrap()
        .recovery()
        .unwrap()
        .clone();
    assert_eq!(
        supervisor.start_live(StartMode::Toggle, config()),
        EventDisposition::Applied
    );
    let correlation = voice_core::RecoveryCorrelation::new(
        recovery.id(),
        recovery.record_id(),
        recovery.session_id(),
    );
    assert_eq!(
        supervisor.persist_recovery(correlation),
        EventDisposition::Applied
    );
    assert!(!supervisor.recoveries()[0].is_closed());
    assert_eq!(
        supervisor.persist_recovery(correlation),
        EventDisposition::Applied
    );
    assert!(supervisor.recoveries()[0].is_closed());
}
