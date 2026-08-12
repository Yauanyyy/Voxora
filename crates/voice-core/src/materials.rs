use std::fmt;

use crate::{
    AudioReferenceId, CodeParseError, ConfigurationId, DictationRecordId, RecognitionAttemptId,
    RecoveryId, Revision, SanitizedFailure, SessionId, TargetId, TerminalOutcome, Warning,
};

/// A material is either absent or available with independently tracked durability.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum MaterialState {
    #[default]
    Absent,
    Available(Durability),
}

impl MaterialState {
    #[must_use]
    pub const fn available(self) -> bool {
        !matches!(self, Self::Absent)
    }

    #[must_use]
    pub const fn durability(self) -> Option<Durability> {
        match self {
            Self::Absent => None,
            Self::Available(durability) => Some(durability),
        }
    }

    #[must_use]
    pub const fn non_durable() -> Self {
        Self::Available(Durability::NonDurable)
    }

    #[must_use]
    pub const fn durable() -> Self {
        Self::Available(Durability::Durable)
    }
}

/// Whether an available material has been persisted successfully.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialOrd, Ord, PartialEq)]
pub enum Durability {
    NonDurable,
    Durable,
}

impl Durability {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NonDurable => "non_durable",
            Self::Durable => "durable",
        }
    }

    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "non_durable" => Some(Self::NonDurable),
            "durable" => Some(Self::Durable),
            _ => None,
        }
    }
}

/// Independently observable recoverable materials.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MaterialKind {
    RecordedAudio,
    PartialTranscript,
    RawTranscript,
    ProcessedText,
    FinalText,
    ResultPanel,
    ClipboardFallback,
}

impl MaterialKind {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::RecordedAudio => "recorded_audio",
            Self::PartialTranscript => "partial_transcript",
            Self::RawTranscript => "raw_transcript",
            Self::ProcessedText => "processed_text",
            Self::FinalText => "final_text",
            Self::ResultPanel => "result_panel",
            Self::ClipboardFallback => "clipboard_fallback",
        }
    }

    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "recorded_audio" => Some(Self::RecordedAudio),
            "partial_transcript" => Some(Self::PartialTranscript),
            "raw_transcript" => Some(Self::RawTranscript),
            "processed_text" => Some(Self::ProcessedText),
            "final_text" => Some(MaterialKind::FinalText),
            "result_panel" => Some(MaterialKind::ResultPanel),
            "clipboard_fallback" => Some(MaterialKind::ClipboardFallback),
            _ => None,
        }
    }
}

/// Availability and durability for all M3 materials.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MaterialLedger {
    recorded_audio: MaterialState,
    partial_transcript: MaterialState,
    raw_transcript: MaterialState,
    processed_text: MaterialState,
    final_text: MaterialState,
    result_panel: MaterialState,
    clipboard_fallback: MaterialState,
}

impl MaterialLedger {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            recorded_audio: MaterialState::Absent,
            partial_transcript: MaterialState::Absent,
            raw_transcript: MaterialState::Absent,
            processed_text: MaterialState::Absent,
            final_text: MaterialState::Absent,
            result_panel: MaterialState::Absent,
            clipboard_fallback: MaterialState::Absent,
        }
    }

    #[must_use]
    pub const fn state(&self, kind: MaterialKind) -> MaterialState {
        match kind {
            MaterialKind::RecordedAudio => self.recorded_audio,
            MaterialKind::PartialTranscript => self.partial_transcript,
            MaterialKind::RawTranscript => self.raw_transcript,
            MaterialKind::ProcessedText => self.processed_text,
            MaterialKind::FinalText => self.final_text,
            MaterialKind::ResultPanel => self.result_panel,
            MaterialKind::ClipboardFallback => self.clipboard_fallback,
        }
    }

    pub const fn set(&mut self, kind: MaterialKind, state: MaterialState) {
        match kind {
            MaterialKind::RecordedAudio => self.recorded_audio = state,
            MaterialKind::PartialTranscript => self.partial_transcript = state,
            MaterialKind::RawTranscript => self.raw_transcript = state,
            MaterialKind::ProcessedText => self.processed_text = state,
            MaterialKind::FinalText => self.final_text = state,
            MaterialKind::ResultPanel => self.result_panel = state,
            MaterialKind::ClipboardFallback => self.clipboard_fallback = state,
        }
    }

    pub const fn mark_available(&mut self, kind: MaterialKind) {
        self.set(kind, MaterialState::non_durable());
    }

    /// Mark only currently available materials durable.
    pub const fn mark_durable(&mut self, kinds: &[MaterialKind]) {
        let mut index = 0;
        while index < kinds.len() {
            let kind = kinds[index];
            if self.state(kind).available() {
                self.set(kind, MaterialState::durable());
            }
            index += 1;
        }
    }

    /// Keep all available material explicitly non-durable after a persistence error.
    pub const fn mark_all_non_durable(&mut self) {
        let kinds = Self::all_kinds();
        let mut index = 0;
        while index < kinds.len() {
            let kind = kinds[index];
            if self.state(kind).available() {
                self.set(kind, MaterialState::non_durable());
            }
            index += 1;
        }
    }

    #[must_use]
    pub fn available_kinds(&self) -> Vec<MaterialKind> {
        Self::all_kinds()
            .into_iter()
            .filter(|kind| self.state(*kind).available())
            .collect()
    }

    #[must_use]
    pub const fn all_kinds() -> [MaterialKind; 7] {
        [
            MaterialKind::RecordedAudio,
            MaterialKind::PartialTranscript,
            MaterialKind::RawTranscript,
            MaterialKind::ProcessedText,
            MaterialKind::FinalText,
            MaterialKind::ResultPanel,
            MaterialKind::ClipboardFallback,
        ]
    }

    #[must_use]
    pub fn all_available_durable(&self) -> bool {
        self.available_kinds()
            .iter()
            .all(|kind| self.state(*kind).durability() == Some(Durability::Durable))
    }
}

