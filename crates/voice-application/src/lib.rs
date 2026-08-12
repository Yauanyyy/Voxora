use std::collections::VecDeque;

use voice_core::{
    CaptureLimit, ConfigurationId, DeliveryCertainty, DictationRecord, DurationLimit,
    EventDisposition, InsertionTarget, LiveCommand, LiveCorrelation, LiveEffect, LiveEvent,
    LiveInput, LiveState, Phase, ProcessingPlan, RecognitionCorrelation, RecoveryContext,
    RecoveryCorrelation, RetryCommand, RetryEffect, RetryEvent, RetryInput, RetryState, StartMode,
    StartRequest, reduce_live, reduce_retry,
};
use voice_ports::{
    AllocationError, AudioCapturePort, AudioStartRequest, AudioStopRequest, CancellationPort,
    ClipboardPort, ClockPort, CredentialStorePort, HistoryPersistRequest, HistoryStorePort,
    IdentifierSource, InjectionDisposition, InsertionRequest, ModelManagerPort, PortResult,
    ProcessingRequest, RecognitionEnginePort, RecognitionRequest, ResultPanelPort,
    RetryAttemptPersistRequest, RetryResultPersistRequest, ShortcutPort, TargetRequest,
    TargetResolverPort, TargetValidatorPort, TextInjectorPort, TextProcessorPort,
};