/// Recorded audio is an opaque artifact reference, not a filesystem path or bytes.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct RecordedAudio {
    reference: AudioReferenceId,
    has_samples: bool,
}

impl RecordedAudio {
    #[must_use]
    pub const fn new(reference: AudioReferenceId, has_samples: bool) -> Self {
        Self {
            reference,
            has_samples,
        }
    }

    #[must_use]
    pub const fn reference(&self) -> AudioReferenceId {
        self.reference
    }

    #[must_use]
    pub const fn has_samples(&self) -> bool {
        self.has_samples
    }
}

impl fmt::Debug for RecordedAudio {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RecordedAudio(<redacted>)")
    }
}

impl fmt::Display for RecordedAudio {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<recorded-audio>")
    }
}

macro_rules! sensitive_text {
    ($name:ident, $label:literal) => {
        #[derive(Clone, Eq, Hash, PartialEq)]
        pub struct $name(String);

        impl $name {
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            #[must_use]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(concat!($label, "(<redacted>)"))
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("<redacted>")
            }
        }
    };
}

sensitive_text!(PartialTranscript, "PartialTranscript");
sensitive_text!(RawTranscript, "RawTranscript");
sensitive_text!(ProcessedText, "ProcessedText");
sensitive_text!(FinalText, "FinalText");
sensitive_text!(TargetToken, "TargetToken");
sensitive_text!(ApplicationIdentity, "ApplicationIdentity");
sensitive_text!(CredentialSecret, "CredentialSecret");

/// A resolved insertion target, represented only by an opaque token and identity.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct InsertionTarget {
    id: TargetId,
    token: TargetToken,
    application: Option<ApplicationIdentity>,
}

impl InsertionTarget {
    #[must_use]
    pub fn new(id: TargetId, token: TargetToken, application: Option<ApplicationIdentity>) -> Self {
        Self {
            id,
            token,
            application,
        }
    }

    #[must_use]
    pub const fn id(&self) -> TargetId {
        self.id
    }

    #[must_use]
    pub const fn token(&self) -> &TargetToken {
        &self.token
    }

    #[must_use]
    pub const fn application(&self) -> Option<&ApplicationIdentity> {
        self.application.as_ref()
    }
}

impl fmt::Debug for InsertionTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InsertionTarget")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

/// A target resolution result at capture end.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TargetResolution {
    Eligible(InsertionTarget),
    Ineligible,
}

/// A complete recovery payload.  The record is owned rather than reconstructed
/// from a material ledger, so audio/text/attempts/outcome/warnings/failure all
/// survive a live-state reset and later callback.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryContext {
    id: RecoveryId,
    record: DictationRecord,
    closed: bool,
}

impl RecoveryContext {
    #[must_use]
    pub fn new(id: RecoveryId, record: DictationRecord) -> Self {
        Self {
            id,
            record,
            closed: false,
        }
    }

    #[must_use]
    pub const fn id(&self) -> RecoveryId {
        self.id
    }

    #[must_use]
    pub const fn record(&self) -> &DictationRecord {
        &self.record
    }

    #[must_use]
    pub const fn record_id(&self) -> DictationRecordId {
        self.record.id()
    }

    #[must_use]
    pub const fn session_id(&self) -> SessionId {
        self.record.originating_session_id()
    }

    #[must_use]
    pub const fn materials(&self) -> &MaterialLedger {
        self.record.materials()
    }