pub struct ApplicationPorts {
    pub audio: Box<dyn AudioCapturePort>,
    pub shortcuts: Box<dyn ShortcutPort>,
    pub recognition: Box<dyn RecognitionEnginePort>,
    pub processor: Box<dyn TextProcessorPort>,
    pub target_resolver: Box<dyn TargetResolverPort>,
    pub target_validator: Box<dyn TargetValidatorPort>,
    pub injector: Box<dyn TextInjectorPort>,
    pub result_panel: Box<dyn ResultPanelPort>,
    pub clipboard: Box<dyn ClipboardPort>,
    pub credentials: Box<dyn CredentialStorePort>,
    pub history: Box<dyn HistoryStorePort>,
    pub models: Box<dyn ModelManagerPort>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveStartConfig {
    pub max_duration: CaptureLimit,
    pub recognition_timeout: DurationLimit,
    pub recognition_configuration_id: ConfigurationId,
    pub processing_plan: ProcessingPlan,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetryStartConfig {
    pub recognition_configuration_id: ConfigurationId,
    pub timeout: DurationLimit,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorkGuard {
    live: bool,
    retry: bool,
}
impl WorkGuard {
    #[must_use]
    pub const fn can_start_live(self) -> bool {
        !self.live && !self.retry
    }
    #[must_use]
    pub const fn can_start_retry(self) -> bool {
        !self.live && !self.retry
    }
    #[must_use]
    pub const fn live(self) -> bool {
        self.live
    }
    #[must_use]
    pub const fn retry(self) -> bool {
        self.retry
    }
}

/// Owns only the pure live state and pending live effects.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LiveSessionSupervisor {
    state: LiveState,
    effects: Vec<LiveEffect>,
}
impl LiveSessionSupervisor {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: LiveState::Idle,
            effects: Vec::new(),
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
    fn submit(&mut self, input: LiveInput) -> EventDisposition {
        let transition = reduce_live(&self.state, input);
        let disposition = transition.disposition();
        let (state, effects, _) = transition.into_parts();
        self.state = state;
        self.effects.extend(effects);
        disposition
    }
    fn take_effects(&mut self) -> Vec<LiveEffect> {
        std::mem::take(&mut self.effects)
    }
}

/// Owns recovery payloads after the live state is archived.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecoveryService {
    contexts: Vec<RecoveryContext>,
}
impl RecoveryService {
    #[must_use]
    pub fn contexts(&self) -> &[RecoveryContext] {
        &self.contexts
    }
    fn archive(&mut self, context: RecoveryContext) {
        if !self.contexts.iter().any(|old| old.id() == context.id()) {
            self.contexts.push(context);
        }
    }
    fn find(&self, correlation: RecoveryCorrelation) -> Option<&RecoveryContext> {
        self.contexts.iter().find(|context| {
            !context.is_closed()
                && context.id() == correlation.recovery_id()
                && context.record_id() == correlation.record_id()
                && context.session_id() == correlation.session_id()
        })
    }
    fn close_success(
        &mut self,
        correlation: RecoveryCorrelation,
        report: &voice_core::PersistenceReport,
    ) -> bool {
        let Some(context) = self.contexts.iter_mut().find(|context| {
            !context.is_closed()
                && context.id() == correlation.recovery_id()
                && context.record_id() == correlation.record_id()
                && context.session_id() == correlation.session_id()
        }) else {
            return false;
        };
        context.mark_durable(&report.durable_materials);
        if context.record().materials().all_available_durable() {
            context.close();
        }
        true
    }
}

/// Owns only record-scoped retry reducer state/effects.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RetryService {
    state: Option<RetryState>,
    effects: Vec<RetryEffect>,
}
impl RetryService {
    #[must_use]
    pub const fn state(&self) -> Option<&RetryState> {
        self.state.as_ref()
    }
    #[must_use]
    pub fn effects(&self) -> &[RetryEffect] {
        &self.effects
    }
    fn submit(&mut self, input: RetryInput) -> EventDisposition {
        let Some(state) = self.state.as_ref() else {
            return EventDisposition::Ignored(voice_core::RejectReason::NoActiveSession);
        };
        let transition = reduce_retry(state, input);
        let disposition = transition.disposition();
        self.state = Some(transition.state().clone());
        self.effects.extend_from_slice(transition.effects());
        disposition
    }
    fn take_effects(&mut self) -> Vec<RetryEffect> {
        std::mem::take(&mut self.effects)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct CaptureService;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RecognitionService;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ProcessingService;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct DeliveryService;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct HistoryMappingService;

#[allow(clippy::unused_self)]
impl CaptureService {
    fn start(
        self,
        audio: &mut dyn AudioCapturePort,
        correlation: LiveCorrelation,
        max_duration: CaptureLimit,
        cancellation_token: voice_core::CancellationTokenId,
    ) -> LiveEvent {
        let result = audio.start(AudioStartRequest {
            session_id: correlation.session_id(),
            max_duration: max_duration.duration(),
            cancellation_token,
        });
        if result.is_ok() {
            LiveEvent::CaptureStarted(correlation)
        } else {
            LiveEvent::CaptureFailed {
                correlation,
                audio: None,
            }
        }
    }

    fn stop(
        self,
        audio: &mut dyn AudioCapturePort,
        correlation: LiveCorrelation,
    ) -> Option<LiveEvent> {
        audio
            .stop(AudioStopRequest {
                session_id: correlation.session_id(),
            })
            .err()
            .map(|_| LiveEvent::CaptureFailed {
                correlation,
                audio: None,
            })
    }
}

#[allow(clippy::unused_self)]
impl RecognitionService {
    fn live(
        self,
        recognition: &mut dyn RecognitionEnginePort,
        correlation: RecognitionCorrelation,
        audio: voice_core::RecordedAudio,
        timeout: DurationLimit,
        cancellation_token: voice_core::CancellationTokenId,
    ) -> Option<LiveEvent> {
        recognition
            .recognize(RecognitionRequest::Live {
                correlation,
                audio,
                timeout,
                cancellation_token,
            })
            .err()
            .map(|failure| LiveEvent::RecognitionFailed {
                correlation,
                code: voice_core::FailureCode::for_stage(
                    voice_core::FailureStage::Recognition,
                    failure.code(),
                ),
            })
    }

    fn cancel(
        self,
        recognition: &mut dyn RecognitionEnginePort,
        token: voice_core::CancellationTokenId,
    ) {
        let _ = recognition.cancel(token);
    }

    fn retry(
        self,
        recognition: &mut dyn RecognitionEnginePort,
        correlation: voice_core::RetryCorrelation,
        audio: voice_core::RecordedAudio,
        timeout: DurationLimit,
        cancellation_token: voice_core::CancellationTokenId,
    ) -> bool {
        recognition
            .recognize(RecognitionRequest::Retry {
                correlation,
                audio,
                timeout,
                cancellation_token,
            })
            .is_err()
    }
}

#[allow(clippy::unused_self)]
impl ProcessingService {
    fn process(
        self,
        processor: &mut dyn TextProcessorPort,
        correlation: LiveCorrelation,
        plan: ProcessingPlan,
        cancellation_token: voice_core::CancellationTokenId,
    ) -> LiveEvent {
        match processor.process(ProcessingRequest {
            correlation,
            plan,
            cancellation_token,
        }) {
            Ok(result) => LiveEvent::ProcessingSucceeded {
                correlation,
                result,
            },
            Err(failure) => LiveEvent::ProcessingFailed {
                correlation,
                code: voice_core::FailureCode::for_stage(
                    voice_core::FailureStage::Processing,
                    failure.code(),
                ),
            },
        }
    }
}

#[allow(clippy::unused_self)]
impl DeliveryService {
    fn insertion(
        self,
        ports: &mut ApplicationPorts,
        correlation: voice_core::DeliveryOperationCorrelation,
        target: InsertionTarget,
        final_text: voice_core::FinalText,
    ) -> LiveEvent {
        let valid = ports.target_validator.validate(&target).unwrap_or(false);
        if !valid {
            return LiveEvent::TargetInvalidated(correlation.live());
        }
        match ports.injector.insert(InsertionRequest {
            correlation,
            target,
            final_text,
        }) {
            Ok(InjectionDisposition::Confirmed) => {
                LiveEvent::InsertionSucceededForOperation(correlation)
            }
            Ok(InjectionDisposition::DefiniteFailure) => {
                LiveEvent::InsertionFailedForOperation(correlation)
            }
            Ok(InjectionDisposition::Uncertain) => {
                LiveEvent::InsertionUncertainForOperation(correlation)
            }
            Err(failure) if failure.certainty() == DeliveryCertainty::Uncertain => {
                LiveEvent::InsertionUncertainForOperation(correlation)
            }
            Err(_) => LiveEvent::InsertionFailedForOperation(correlation),
        }
    }
}

#[allow(clippy::unused_self)]
impl HistoryMappingService {
    fn persist(
        self,
        history: &mut dyn HistoryStorePort,
        correlation: LiveCorrelation,
        operation_id: voice_core::OperationId,
        recovery_id: voice_core::RecoveryId,
        record: DictationRecord,
    ) -> LiveEvent {
        match history.persist(HistoryPersistRequest {
            record_id: record.id(),
            record,
        }) {
            Ok(report) => LiveEvent::PersistenceSucceededForOperation {
                correlation,
                operation_id,
                recovery_id,
                report,
            },
            Err(_) => LiveEvent::PersistenceFailedForOperation {
                correlation,
                operation_id,
                recovery_id,
            },
        }
    }
}

pub struct ApplicationSupervisor {
    ports: ApplicationPorts,
    identifiers: Box<dyn IdentifierSource>,
    clock: Box<dyn ClockPort>,
    cancellation: Box<dyn CancellationPort>,
    live: LiveSessionSupervisor,
    retry: RetryService,
    recovery: RecoveryService,
    capture_service: CaptureService,
    recognition_service: RecognitionService,
    processing_service: ProcessingService,
    delivery_service: DeliveryService,
    history_mapping_service: HistoryMappingService,
    guard: WorkGuard,
}

impl ApplicationSupervisor {
    #[must_use]
    pub fn new(
        ports: ApplicationPorts,
        identifiers: Box<dyn IdentifierSource>,
        clock: Box<dyn ClockPort>,
        cancellation: Box<dyn CancellationPort>,
    ) -> Self {
        Self {
            ports,
            identifiers,
            clock,
            cancellation,
            live: LiveSessionSupervisor::new(),
            retry: RetryService::default(),
            recovery: RecoveryService::default(),
            capture_service: CaptureService,
            recognition_service: RecognitionService,
            processing_service: ProcessingService,
            delivery_service: DeliveryService,
            history_mapping_service: HistoryMappingService,
            guard: WorkGuard::default(),
        }
    }

    #[must_use]
    pub const fn live_state(&self) -> &LiveState {
        self.live.state()
    }
    #[must_use]
    pub const fn retry_state(&self) -> Option<&RetryState> {
        self.retry.state()
    }
    #[must_use]
    pub const fn work_guard(&self) -> WorkGuard {
        self.guard
    }
    #[must_use]
    pub fn recoveries(&self) -> &[RecoveryContext] {
        self.recovery.contexts()
    }
    #[must_use]
    pub fn pending_live_effects(&self) -> &[LiveEffect] {
        self.live.effects()
    }
    #[must_use]
    pub fn pending_retry_effects(&self) -> &[RetryEffect] {
        self.retry.effects()
    }

    pub fn register_shortcuts(&mut self) -> (PortResult<()>, PortResult<()>) {
        (
            self.ports.shortcuts.register(StartMode::PushToTalk),
            self.ports.shortcuts.register(StartMode::Toggle),
        )
    }

    pub fn start_live(&mut self, mode: StartMode, config: LiveStartConfig) -> EventDisposition {
        self.sync_guard();
        if !self.guard.can_start_live() {
            return EventDisposition::Ignored(voice_core::RejectReason::CompetingWork);
        }
        let session_id = match self.identifiers.next_session_id() {
            Ok(id) => id,
            Err(AllocationError::Exhausted) => {
                return EventDisposition::Ignored(voice_core::RejectReason::AllocationExhausted);
            }
        };
        let record_id = match self.identifiers.next_record_id() {
            Ok(id) => id,
            Err(AllocationError::Exhausted) => {
                return EventDisposition::Ignored(voice_core::RejectReason::AllocationExhausted);
            }
        };
        let recognition_attempt_id = match self.identifiers.next_attempt_id() {
            Ok(id) => id,
            Err(AllocationError::Exhausted) => {
                return EventDisposition::Ignored(voice_core::RejectReason::AllocationExhausted);
            }
        };
        let recovery_id = match self.identifiers.next_recovery_id() {
            Ok(id) => id,
            Err(AllocationError::Exhausted) => {
                return EventDisposition::Ignored(voice_core::RejectReason::AllocationExhausted);
            }
        };
        let cancellation_token = match self.cancellation.allocate() {
            Ok(token) => token,
            Err(AllocationError::Exhausted) => {
                return EventDisposition::Ignored(voice_core::RejectReason::AllocationExhausted);
            }
        };
        if let LiveState::Terminal(session) = self.live.state() {
            if let Some(context) = session.recovery().cloned() {
                self.recovery.archive(context);
            }
            self.live.state = LiveState::Idle;
        }
        let request = StartRequest {
            session_id,
            record_id,
            max_duration: config.max_duration,
            recognition_timeout: config.recognition_timeout,
            started_at: self.clock.now(),
            cancellation_token,
            recovery_id,
            recognition_attempt_id,
            recognition_configuration_id: config.recognition_configuration_id,
            processing_plan: config.processing_plan,
        };
        let disposition = self
            .live
            .submit(LiveInput::Command(LiveCommand::Start { mode, request }));
        self.route_live_effects();
        self.sync_guard();
        disposition
    }

    pub fn submit_live(&mut self, input: LiveInput) -> EventDisposition {
        if matches!(&input, LiveInput::Command(LiveCommand::Start { .. })) {
            self.sync_guard();
            if !self.guard.can_start_live() {
                return EventDisposition::Ignored(voice_core::RejectReason::CompetingWork);
            }
        }
        let disposition = self.live.submit(input);
        self.route_live_effects();
        self.sync_guard();
        disposition
    }

    /// Retry one pending capture cleanup obligation, if any.
    pub fn retry_capture_cleanup(&mut self) -> EventDisposition {
        let Some((correlation, token)) = self.live.state().session().and_then(|session| {
            session
                .pending_cleanup()
                .map(|correlation| (correlation, session.cancellation_token()))
        }) else {
            return EventDisposition::Ignored(voice_core::RejectReason::StaleRevision);
        };
        let disposition = self.route_capture_cleanup(correlation, token);
        self.route_live_effects();
        self.sync_guard();
        disposition
    }

    pub fn tick(&mut self) -> EventDisposition {
        if let Some(session) = self.live.state().session() {
            match session.phase() {
                Phase::Capturing if self.clock.now() >= session.capture_deadline() => {
                    let correlation = LiveCorrelation::new(
                        session.session_id(),
                        session.session_revision(),
                        Phase::Capturing,
                    );
                    return self.submit_live(LiveInput::Command(
                        LiveCommand::CaptureDeadlineReached(correlation),
                    ));
                }
                Phase::Recognizing => {
                    if let Some(deadline) = session.recognition_deadline()
                        && self.clock.now() >= deadline
                    {
                        let correlation = RecognitionCorrelation::new(
                            LiveCorrelation::new(
                                session.session_id(),
                                session.session_revision(),
                                Phase::Recognizing,
                            ),
                            session.attempt_id(),
                            session.attempt_revision(),
                        );
                        return self.submit_live(LiveInput::Command(
                            LiveCommand::RecognitionDeadlineReached(correlation),
                        ));
                    }
                }
                _ => {}
            }
        }
        if let Some(state) = self.retry.state()
            && let (Some(deadline), Some(correlation)) = (state.deadline(), state.active())
            && self.clock.now() >= deadline
            && correlation.expected_retry_phase() == voice_core::RetryPhase::Recognizing
        {
            return self.submit_retry(RetryInput::Event(RetryEvent::RecognitionTimedOut(
                correlation,
            )));
        }
        EventDisposition::Ignored(voice_core::RejectReason::UnexpectedPhase)
    }

    pub fn persist_recovery(&mut self, correlation: RecoveryCorrelation) -> EventDisposition {
        if let Some(context) = self.recovery.find(correlation).cloned() {
            let result = self
                .ports
                .history
                .persist_recovery(voice_ports::RecoveryPersistRequest {
                    correlation,
                    record: context.record().clone(),
                });
            return self.finish_archived_recovery(correlation, result);
        }
        if let LiveState::Terminal(_) = self.live.state() {
            let context = self
                .live
                .state()
                .session()
                .and_then(|session| session.recovery())
                .filter(|context| {
                    context.id() == correlation.recovery_id()
                        && context.record_id() == correlation.record_id()
                        && context.session_id() == correlation.session_id()
                        && !context.is_closed()
                })
                .cloned();
            let Some(context) = context else {
                return EventDisposition::Ignored(voice_core::RejectReason::StaleRecovery);
            };
            let result = self
                .ports
                .history
                .persist_recovery(voice_ports::RecoveryPersistRequest {
                    correlation,
                    record: context.record().clone(),
                });
            let event = match result {
                Ok(report) => {
                    self.live
                        .submit(LiveInput::Event(LiveEvent::RecoveryPersistenceSucceeded {
                            recovery: correlation,
                            report,
                        }))
                }
                Err(_) => self
                    .live
                    .submit(LiveInput::Event(LiveEvent::RecoveryPersistenceFailed(
                        correlation,
                    ))),
            };
            self.sync_guard();
            return event;
        }
        EventDisposition::Ignored(voice_core::RejectReason::StaleRecovery)
    }

    fn finish_archived_recovery(
        &mut self,
        correlation: RecoveryCorrelation,
        result: PortResult<voice_core::PersistenceReport>,
    ) -> EventDisposition {
        match result {
            Ok(report) if self.recovery.close_success(correlation, &report) => {
                EventDisposition::Applied
            }
            Ok(_) | Err(_) => EventDisposition::Applied,
        }
    }

    pub fn retry_recognition(
        &mut self,
        record: DictationRecord,
        config: RetryStartConfig,
    ) -> EventDisposition {
        self.sync_guard();
        if !self.guard.can_start_retry() {
            return EventDisposition::Ignored(voice_core::RejectReason::CompetingWork);
        }
        let token = match self.cancellation.allocate() {
            Ok(token) => token,
            Err(AllocationError::Exhausted) => {
                return EventDisposition::Ignored(voice_core::RejectReason::AllocationExhausted);
            }
        };
        let attempt_id = match self.identifiers.next_attempt_id() {
            Ok(id) => id,
            Err(AllocationError::Exhausted) => {
                return EventDisposition::Ignored(voice_core::RejectReason::AllocationExhausted);
            }
        };
        let recovery_id = match self.identifiers.next_recovery_id() {
            Ok(id) => id,
            Err(AllocationError::Exhausted) => {
                return EventDisposition::Ignored(voice_core::RejectReason::AllocationExhausted);
            }
        };
        let state = RetryState::new(record);
        self.retry.state = Some(state);
        let disposition = self.retry.submit(RetryInput::Command(RetryCommand::Begin {
            attempt_id,
            configuration_id: config.recognition_configuration_id,
            timeout: config.timeout,
            cancellation_token: token,
            recovery_id,
            started_at: self.clock.now(),
        }));
        if !matches!(disposition, EventDisposition::Applied) {
            self.retry.state = None;
            self.retry.effects.clear();
        }
        self.route_retry_effects();
        self.sync_guard();
        disposition
    }

    pub fn submit_retry(&mut self, input: RetryInput) -> EventDisposition {
        if matches!(&input, RetryInput::Command(RetryCommand::Begin { .. })) {
            self.sync_guard();
            if !self.guard.can_start_retry() {
                return EventDisposition::Ignored(voice_core::RejectReason::CompetingWork);
            }
        }
        let disposition = self.retry.submit(input);
        self.route_retry_effects();
        self.sync_guard();
        disposition
    }

    fn sync_guard(&mut self) {
        self.guard.live = self.live.state().is_active()
            || self
                .live
                .state()
                .session()
                .is_some_and(|session| session.pending_cleanup().is_some());
        self.guard.retry = self.retry.state().and_then(RetryState::active).is_some();
        if self.guard.live {
            self.guard.retry = false;
        }
    }

    fn route_live_effects(&mut self) {
        let mut queue = VecDeque::new();
        queue.extend(self.live.take_effects());
        while let Some(effect) = queue.pop_front() {
            self.route_live_effect(effect);
            queue.extend(self.live.take_effects());
        }
    }

    fn route_live_effect(&mut self, effect: LiveEffect) {
        match effect {
            LiveEffect::StartCapture {
                correlation,
                max_duration,
                cancellation_token,
            } => self.route_capture_start(correlation, max_duration, cancellation_token),
            LiveEffect::StopCapture(correlation) => self.route_capture_stop(correlation),
            LiveEffect::CancelCapture {
                correlation,
                cancellation_token,
            } => self.route_capture_cancel(correlation, cancellation_token),
            LiveEffect::CleanupCapture {
                correlation,
                cancellation_token,
            } => {
                let _ = self.route_capture_cleanup(correlation, cancellation_token);
            }
            LiveEffect::RetryCaptureCleanup { .. } | LiveEffect::NotifyUnsavedHistory(_) => {}
            LiveEffect::DiscardCaptureAudio { session_id } => {
                self.route_capture_discard(session_id);
            }
            LiveEffect::Cancel(token) => self.route_cancel(token),
            LiveEffect::ResolveTarget { correlation } => self.route_target_resolution(correlation),
            LiveEffect::StartRecognition {
                correlation,
                audio,
                timeout,
                deadline: _,
                cancellation_token,
            } => self.route_live_recognition(correlation, audio, timeout, cancellation_token),
            LiveEffect::StartProcessing {
                correlation,
                plan,
                cancellation_token,
            } => self.route_processing(correlation, plan, cancellation_token),
            LiveEffect::BeginInsertion {
                correlation,
                target,
                final_text,
            } => self.route_insertion(correlation, target, final_text),
            LiveEffect::PresentResultPanel {
                correlation,
                final_text,
            } => self.route_result_panel(correlation, final_text),
            LiveEffect::CopyToClipboard {
                correlation,
                final_text,
            } => self.route_clipboard(correlation, final_text),
            LiveEffect::PersistRecord {
                correlation,
                operation_id,
                recovery_id,
                record,
            } => self.route_persist_record(correlation, operation_id, recovery_id, record),
        }
    }

    fn route_capture_start(
        &mut self,
        correlation: LiveCorrelation,
        max_duration: CaptureLimit,
        cancellation_token: voice_core::CancellationTokenId,
    ) {
        let event = self.capture_service.start(
            &mut *self.ports.audio,
            correlation,
            max_duration,
            cancellation_token,
        );
        self.live.submit(LiveInput::Event(event));
    }

    fn route_capture_stop(&mut self, correlation: LiveCorrelation) {
        if let Some(event) = self
            .capture_service
            .stop(&mut *self.ports.audio, correlation)
        {
            self.live.submit(LiveInput::Event(event));
        }
    }

    fn route_capture_cancel(
        &mut self,
        correlation: LiveCorrelation,
        cancellation_token: voice_core::CancellationTokenId,
    ) {
        let _ = self.ports.audio.cancel(AudioStopRequest {
            session_id: correlation.session_id(),
        });
        let _ = self.cancellation.cancel(cancellation_token);
    }

    fn route_capture_cleanup(
        &mut self,
        correlation: LiveCorrelation,
        cancellation_token: voice_core::CancellationTokenId,
    ) -> EventDisposition {
        let audio_cancelled = self
            .ports
            .audio
            .cancel(AudioStopRequest {
                session_id: correlation.session_id(),
            })
            .is_ok();
        let audio_discarded = self.ports.audio.discard(correlation.session_id()).is_ok();
        let cancellation_cancelled = self.cancellation.cancel(cancellation_token).is_ok();
        let disposition = self
            .live
            .submit(LiveInput::Event(LiveEvent::CaptureCleanupCompleted {
                correlation,
                audio_cancelled,
                audio_discarded,
                cancellation_cancelled,
            }));
        self.sync_guard();
        disposition
    }

    fn route_capture_discard(&mut self, session_id: voice_core::SessionId) {
        let _ = self.ports.audio.discard(session_id);
    }

    fn route_cancel(&mut self, token: voice_core::CancellationTokenId) {
        let _ = self.cancellation.cancel(token);
        self.recognition_service
            .cancel(&mut *self.ports.recognition, token);
        let _ = self.ports.processor.cancel(token);
    }

    fn route_target_resolution(&mut self, correlation: voice_core::TargetOperationCorrelation) {
        let resolution = self
            .ports
            .target_resolver
            .resolve(TargetRequest { correlation })
            .unwrap_or(voice_core::TargetResolution::Ineligible);
        self.live
            .submit(LiveInput::Event(LiveEvent::TargetResolvedForOperation {
                correlation,
                resolution,
            }));
    }

    fn route_live_recognition(
        &mut self,
        correlation: RecognitionCorrelation,
        audio: voice_core::RecordedAudio,
        timeout: DurationLimit,
        cancellation_token: voice_core::CancellationTokenId,
    ) {
        if let Some(event) = self.recognition_service.live(
            &mut *self.ports.recognition,
            correlation,
            audio,
            timeout,
            cancellation_token,
        ) {
            self.live.submit(LiveInput::Event(event));
        }
    }

    fn route_processing(
        &mut self,
        correlation: LiveCorrelation,
        plan: ProcessingPlan,
        cancellation_token: voice_core::CancellationTokenId,
    ) {
        let event = self.processing_service.process(
            &mut *self.ports.processor,
            correlation,
            plan,
            cancellation_token,
        );
        self.live.submit(LiveInput::Event(event));
    }

    fn route_insertion(
        &mut self,
        correlation: voice_core::DeliveryOperationCorrelation,
        target: InsertionTarget,
        final_text: voice_core::FinalText,
    ) {
        let event =
            self.delivery_service
                .insertion(&mut self.ports, correlation, target, final_text);
        self.live.submit(LiveInput::Event(event));
    }

    fn route_result_panel(
        &mut self,
        correlation: voice_core::DeliveryOperationCorrelation,
        final_text: voice_core::FinalText,
    ) {
        let presented = self
            .ports
            .result_panel
            .present(voice_ports::ResultPanelRequest {
                correlation,
                final_text,
            })
            .unwrap_or(false);
        self.live.submit(LiveInput::Event(
            LiveEvent::ResultPanelPresentedForOperation {
                correlation,
                presented,
            },
        ));
    }

    fn route_clipboard(
        &mut self,
        correlation: voice_core::DeliveryOperationCorrelation,
        final_text: voice_core::FinalText,
    ) {
        let copied = self
            .ports
            .clipboard
            .copy(voice_ports::ClipboardRequest {
                correlation,
                final_text,
            })
            .unwrap_or(false);
        self.live
            .submit(LiveInput::Event(LiveEvent::ClipboardFallbackForOperation {
                correlation,
                copied,
            }));
    }

    fn route_persist_record(
        &mut self,
        correlation: LiveCorrelation,
        operation_id: voice_core::OperationId,
        recovery_id: voice_core::RecoveryId,
        record: DictationRecord,
    ) {
        let event = self.history_mapping_service.persist(
            &mut *self.ports.history,
            correlation,
            operation_id,
            recovery_id,
            record,
        );
        self.live.submit(LiveInput::Event(event));
    }

    fn route_retry_effects(&mut self) {
        let mut queue = VecDeque::new();
        queue.extend(self.retry.take_effects());
        while let Some(effect) = queue.pop_front() {
            match effect {
                RetryEffect::PersistAttempt {
                    correlation,
                    record,
                    attempt,
                } => {
                    let result =
                        self.ports
                            .history
                            .persist_retry_attempt(RetryAttemptPersistRequest {
                                correlation,
                                record,
                                attempt,
                            });
                    let event = if result.is_ok() {
                        RetryEvent::AttemptPersistenceSucceeded(correlation)
                    } else {
                        RetryEvent::AttemptPersistenceFailed(correlation)
                    };
                    self.retry.submit(RetryInput::Event(event));
                }
                RetryEffect::StartRecognition {
                    correlation,
                    audio,
                    timeout,
                    cancellation_token,
                    ..
                } => {
                    if self.recognition_service.retry(
                        &mut *self.ports.recognition,
                        correlation,
                        audio,
                        timeout,
                        cancellation_token,
                    ) {
                        self.retry
                            .submit(RetryInput::Event(RetryEvent::RecognitionFailed(
                                correlation,
                            )));
                    }
                }
                RetryEffect::PersistResult {
                    correlation,
                    record,
                    attempt,
                } => {
                    let result =
                        self.ports
                            .history
                            .persist_retry_result(RetryResultPersistRequest {
                                correlation,
                                record,
                                attempt,
                            });
                    let event = if result.is_ok() {
                        RetryEvent::ResultPersistenceSucceeded(correlation)
                    } else {
                        RetryEvent::ResultPersistenceFailed(correlation)
                    };
                    self.retry.submit(RetryInput::Event(event));
                }
                RetryEffect::ArchiveRecovery {
                    recovery_id,
                    record,
                } => {
                    self.recovery
                        .archive(RecoveryContext::new(recovery_id, record));
                }
                RetryEffect::Cancel(token) => {
                    let _ = self.cancellation.cancel(token);
                    self.recognition_service
                        .cancel(&mut *self.ports.recognition, token);
                }
            }
            queue.extend(self.retry.take_effects());
        }
    }
}

/// A focused pure live service useful for UI-independent tests.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LiveSessionService {
    state: LiveState,
    effects: Vec<LiveEffect>,
}
impl LiveSessionService {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: LiveState::Idle,
            effects: Vec::new(),
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
    pub fn submit(&mut self, input: LiveInput) -> EventDisposition {
        let transition = reduce_live(&self.state, input);
        let disposition = transition.disposition();
        let (state, effects, _) = transition.into_parts();
        self.state = state;
        self.effects.extend(effects);
        disposition
    }
    pub fn drain_effects(&mut self) -> Vec<LiveEffect> {
        std::mem::take(&mut self.effects)
    }
}