    #[must_use]
    pub const fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn mark_durable(&mut self, kinds: &[MaterialKind]) {
        self.record.mark_materials_durable(kinds);
        if self.record.materials().all_available_durable() {
            self.record.mark_durable();
        }
    }

    /// Replace the retained payload after a later manual-preservation result.
    ///
    /// Recovery callbacks remain correlated to the same record/session pair;
    /// only the available material and terminal metadata are refreshed.
    pub fn replace_record(&mut self, record: DictationRecord) {
        if !self.closed
            && record.id() == self.record.id()
            && record.originating_session_id() == self.record.originating_session_id()
        {
            self.record = record;
        }
    }

    pub fn close(&mut self) {
        self.closed = true;
    }
}

/// A history record containing a terminal session and recognition attempts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DictationRecord {
    id: DictationRecordId,
    originating_session_id: SessionId,
    recorded_audio: Option<RecordedAudio>,
    partial_transcript: Option<PartialTranscript>,
    raw_transcript: Option<RawTranscript>,
    processed_text: Option<ProcessedText>,
    final_text: Option<FinalText>,
    attempts: Vec<RecognitionAttempt>,
    materials: MaterialLedger,
    warnings: Vec<Warning>,
    failure: Option<SanitizedFailure>,
    outcome: Option<TerminalOutcome>,
    hotwords_used: u32,
    hotwords_total: u32,
    durable: bool,
}

impl DictationRecord {
    #[must_use]
    pub fn new(id: DictationRecordId, originating_session_id: SessionId) -> Self {
        Self {
            id,
            originating_session_id,
            recorded_audio: None,
            partial_transcript: None,
            raw_transcript: None,
            processed_text: None,
            final_text: None,
            attempts: Vec::new(),
            materials: MaterialLedger::new(),
            warnings: Vec::new(),
            failure: None,
            outcome: None,
            hotwords_used: 0,
            hotwords_total: 0,
            durable: false,
        }
    }

    #[must_use]
    pub const fn id(&self) -> DictationRecordId {
        self.id
    }

    #[must_use]
    pub const fn originating_session_id(&self) -> SessionId {
        self.originating_session_id
    }

    #[must_use]
    pub const fn recorded_audio(&self) -> Option<&RecordedAudio> {
        self.recorded_audio.as_ref()
    }

    #[must_use]
    pub const fn partial_transcript(&self) -> Option<&PartialTranscript> {
        self.partial_transcript.as_ref()
    }

    #[must_use]
    pub const fn raw_transcript(&self) -> Option<&RawTranscript> {
        self.raw_transcript.as_ref()
    }

    #[must_use]
    pub const fn processed_text(&self) -> Option<&ProcessedText> {
        self.processed_text.as_ref()
    }

    #[must_use]
    pub const fn final_text(&self) -> Option<&FinalText> {
        self.final_text.as_ref()
    }

    #[must_use]
    pub fn attempts(&self) -> &[RecognitionAttempt] {
        &self.attempts
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
    pub const fn hotword_usage(&self) -> (u32, u32) {
        (self.hotwords_used, self.hotwords_total)
    }

    #[must_use]
    pub const fn is_durable(&self) -> bool {
        self.durable
    }

    #[must_use]
    pub fn has_usable_durable_audio(&self) -> bool {
        self.recorded_audio
            .as_ref()
            .is_some_and(RecordedAudio::has_samples)
            && self.materials.state(MaterialKind::RecordedAudio)
                == MaterialState::Available(Durability::Durable)
    }

    pub fn append_attempt(&mut self, attempt: RecognitionAttempt) {
        self.attempts.push(attempt);
        self.durable = false;
    }

    /// Replace only the freshly appended retry attempt.  The application and
    /// reducer use this narrow helper so earlier durable attempts remain
    /// immutable.
    pub fn attempts_mut_last_for_m3(&mut self) -> Option<&mut RecognitionAttempt> {
        self.attempts.last_mut()
    }

    pub const fn set_recorded_audio(&mut self, audio: RecordedAudio) {
        self.recorded_audio = Some(audio);
        self.materials
            .set(MaterialKind::RecordedAudio, MaterialState::non_durable());
        self.durable = false;
    }

    pub fn set_partial_transcript(&mut self, partial: PartialTranscript) {
        self.partial_transcript = Some(partial);
        self.materials.set(
            MaterialKind::PartialTranscript,
            MaterialState::non_durable(),
        );
        self.durable = false;
    }

    pub fn clear_partial_transcript(&mut self) {
        self.partial_transcript = None;
        self.materials
            .set(MaterialKind::PartialTranscript, MaterialState::Absent);
    }

    pub fn set_raw_transcript(&mut self, raw: RawTranscript) {
        self.raw_transcript = Some(raw);
        self.materials
            .set(MaterialKind::RawTranscript, MaterialState::non_durable());
        self.durable = false;
    }

    pub fn set_processed_text(&mut self, processed: ProcessedText) {
        self.processed_text = Some(processed);
        self.materials
            .set(MaterialKind::ProcessedText, MaterialState::non_durable());
        self.durable = false;
    }

    pub fn set_final_text(&mut self, final_text: FinalText) {
        self.final_text = Some(final_text);
        self.materials
            .set(MaterialKind::FinalText, MaterialState::non_durable());
        self.durable = false;
    }

    pub const fn set_materials(&mut self, materials: MaterialLedger) {
        self.materials = materials;
        self.durable = false;
    }

    pub const fn set_outcome(&mut self, outcome: TerminalOutcome) {
        self.outcome = Some(outcome);
    }

    pub const fn set_hotword_usage(&mut self, used: u32, total: u32) {
        self.hotwords_total = total;
        self.hotwords_used = if used > total { total } else { used };
    }

    pub fn set_warnings(&mut self, warnings: Vec<Warning>) {
        self.warnings = warnings;
    }

    pub fn add_warning(&mut self, warning: Warning) {
        if !self.warnings.contains(&warning) {
            self.warnings.push(warning);
        }
    }

    pub const fn set_failure(&mut self, failure: Option<SanitizedFailure>) {
        self.failure = failure;
    }

    pub fn mark_materials_durable(&mut self, kinds: &[MaterialKind]) {
        self.materials.mark_durable(kinds);
        self.durable = self.materials.all_available_durable();
    }

    pub fn mark_durable(&mut self) {
        self.durable = true;
    }
}

/// One recognition execution in a record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecognitionAttempt {
    id: RecognitionAttemptId,
    revision: Revision,
    configuration_id: ConfigurationId,
    status: AttemptStatus,
    raw_transcript: Option<RawTranscript>,
    partial_transcript: Option<PartialTranscript>,
    failure: Option<SanitizedFailure>,
}

impl RecognitionAttempt {
    #[must_use]
    pub const fn new(
        id: RecognitionAttemptId,
        revision: Revision,
        configuration_id: ConfigurationId,
    ) -> Self {
        Self {
            id,
            revision,
            configuration_id,
            status: AttemptStatus::Pending,
            raw_transcript: None,
            partial_transcript: None,
            failure: None,
        }
    }

    #[must_use]
    pub const fn id(&self) -> RecognitionAttemptId {
        self.id
    }

    #[must_use]
    pub const fn revision(&self) -> Revision {
        self.revision
    }

    #[must_use]
    pub const fn configuration_id(&self) -> ConfigurationId {
        self.configuration_id
    }

    #[must_use]
    pub const fn status(&self) -> AttemptStatus {
        self.status
    }

    #[must_use]
    pub const fn raw_transcript(&self) -> Option<&RawTranscript> {
        self.raw_transcript.as_ref()
    }

    #[must_use]
    pub const fn partial_transcript(&self) -> Option<&PartialTranscript> {
        self.partial_transcript.as_ref()
    }

    #[must_use]
    pub const fn failure(&self) -> Option<&SanitizedFailure> {
        self.failure.as_ref()
    }

    pub fn accept_partial(&mut self, partial: PartialTranscript) {
        self.partial_transcript = Some(partial);
    }

    pub fn clear_partial(&mut self) {
        self.partial_transcript = None;
    }

    pub fn accept_final(&mut self, raw: RawTranscript) {
        self.raw_transcript = Some(raw);
        self.partial_transcript = None;
        self.status = AttemptStatus::Succeeded;
    }

    pub fn fail(&mut self, failure: SanitizedFailure) {
        self.failure = Some(failure);
        self.status = AttemptStatus::Failed;
    }

    /// Restore persisted attempt state using only validated portable values.
    #[must_use]
    pub fn restore(
        id: RecognitionAttemptId,
        revision: Revision,
        configuration_id: ConfigurationId,
        status: AttemptStatus,
        raw_transcript: Option<RawTranscript>,
        partial_transcript: Option<PartialTranscript>,
        failure: Option<SanitizedFailure>,
    ) -> Self {
        Self {
            id,
            revision,
            configuration_id,
            status,
            raw_transcript,
            partial_transcript,
            failure,
        }
    }
}

/// Attempt status is separate from the five session terminal outcomes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AttemptStatus {
    Pending,
    Succeeded,
    Failed,
}

impl AttemptStatus {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }

    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "pending" => Some(Self::Pending),
            "succeeded" => Some(Self::Succeeded),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

impl fmt::Display for AttemptStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::str::FromStr for AttemptStatus {
    type Err = CodeParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::from_code(value).ok_or(CodeParseError)
    }
}
