#![deny(unsafe_code)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::collapsible_if,
    clippy::items_after_statements,
    clippy::manual_let_else,
    clippy::map_unwrap_or,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::needless_pass_by_value,
    clippy::redundant_closure_for_method_calls,
    clippy::single_match_else,
    clippy::too_many_lines,
    clippy::type_complexity,
    clippy::unused_self
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, Transaction, params};
use voice_core::{
    ApplicationProfile, ApplicationProfileId, AttemptStatus, AudioReferenceId, BaseUrl,
    BuiltInPromptId, BuiltInRuleId, ConfigurationId, CredentialReferenceId, DictationRecord,
    DictationRecordId, DurationLimit, FailureCode, FailureStage, Hotword, HotwordGroup,
    HotwordGroupId, HotwordId, LanguageModelConfiguration, MaterialKind, PartialTranscript,
    PersistenceReport, ProcessedText, ProcessingOrder, ProcessingRuleId,
    ProcessingStepConfiguration, PromptPreset, PromptPresetId, PromptShortcut, RawTranscript,
    ReasoningMode, RecognitionAttempt, RecognitionAttemptId, RecognitionConfiguration,
    RecordedAudio, RetentionPolicy, Revision, RuleOverride, SanitizedFailure, SessionId,
    TerminalOutcome, Timestamp, Warning,
};
use voice_ports::{
    AudioArtifactStorePort, AudioMaintenanceReport, ConfigurationStorePort, HistoryDeletionReport,
    HistoryMaintenancePort, HistoryStorePort, LibraryStorePort, PortResult,
    ProcessingConfigurationPort, PromptDeleteReport, PromptStorePort, RetentionReport,
};

const SCHEMA_VERSION: i64 = 2;

#[derive(Debug)]
pub enum SqliteError {
    Database,
    Migration,
    UnsupportedSchema,
    InvalidArtifactName,
    Filesystem,
}

impl SqliteError {
    fn failure(&self) -> voice_core::SanitizedFailure {
        voice_core::SanitizedFailure::from_boundary(
            voice_core::FailureStage::Persistence,
            voice_core::FailureCode::PersistenceUnavailable,
            voice_core::RetryMeaning::Retryable,
            voice_core::DeliveryCertainty::NotApplicable,
        )
    }
}

pub struct HistorySqlite {
    connection: Connection,
    database_path: Option<PathBuf>,
    root: PathBuf,
}

impl HistorySqlite {
    pub fn open(
        path: impl AsRef<Path>,
        artifact_root: impl AsRef<Path>,
    ) -> Result<Self, SqliteError> {
        let path = path.as_ref();
        let database_path = (path != Path::new(":memory:")).then(|| path.to_owned());
        let connection = Connection::open(path).map_err(|_| SqliteError::Database)?;
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(|_| SqliteError::Database)?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|_| SqliteError::Database)?;
        let mut adapter = Self {
            connection,
            database_path,
            root: artifact_root.as_ref().to_owned(),
        };
        adapter.migrate()?;
        fs::create_dir_all(adapter.temporary_dir()).map_err(|_| SqliteError::Filesystem)?;
        fs::create_dir_all(adapter.committed_dir()).map_err(|_| SqliteError::Filesystem)?;
        Ok(adapter)
    }

    fn migrate(&mut self) -> Result<(), SqliteError> {
        let tx = self
            .connection
            .transaction()
            .map_err(|_| SqliteError::Migration)?;
        tx.execute_batch("CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL);")
            .map_err(|_| SqliteError::Migration)?;
        let current: Option<i64> = tx
            .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
                row.get(0)
            })
            .optional()
            .map_err(|_| SqliteError::Migration)?;
        if current.is_some_and(|version| version > SCHEMA_VERSION) {
            return Err(SqliteError::UnsupportedSchema);
        }
        match current.unwrap_or(0) {
            0 => {
                tx.execute_batch(include_str!("../migrations/0001_initial.sql"))
                    .map_err(|_| SqliteError::Migration)?;
                tx.execute_batch(include_str!("../migrations/0002_seed_defaults.sql"))
                    .map_err(|_| SqliteError::Migration)?;
                tx.execute(
                    "INSERT INTO schema_version(version) VALUES (?)",
                    [SCHEMA_VERSION],
                )
                .map_err(|_| SqliteError::Migration)?;
            }
            1 => {
                tx.execute_batch(include_str!("../migrations/0001_initial.sql"))
                    .map_err(|_| SqliteError::Migration)?;
                tx.execute_batch(include_str!("../migrations/0002_seed_defaults.sql"))
                    .map_err(|_| SqliteError::Migration)?;
                tx.execute("UPDATE schema_version SET version = ?", [SCHEMA_VERSION])
                    .map_err(|_| SqliteError::Migration)?;
            }
            SCHEMA_VERSION => {}
            _ => return Err(SqliteError::Migration),
        }
        tx.commit().map_err(|_| SqliteError::Migration)
    }

    fn temporary_dir(&self) -> PathBuf {
        self.root.join("temporary")
    }
    fn committed_dir(&self) -> PathBuf {
        self.root.join("committed")
    }
    fn backup_temporary_path(&self) -> PathBuf {
        self.temporary_dir().join("sqlite-backup.tmp")
    }
    fn artifact_name(reference: AudioReferenceId) -> String {
        format!("audio-{}.bin", reference.get())
    }
    fn checked_path(&self, directory: &Path, reference: AudioReferenceId) -> PathBuf {
        directory.join(Self::artifact_name(reference))
    }

    fn committed_artifact_exists(&self, reference: AudioReferenceId) -> bool {
        let metadata_exists = self
            .connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM audio_artifacts WHERE audio_ref=? AND artifact_name=? AND nonempty=1)",
                params![reference.to_string(), Self::artifact_name(reference)],
                |row| row.get::<_, bool>(0),
            )
            .unwrap_or(false);
        metadata_exists
            && fs::metadata(self.checked_path(&self.committed_dir(), reference))
                .is_ok_and(|metadata| metadata.is_file() && metadata.len() > 0)
    }

    fn parse_reference(value: &str) -> Result<AudioReferenceId, SqliteError> {
        if value.contains('/') || value.contains('\\') || value.contains("..") {
            return Err(SqliteError::InvalidArtifactName);
        }
        value.parse().map_err(|_| SqliteError::InvalidArtifactName)
    }

    fn parse_artifact_name(value: &str) -> Result<AudioReferenceId, SqliteError> {
        let number = value
            .strip_prefix("audio-")
            .and_then(|value| value.strip_suffix(".bin"))
            .ok_or(SqliteError::InvalidArtifactName)?;
        if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(SqliteError::InvalidArtifactName);
        }
        AudioReferenceId::new(
            number
                .parse::<u64>()
                .map_err(|_| SqliteError::InvalidArtifactName)?,
        )
        .ok_or(SqliteError::InvalidArtifactName)
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    fn now_millis() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
            .unwrap_or(0)
    }

    fn load_retention_inner(&self) -> Result<RetentionPolicy, rusqlite::Error> {
        let value: Option<(i64, i64, i64, i64)> = self
            .connection
            .query_row(
                "SELECT text_enabled,audio_enabled,text_days,audio_days FROM retention_policy WHERE id=1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()?;
        Ok(value.map_or_else(
            RetentionPolicy::default,
            |(text, audio, text_days, audio_days)| RetentionPolicy {
                text_enabled: text != 0,
                audio_enabled: audio != 0,
                text_days: text_days.max(0) as u32,
                audio_days: audio_days.max(0) as u32,
            },
        ))
    }

    fn process_deletion_queue(&mut self) -> Result<(u64, u64), SqliteError> {
        let rows: Vec<(i64, String)> = {
            let mut statement = self
                .connection
                .prepare("SELECT id,artifact_name FROM artifact_deletion_queue ORDER BY id")
                .map_err(|_| SqliteError::Database)?;
            let mapped = statement
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .map_err(|_| SqliteError::Database)?;
            mapped
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| SqliteError::Database)?
        };
        let mut deleted = 0;
        for (id, value) in rows {
            let reference = match Self::parse_artifact_name(&value) {
                Ok(reference) => reference,
                Err(_) => {
                    self.connection
                        .execute(
                            "UPDATE artifact_deletion_queue SET attempts=attempts+1 WHERE id=?",
                            [id],
                        )
                        .map_err(|_| SqliteError::Database)?;
                    continue;
                }
            };
            let path = self.checked_path(&self.committed_dir(), reference);
            let result = if path.exists() {
                fs::remove_file(path)
            } else {
                Ok(())
            };
            match result {
                Ok(()) => {
                    self.connection
                        .execute("DELETE FROM artifact_deletion_queue WHERE id=?", [id])
                        .map_err(|_| SqliteError::Database)?;
                    deleted += 1;
                }
                Err(_) => {
                    self.connection
                        .execute(
                            "UPDATE artifact_deletion_queue SET attempts=attempts+1 WHERE id=?",
                            [id],
                        )
                        .map_err(|_| SqliteError::Database)?;
                }
            }
        }
        let remaining: u64 = self
            .connection
            .query_row("SELECT COUNT(*) FROM artifact_deletion_queue", [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|_| SqliteError::Database)? as u64;
        Ok((deleted, remaining))
    }

    fn queue_audio(
        tx: &Transaction<'_>,
        reference: AudioReferenceId,
    ) -> Result<(), rusqlite::Error> {
        tx.execute(
            "DELETE FROM audio_artifacts WHERE audio_ref=?",
            [reference.to_string()],
        )?;
        tx.execute(
            "INSERT OR IGNORE INTO artifact_deletion_queue(artifact_name) VALUES(?)",
            [Self::artifact_name(reference)],
        )?;
        Ok(())
    }

    fn save_record(
        &mut self,
        record: &DictationRecord,
        force_recovery: bool,
    ) -> Result<PersistenceReport, SqliteError> {
        let retention = self
            .load_retention_inner()
            .map_err(|_| SqliteError::Database)?;
        let failure = record.failure();
        let capture_boundary = failure.is_some_and(|value| value.stage() == FailureStage::Capture);
        let recovery_required = force_recovery
            || matches!(
                record.outcome(),
                Some(
                    voice_core::TerminalOutcome::Failed
                        | voice_core::TerminalOutcome::Cancelled
                        | voice_core::TerminalOutcome::ManualDeliveryRequired
                        | voice_core::TerminalOutcome::DeliveryUncertain
                )
            );
        let retain_text = !capture_boundary && (recovery_required || retention.text_enabled);
        let retain_audio = recovery_required || retention.audio_enabled;
        let available_audio_ref = record
            .recorded_audio()
            .filter(|audio| audio.has_samples())
            .map(|audio| audio.reference());
        let available_audio_durable =
            available_audio_ref.is_some_and(|reference| self.committed_artifact_exists(reference));
        if !recovery_required && !retain_text && !retain_audio {
            if let Some(reference) = available_audio_ref.filter(|_| available_audio_durable) {
                let tx = self
                    .connection
                    .transaction()
                    .map_err(|_| SqliteError::Database)?;
                Self::queue_audio(&tx, reference).map_err(|_| SqliteError::Database)?;
                tx.commit().map_err(|_| SqliteError::Database)?;
                let _ = self.process_deletion_queue()?;
            }
            return Ok(PersistenceReport {
                durable_materials: Vec::new(),
            });
        }
        let old_audio: Option<Option<String>> = self
            .connection
            .query_row(
                "SELECT audio_ref FROM dictation_records WHERE id=?",
                [record.id().get() as i64],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| SqliteError::Database)?;
        let candidate_audio_ref = available_audio_ref.filter(|_| retain_audio);
        let audio_durable = candidate_audio_ref.is_some() && available_audio_durable;
        let audio_ref = candidate_audio_ref.filter(|_| audio_durable);
        let raw = record
            .raw_transcript()
            .filter(|_| retain_text)
            .map(|value| value.as_str());
        let processed = record
            .processed_text()
            .filter(|_| retain_text)
            .map(|value| value.as_str());
        let final_text = record
            .final_text()
            .filter(|_| retain_text)
            .map(|value| value.as_str());
        let partial = record
            .partial_transcript()
            .filter(|_| retain_text)
            .map(|value| value.as_str());
        let durable_materials = [
            (MaterialKind::RawTranscript, raw.is_some()),
            (MaterialKind::ProcessedText, processed.is_some()),
            (MaterialKind::FinalText, final_text.is_some()),
            (MaterialKind::PartialTranscript, partial.is_some()),
            (MaterialKind::RecordedAudio, audio_durable),
        ];
        let retained_any = raw.is_some()
            || processed.is_some()
            || final_text.is_some()
            || partial.is_some()
            || audio_ref.is_some();
        let all_durable = retained_any && (audio_ref.is_none() || audio_durable);
        let tx = self
            .connection
            .transaction()
            .map_err(|_| SqliteError::Database)?;
        tx.execute(
            "INSERT INTO dictation_records(id,session_id,outcome,raw_text,processed_text,final_text,partial_text,audio_ref,audio_durable,durable,created_at,failure_stage,failure_code,failure_retry,failure_certainty,hotwords_used,hotwords_total)
             VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)
             ON CONFLICT(id) DO UPDATE SET session_id=excluded.session_id,outcome=excluded.outcome,raw_text=excluded.raw_text,processed_text=excluded.processed_text,final_text=excluded.final_text,partial_text=excluded.partial_text,audio_ref=excluded.audio_ref,audio_durable=excluded.audio_durable,durable=excluded.durable,failure_stage=excluded.failure_stage,failure_code=excluded.failure_code,failure_retry=excluded.failure_retry,failure_certainty=excluded.failure_certainty,hotwords_used=excluded.hotwords_used,hotwords_total=excluded.hotwords_total",
            params![
                record.id().get() as i64,
                record.originating_session_id().get() as i64,
                record.outcome().map(|value| value.code()),
                raw,
                processed,
                final_text,
                partial,
                audio_ref.map(|value| value.to_string()),
                i64::from(audio_durable),
                i64::from(all_durable),
                Self::now_millis(),
                failure.map(|value| value.stage().code()),
                failure.map(|value| value.code().code()),
                failure.map(|value| value.retry().code()),
                failure.map(|value| value.certainty().code()),
                record.hotword_usage().0,
                record.hotword_usage().1,
            ],
        )
        .map_err(|_| SqliteError::Database)?;
        tx.execute(
            "DELETE FROM dictation_warnings WHERE record_id=?",
            [record.id().get() as i64],
        )
        .map_err(|_| SqliteError::Database)?;
        for warning in record.warnings() {
            tx.execute(
                "INSERT INTO dictation_warnings(record_id,warning_code) VALUES(?,?)",
                params![record.id().get() as i64, warning.code()],
            )
            .map_err(|_| SqliteError::Database)?;
        }
        tx.execute(
            "DELETE FROM recognition_attempts WHERE record_id=?",
            [record.id().get() as i64],
        )
        .map_err(|_| SqliteError::Database)?;
        for attempt in record.attempts() {
            let attempt_failure = attempt.failure().copied();
            tx.execute(
                "INSERT INTO recognition_attempts(record_id,id,revision,configuration_id,status,raw_text,partial_text,failure_stage,failure_code,failure_retry,failure_certainty) VALUES(?,?,?,?,?,?,?,?,?,?,?)",
                params![
                    record.id().get() as i64,
                    attempt.id().get() as i64,
                    attempt.revision().get() as i64,
                    attempt.configuration_id().get() as i64,
                    attempt.status().code(),
                    attempt.raw_transcript().filter(|_| retain_text).map(|value| value.as_str()),
                    attempt.partial_transcript().filter(|_| retain_text).map(|value| value.as_str()),
                    attempt_failure.map(|value| value.stage().code()),
                    attempt_failure.map(|value| value.code().code()),
                    attempt_failure.map(|value| value.retry().code()),
                    attempt_failure.map(|value| value.certainty().code()),
                ],
            )
            .map_err(|_| SqliteError::Database)?;
        }
        let old_audio = old_audio
            .flatten()
            .map(|value| Self::parse_reference(&value))
            .transpose()?;
        if let Some(old_audio) = old_audio {
            if Some(old_audio) != audio_ref {
                Self::queue_audio(&tx, old_audio).map_err(|_| SqliteError::Database)?;
            }
        }
        if !retain_audio {
            if let Some(reference) = available_audio_ref.filter(|_| available_audio_durable) {
                Self::queue_audio(&tx, reference).map_err(|_| SqliteError::Database)?;
            }
        }
        tx.commit().map_err(|_| SqliteError::Database)?;
        let _ = self.process_deletion_queue()?;
        let durable_materials = durable_materials
            .into_iter()
            .filter_map(|(kind, durable)| durable.then_some(kind))
            .collect();
        Ok(PersistenceReport { durable_materials })
    }
}

impl ConfigurationStorePort for HistorySqlite {
    fn load_recognition_configurations(&mut self) -> PortResult<Vec<RecognitionConfiguration>> {
        let mut statement = self
            .connection
            .prepare("SELECT id,name,provider_code,base_url,credential_ref,model FROM recognition_configurations ORDER BY id")
            .map_err(|_| SqliteError::Database.failure())?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })
            .map_err(|_| SqliteError::Database.failure())?;
        let mut configurations = Vec::new();
        for row in rows {
            let (id, name, provider, base_url, credential_ref, model) =
                row.map_err(|_| SqliteError::Database.failure())?;
            let Some(id) = ConfigurationId::new(id as u64) else {
                return Err(SqliteError::Database.failure());
            };
            let base_url = match base_url {
                Some(value) => {
                    Some(BaseUrl::parse(value).map_err(|_| SqliteError::Database.failure())?)
                }
                None => None,
            };
            let credential_reference = match credential_ref {
                Some(value) => Some(
                    CredentialReferenceId::new(value as u64)
                        .ok_or_else(|| SqliteError::Database.failure())?,
                ),
                None => None,
            };
            configurations.push(RecognitionConfiguration::new(
                id,
                name,
                provider,
                base_url,
                credential_reference,
                model,
            ));
        }
        Ok(configurations)
    }

    fn save_recognition_configuration(
        &mut self,
        configuration: RecognitionConfiguration,
    ) -> PortResult<()> {
        self.connection
            .execute(
                "INSERT INTO recognition_configurations(id,name,provider_code,base_url,credential_ref,model) VALUES(?,?,?,?,?,?) ON CONFLICT(id) DO UPDATE SET name=excluded.name,provider_code=excluded.provider_code,base_url=excluded.base_url,credential_ref=excluded.credential_ref,model=excluded.model",
                params![
                    configuration.id().get() as i64,
                    configuration.name(),
                    configuration.provider_code(),
                    configuration.base_url().map(BaseUrl::as_str),
                    configuration.credential_reference().map(|value| value.get() as i64),
                    configuration.model(),
                ],
            )
            .map_err(|_| SqliteError::Database.failure())?;
        Ok(())
    }

    fn set_active_recognition_configuration(
        &mut self,
        configuration: Option<ConfigurationId>,
    ) -> PortResult<()> {
        if let Some(id) = configuration {
            let exists: bool = self
                .connection
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM recognition_configurations WHERE id=?)",
                    [id.get() as i64],
                    |row| row.get(0),
                )
                .map_err(|_| SqliteError::Database.failure())?;
            if !exists {
                return Err(SqliteError::Database.failure());
            }
        }
        self.connection
            .execute(
                "INSERT INTO scalar_settings(key,value) VALUES('active_recognition',?) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                [configuration.map_or_else(String::new, |id| id.get().to_string())],
            )
            .map_err(|_| SqliteError::Database.failure())?;
        Ok(())
    }

    fn active_recognition_configuration(&mut self) -> PortResult<Option<ConfigurationId>> {
        let value: Option<String> = self
            .connection
            .query_row(
                "SELECT value FROM scalar_settings WHERE key='active_recognition'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| SqliteError::Database.failure())?;
        Ok(value
            .and_then(|value| value.parse::<u64>().ok())
            .and_then(ConfigurationId::new))
    }

    fn load_llm_configurations(&mut self) -> PortResult<Vec<LanguageModelConfiguration>> {
        let mut statement = self
            .connection
            .prepare("SELECT id,name,base_url,credential_ref,model,timeout_ms,reasoning FROM llm_configurations ORDER BY id")
            .map_err(|_| SqliteError::Database.failure())?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .map_err(|_| SqliteError::Database.failure())?;
        let mut configurations = Vec::new();
        for row in rows {
            let (id, name, base_url, credential_ref, model, timeout, reasoning) =
                row.map_err(|_| SqliteError::Database.failure())?;
            let (Some(id), Some(credential_reference), Some(timeout)) = (
                ConfigurationId::new(id as u64),
                CredentialReferenceId::new(credential_ref as u64),
                DurationLimit::new(timeout as u64),
            ) else {
                continue;
            };
            let Ok(base_url) = BaseUrl::parse(base_url) else {
                continue;
            };
            let reasoning_mode = match reasoning.as_str() {
                "disabled" => ReasoningMode::Disabled,
                "enabled" => ReasoningMode::Enabled,
                _ => ReasoningMode::ProviderDefault,
            };
            configurations.push(LanguageModelConfiguration::new(
                id,
                name,
                base_url,
                credential_reference,
                model,
                timeout,
                reasoning_mode,
            ));
        }
        Ok(configurations)
    }

    fn save_llm_configuration(
        &mut self,
        configuration: LanguageModelConfiguration,
    ) -> PortResult<()> {
        self.connection
            .execute(
                "INSERT INTO llm_configurations(id,name,base_url,credential_ref,model,timeout_ms,reasoning) VALUES(?,?,?,?,?,?,?) ON CONFLICT(id) DO UPDATE SET name=excluded.name,base_url=excluded.base_url,credential_ref=excluded.credential_ref,model=excluded.model,timeout_ms=excluded.timeout_ms,reasoning=excluded.reasoning",
                params![
                    configuration.id().get() as i64,
                    configuration.name(),
                    configuration.base_url().as_str(),
                    configuration.credential_reference().get() as i64,
                    configuration.model(),
                    configuration.timeout().milliseconds() as i64,
                    match configuration.reasoning_mode() {
                        ReasoningMode::Disabled => "disabled",
                        ReasoningMode::Enabled => "enabled",
                        ReasoningMode::ProviderDefault => "provider_default",
                    },
                ],
            )
            .map_err(|_| SqliteError::Database.failure())?;
        Ok(())
    }

    fn set_active_llm_configuration(
        &mut self,
        configuration: Option<ConfigurationId>,
    ) -> PortResult<()> {
        if let Some(id) = configuration {
            let exists: bool = self
                .connection
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM llm_configurations WHERE id=?)",
                    [id.get() as i64],
                    |row| row.get(0),
                )
                .map_err(|_| SqliteError::Database.failure())?;
            if !exists {
                return Err(SqliteError::Database.failure());
            }
        }
        self.connection
            .execute(
                "INSERT INTO scalar_settings(key,value) VALUES('active_llm',?) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                [configuration.map_or_else(String::new, |id| id.get().to_string())],
            )
            .map_err(|_| SqliteError::Database.failure())?;
        Ok(())
    }

    fn active_llm_configuration(&mut self) -> PortResult<Option<ConfigurationId>> {
        let value: Option<String> = self
            .connection
            .query_row(
                "SELECT value FROM scalar_settings WHERE key='active_llm'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| SqliteError::Database.failure())?;
        Ok(value
            .and_then(|value| value.parse::<u64>().ok())
            .and_then(ConfigurationId::new))
    }

    fn load_retention_policy(&mut self) -> PortResult<RetentionPolicy> {
        self.load_retention_inner()
            .map_err(|_| SqliteError::Database.failure())
    }

    fn save_retention_policy(&mut self, policy: RetentionPolicy) -> PortResult<()> {
        self.connection
            .execute(
                "INSERT INTO retention_policy(id,text_enabled,audio_enabled,text_days,audio_days) VALUES(1,?,?,?,?) ON CONFLICT(id) DO UPDATE SET text_enabled=excluded.text_enabled,audio_enabled=excluded.audio_enabled,text_days=excluded.text_days,audio_days=excluded.audio_days",
                params![policy.text_enabled as i64, policy.audio_enabled as i64, policy.text_days, policy.audio_days],
            )
            .map_err(|_| SqliteError::Database.failure())?;
        Ok(())
    }
}

impl PromptStorePort for HistorySqlite {
    fn list_prompts(&mut self) -> PortResult<Vec<PromptPreset>> {
        let mut statement = self
            .connection
            .prepare("SELECT id,name,content,built_in,shortcut FROM prompt_presets ORDER BY id")
            .map_err(|_| SqliteError::Database.failure())?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            })
            .map_err(|_| SqliteError::Database.failure())?;
        let mut prompts = Vec::new();
        for row in rows {
            let (id, name, content, built_in, shortcut) =
                row.map_err(|_| SqliteError::Database.failure())?;
            let Some(id) = PromptPresetId::new(id as u64) else {
                continue;
            };
            let mut prompt = match built_in.as_deref() {
                Some("original_text_cleanup") => {
                    PromptPreset::built_in(id, BuiltInPromptId::OriginalTextCleanup)
                }
                Some("concise_expression") => {
                    PromptPreset::built_in(id, BuiltInPromptId::ConciseExpression)
                }
                Some("formal_expression") => {
                    PromptPreset::built_in(id, BuiltInPromptId::FormalExpression)
                }
                _ => PromptPreset::custom(id, name, content),
            };
            if let Some(shortcut) = shortcut.and_then(PromptShortcut::new) {
                let _ = prompt.set_shortcut(Some(shortcut));
            }
            prompts.push(prompt);
        }
        Ok(prompts)
    }

    fn save_prompt(&mut self, prompt: PromptPreset) -> PortResult<()> {
        if prompt.is_built_in() {
            let existing: Option<(String, String, Option<String>)> = self
                .connection
                .query_row(
                    "SELECT name,content,built_in FROM prompt_presets WHERE id=?",
                    [prompt.id().get() as i64],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()
                .map_err(|_| SqliteError::Database.failure())?;
            if existing.is_some_and(|(_, _, built_in)| built_in.is_some()) {
                return Ok(());
            }
            return Err(SqliteError::Database.failure());
        }
        let existing_built_in: Option<Option<String>> = self
            .connection
            .query_row(
                "SELECT built_in FROM prompt_presets WHERE id=?",
                [prompt.id().get() as i64],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| SqliteError::Database.failure())?;
        if existing_built_in.is_some_and(|value| value.is_some()) {
            return Err(SqliteError::Database.failure());
        }
        self.connection
            .execute(
                "INSERT INTO prompt_presets(id,name,content,built_in,shortcut) VALUES(?,?,?,NULL,?) ON CONFLICT(id) DO UPDATE SET name=excluded.name,content=excluded.content,shortcut=excluded.shortcut,built_in=NULL",
                params![prompt.id().get() as i64, prompt.name(), prompt.content(), prompt.shortcut().map(ToString::to_string)],
            )
            .map_err(|_| SqliteError::Database.failure())?;
        Ok(())
    }

    fn delete_prompt(
        &mut self,
        prompt: PromptPresetId,
        confirm_referenced: bool,
    ) -> PortResult<PromptDeleteReport> {
        let built_in: Option<Option<String>> = self
            .connection
            .query_row(
                "SELECT built_in FROM prompt_presets WHERE id=?",
                [prompt.get() as i64],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| SqliteError::Database.failure())?;
        if built_in.as_ref().is_some_and(|value| value.is_some()) || built_in.is_none() {
            return Ok(PromptDeleteReport {
                deleted: false,
                affected_profiles: 0,
            });
        }
        let affected: u64 = self
            .connection
            .query_row(
                "SELECT COUNT(*) FROM application_profiles WHERE prompt_id=?",
                [prompt.get() as i64],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|_| SqliteError::Database.failure())? as u64;
        if affected > 0 && !confirm_referenced {
            return Ok(PromptDeleteReport {
                deleted: false,
                affected_profiles: affected as usize,
            });
        }
        let tx = self
            .connection
            .transaction()
            .map_err(|_| SqliteError::Database.failure())?;
        tx.execute(
            "UPDATE application_profiles SET prompt_id=NULL WHERE prompt_id=?",
            [prompt.get() as i64],
        )
        .map_err(|_| SqliteError::Database.failure())?;
        tx.execute(
            "UPDATE scalar_settings SET value='1' WHERE key='active_prompt' AND value=?",
            [prompt.get().to_string()],
        )
        .map_err(|_| SqliteError::Database.failure())?;
        tx.execute(
            "DELETE FROM prompt_presets WHERE id=?",
            [prompt.get() as i64],
        )
        .map_err(|_| SqliteError::Database.failure())?;
        tx.commit().map_err(|_| SqliteError::Database.failure())?;
        Ok(PromptDeleteReport {
            deleted: true,
            affected_profiles: affected as usize,
        })
    }

    fn set_active_prompt(&mut self, prompt: PromptPresetId) -> PortResult<()> {
        let exists: bool = self
            .connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM prompt_presets WHERE id=?)",
                [prompt.get() as i64],
                |row| row.get(0),
            )
            .map_err(|_| SqliteError::Database.failure())?;
        if !exists {
            return Err(SqliteError::Database.failure());
        }
        self.connection
            .execute("INSERT INTO scalar_settings(key,value) VALUES('active_prompt',?) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [prompt.get().to_string()])
            .map_err(|_| SqliteError::Database.failure())?;
        Ok(())
    }

    fn active_prompt(&mut self) -> PortResult<PromptPresetId> {
        let value: String = self
            .connection
            .query_row(
                "SELECT value FROM scalar_settings WHERE key='active_prompt'",
                [],
                |row| row.get(0),
            )
            .map_err(|_| SqliteError::Database.failure())?;
        value
            .parse::<u64>()
            .ok()
            .and_then(PromptPresetId::new)
            .ok_or_else(|| SqliteError::Database.failure())
    }

    fn activate_shortcut(
        &mut self,
        shortcut: &PromptShortcut,
    ) -> PortResult<Option<PromptPresetId>> {
        let id: Option<i64> = self
            .connection
            .query_row(
                "SELECT id FROM prompt_presets WHERE shortcut=?",
                [shortcut.as_str()],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| SqliteError::Database.failure())?;
        let Some(id) = id.and_then(|value| PromptPresetId::new(value as u64)) else {
            return Ok(None);
        };
        self.set_active_prompt(id)?;
        Ok(Some(id))
    }
}

impl LibraryStorePort for HistorySqlite {
    fn load_hotword_groups(&mut self) -> PortResult<Vec<HotwordGroup>> {
        let mut groups = Vec::new();
        let mut statement = self
            .connection
            .prepare("SELECT id,name,enabled FROM hotword_groups ORDER BY id")
            .map_err(|_| SqliteError::Database.failure())?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|_| SqliteError::Database.failure())?;
        for row in rows {
            let (id, name, enabled) = row.map_err(|_| SqliteError::Database.failure())?;
            let Some(id) = HotwordGroupId::new(id as u64) else {
                continue;
            };
            let mut items_statement = self
                .connection
                .prepare("SELECT id,text FROM hotwords WHERE group_id=? ORDER BY id")
                .map_err(|_| SqliteError::Database.failure())?;
            let items_rows = items_statement
                .query_map([id.get() as i64], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|_| SqliteError::Database.failure())?;
            let mut items = Vec::new();
            for row in items_rows {
                let (item_id, text) = row.map_err(|_| SqliteError::Database.failure())?;
                let item_id = HotwordId::new(item_id as u64)
                    .ok_or_else(|| SqliteError::Database.failure())?;
                items.push(Hotword::new(item_id, text));
            }
            groups.push(HotwordGroup::new(id, name, enabled != 0, items));
        }
        Ok(groups)
    }

    fn save_hotword_group(&mut self, group: HotwordGroup) -> PortResult<()> {
        let tx = self
            .connection
            .transaction()
            .map_err(|_| SqliteError::Database.failure())?;
        tx.execute("INSERT INTO hotword_groups(id,name,enabled) VALUES(?,?,?) ON CONFLICT(id) DO UPDATE SET name=excluded.name,enabled=excluded.enabled", params![group.id().get() as i64, group.name(), group.enabled() as i64]).map_err(|_| SqliteError::Database.failure())?;
        tx.execute(
            "DELETE FROM hotwords WHERE group_id=?",
            [group.id().get() as i64],
        )
        .map_err(|_| SqliteError::Database.failure())?;
        for item in group.items() {
            tx.execute(
                "INSERT INTO hotwords(id,group_id,text) VALUES(?,?,?)",
                params![item.id().get() as i64, group.id().get() as i64, item.text()],
            )
            .map_err(|_| SqliteError::Database.failure())?;
        }
        tx.commit().map_err(|_| SqliteError::Database.failure())?;
        Ok(())
    }

    fn load_application_profiles(&mut self) -> PortResult<Vec<ApplicationProfile>> {
        let mut profiles = Vec::new();
        let mut statement = self
            .connection
            .prepare("SELECT id,identity,enabled,prompt_id FROM application_profiles ORDER BY id")
            .map_err(|_| SqliteError::Database.failure())?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                ))
            })
            .map_err(|_| SqliteError::Database.failure())?;
        for row in rows {
            let (id, identity, enabled, prompt) =
                row.map_err(|_| SqliteError::Database.failure())?;
            let (Some(id), prompt) = (
                ApplicationProfileId::new(id as u64),
                prompt.and_then(|value| PromptPresetId::new(value as u64)),
            ) else {
                continue;
            };
            let mut profile = ApplicationProfile::new(id, identity, enabled != 0);
            profile.set_prompt(prompt);
            let mut overrides = self.connection.prepare("SELECT rule_code,override_code FROM application_profile_rule_overrides WHERE profile_id=? ORDER BY rule_code").map_err(|_| SqliteError::Database.failure())?;
            let override_rows = overrides
                .query_map([id.get() as i64], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|_| SqliteError::Database.failure())?;
            for row in override_rows {
                let (rule, value) = row.map_err(|_| SqliteError::Database.failure())?;
                let rule = match rule.as_str() {
                    "remove_trailing_sentence_punctuation" => 1,
                    "replace_conversational_punctuation_with_spaces" => 2,
                    _ => continue,
                };
                let Some(rule) = ProcessingRuleId::new(rule) else {
                    continue;
                };
                let value = match value.as_str() {
                    "force_enabled" => RuleOverride::ForceEnabled,
                    "force_disabled" => RuleOverride::ForceDisabled,
                    _ => RuleOverride::Inherit,
                };
                profile.set_rule_override(rule, value);
            }
            profiles.push(profile);
        }
        Ok(profiles)
    }

    fn save_application_profile(&mut self, profile: ApplicationProfile) -> PortResult<()> {
        let tx = self
            .connection
            .transaction()
            .map_err(|_| SqliteError::Database.failure())?;
        tx.execute("INSERT INTO application_profiles(id,identity,enabled,prompt_id) VALUES(?,?,?,?) ON CONFLICT(id) DO UPDATE SET identity=excluded.identity,enabled=excluded.enabled,prompt_id=excluded.prompt_id", params![profile.id().get() as i64, profile.identity(), profile.enabled() as i64, profile.prompt().map(|value| value.get() as i64)]).map_err(|_| SqliteError::Database.failure())?;
        tx.execute(
            "DELETE FROM application_profile_rule_overrides WHERE profile_id=?",
            [profile.id().get() as i64],
        )
        .map_err(|_| SqliteError::Database.failure())?;
        for (rule, value) in profile.rule_overrides() {
            let rule_code = match rule.get() {
                1 => BuiltInRuleId::RemoveTrailingSentencePunctuation.code(),
                2 => BuiltInRuleId::ReplaceConversationalPunctuationWithSpaces.code(),
                _ => continue,
            };
            let value_code = match value {
                RuleOverride::Inherit => "inherit",
                RuleOverride::ForceEnabled => "force_enabled",
                RuleOverride::ForceDisabled => "force_disabled",
            };
            tx.execute("INSERT INTO application_profile_rule_overrides(profile_id,rule_code,override_code) VALUES(?,?,?)", params![profile.id().get() as i64, rule_code, value_code]).map_err(|_| SqliteError::Database.failure())?;
        }
        tx.commit().map_err(|_| SqliteError::Database.failure())?;
        Ok(())
    }
}

impl ProcessingConfigurationPort for HistorySqlite {
    fn load_processing_order(&mut self) -> PortResult<ProcessingOrder> {
        let mut statement = self
            .connection
            .prepare("SELECT step_code FROM processing_steps ORDER BY position")
            .map_err(|_| SqliteError::Database.failure())?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|_| SqliteError::Database.failure())?;
        let mut steps = Vec::new();
        for code in rows {
            steps.push(
                match code.map_err(|_| SqliteError::Database.failure())?.as_str() {
                    "remove_trailing_sentence_punctuation" => ProcessingStepConfiguration::BuiltIn(
                        BuiltInRuleId::RemoveTrailingSentencePunctuation,
                    ),
                    "replace_conversational_punctuation_with_spaces" => {
                        ProcessingStepConfiguration::BuiltIn(
                            BuiltInRuleId::ReplaceConversationalPunctuationWithSpaces,
                        )
                    }
                    "language_model" => ProcessingStepConfiguration::LanguageModel,
                    _ => return Err(SqliteError::Database.failure()),
                },
            );
        }
        ProcessingOrder::new(steps).ok_or_else(|| SqliteError::Database.failure())
    }

    fn save_processing_order(&mut self, order: ProcessingOrder) -> PortResult<()> {
        let tx = self
            .connection
            .transaction()
            .map_err(|_| SqliteError::Database.failure())?;
        tx.execute("DELETE FROM processing_steps", [])
            .map_err(|_| SqliteError::Database.failure())?;
        for (position, step) in order.steps().iter().enumerate() {
            let code = match step {
                ProcessingStepConfiguration::BuiltIn(rule) => rule.code(),
                ProcessingStepConfiguration::LanguageModel => "language_model",
            };
            tx.execute(
                "INSERT INTO processing_steps(position,step_code) VALUES(?,?)",
                params![position as i64, code],
            )
            .map_err(|_| SqliteError::Database.failure())?;
        }
        tx.commit().map_err(|_| SqliteError::Database.failure())?;
        Ok(())
    }

    fn load_rule_defaults(&mut self) -> PortResult<Vec<(BuiltInRuleId, bool)>> {
        let mut output = Vec::new();
        for rule in BuiltInRuleId::all() {
            let enabled: Option<i64> = self
                .connection
                .query_row(
                    "SELECT enabled FROM processing_rule_defaults WHERE rule_code=?",
                    [rule.code()],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|_| SqliteError::Database.failure())?;
            output.push((rule, enabled.unwrap_or(0) != 0));
        }
        Ok(output)
    }

    fn save_rule_default(&mut self, rule: BuiltInRuleId, enabled: bool) -> PortResult<()> {
        self.connection.execute("INSERT INTO processing_rule_defaults(rule_code,enabled) VALUES(?,?) ON CONFLICT(rule_code) DO UPDATE SET enabled=excluded.enabled", params![rule.code(), enabled as i64]).map_err(|_| SqliteError::Database.failure())?;
        Ok(())
    }

    fn load_profile_override(
        &mut self,
        profile: ApplicationProfileId,
        rule: BuiltInRuleId,
    ) -> PortResult<RuleOverride> {
        let value: Option<String> = self.connection.query_row("SELECT override_code FROM application_profile_rule_overrides WHERE profile_id=? AND rule_code=?", params![profile.get() as i64, rule.code()], |row| row.get(0)).optional().map_err(|_| SqliteError::Database.failure())?;
        Ok(match value.as_deref() {
            Some("force_enabled") => RuleOverride::ForceEnabled,
            Some("force_disabled") => RuleOverride::ForceDisabled,
            _ => RuleOverride::Inherit,
        })
    }
}

impl HistoryStorePort for HistorySqlite {
    fn load_record(&mut self, record: DictationRecordId) -> PortResult<Option<DictationRecord>> {
        type RecordRow = (
            i64,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            i64,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            i64,
            i64,
        );
        let row: Option<RecordRow> = self
            .connection
            .query_row(
                "SELECT session_id,outcome,raw_text,processed_text,final_text,partial_text,audio_ref,audio_durable,failure_stage,failure_code,failure_retry,failure_certainty,hotwords_used,hotwords_total FROM dictation_records WHERE id=?",
                [record.get() as i64],
                |row| {
                    Ok((
                        row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?,
                        row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?, row.get(9)?,
                        row.get(10)?, row.get(11)?, row.get(12)?, row.get(13)?,
                    ))
                },
            )
            .optional()
            .map_err(|_| SqliteError::Database.failure())?;
        let Some((
            session_id,
            outcome,
            raw,
            processed,
            final_text,
            partial,
            audio_ref,
            audio_durable,
            failure_stage,
            failure_code,
            failure_retry,
            failure_certainty,
            hotwords_used,
            hotwords_total,
        )) = row
        else {
            return Ok(None);
        };
        let session_id =
            SessionId::new(session_id as u64).ok_or_else(|| SqliteError::Database.failure())?;
        let mut restored = DictationRecord::new(record, session_id);
        if let Some(value) = raw {
            restored.set_raw_transcript(RawTranscript::new(value));
        }
        if let Some(value) = processed {
            restored.set_processed_text(ProcessedText::new(value));
        }
        if let Some(value) = final_text {
            restored.set_final_text(voice_core::FinalText::new(value));
        }
        if let Some(value) = partial {
            restored.set_partial_transcript(PartialTranscript::new(value));
        }
        let mut restored_audio_durable = false;
        if let Some(reference) = audio_ref {
            let reference = Self::parse_reference(&reference).map_err(|error| error.failure())?;
            restored_audio_durable =
                audio_durable != 0 && self.committed_artifact_exists(reference);
            if restored_audio_durable {
                restored.set_recorded_audio(RecordedAudio::new(reference, true));
            }
        }
        if let Some(outcome) = outcome.and_then(|value| TerminalOutcome::from_code(&value)) {
            restored.set_outcome(outcome);
        }
        restored.set_hotword_usage(
            u32::try_from(hotwords_used).unwrap_or(0),
            u32::try_from(hotwords_total).unwrap_or(0),
        );
        let failure = match (
            failure_stage.as_deref().and_then(FailureStage::from_code),
            failure_code.as_deref().and_then(FailureCode::from_code),
            failure_retry
                .as_deref()
                .and_then(voice_core::RetryMeaning::from_code),
            failure_certainty
                .as_deref()
                .and_then(voice_core::DeliveryCertainty::from_code),
        ) {
            (Some(stage), Some(code), Some(retry), Some(certainty)) => {
                SanitizedFailure::new(stage, code, retry, certainty)
            }
            _ => None,
        };
        restored.set_failure(failure);

        let mut warnings = Vec::new();
        let mut warning_statement = self
            .connection
            .prepare("SELECT warning_code FROM dictation_warnings WHERE record_id=? ORDER BY warning_code")
            .map_err(|_| SqliteError::Database.failure())?;
        let warning_rows = warning_statement
            .query_map([record.get() as i64], |row| row.get::<_, String>(0))
            .map_err(|_| SqliteError::Database.failure())?;
        for value in warning_rows {
            if let Some(warning) =
                Warning::from_code(&value.map_err(|_| SqliteError::Database.failure())?)
            {
                warnings.push(warning);
            }
        }
        restored.set_warnings(warnings);

        let mut attempt_statement = self.connection.prepare("SELECT id,revision,configuration_id,status,raw_text,partial_text,failure_stage,failure_code,failure_retry,failure_certainty FROM recognition_attempts WHERE record_id=? ORDER BY id").map_err(|_| SqliteError::Database.failure())?;
        let attempt_rows = attempt_statement
            .query_map([record.get() as i64], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                ))
            })
            .map_err(|_| SqliteError::Database.failure())?;
        for row in attempt_rows {
            let (
                id,
                revision,
                configuration_id,
                status,
                raw,
                partial,
                stage,
                code,
                retry,
                certainty,
            ) = row.map_err(|_| SqliteError::Database.failure())?;
            let (Some(id), Some(revision), Some(configuration_id), Some(status)) = (
                RecognitionAttemptId::new(id as u64),
                Revision::new(revision as u64),
                ConfigurationId::new(configuration_id as u64),
                AttemptStatus::from_code(&status),
            ) else {
                return Err(SqliteError::Database.failure());
            };
            let failure = match (
                stage.as_deref().and_then(FailureStage::from_code),
                code.as_deref().and_then(FailureCode::from_code),
                retry
                    .as_deref()
                    .and_then(voice_core::RetryMeaning::from_code),
                certainty
                    .as_deref()
                    .and_then(voice_core::DeliveryCertainty::from_code),
            ) {
                (Some(stage), Some(code), Some(retry), Some(certainty)) => {
                    SanitizedFailure::new(stage, code, retry, certainty)
                }
                _ => None,
            };
            restored.append_attempt(RecognitionAttempt::restore(
                id,
                revision,
                configuration_id,
                status,
                raw.map(RawTranscript::new),
                partial.map(PartialTranscript::new),
                failure,
            ));
        }
        let durable: Vec<MaterialKind> = restored
            .materials()
            .available_kinds()
            .into_iter()
            .filter(|kind| *kind != MaterialKind::RecordedAudio || restored_audio_durable)
            .collect();
        restored.mark_materials_durable(&durable);
        Ok(Some(restored))
    }

    fn persist(
        &mut self,
        request: voice_ports::HistoryPersistRequest,
    ) -> PortResult<PersistenceReport> {
        self.save_record(&request.record, false)
            .map_err(|error| error.failure())
    }

    fn persist_recovery(
        &mut self,
        request: voice_ports::RecoveryPersistRequest,
    ) -> PortResult<PersistenceReport> {
        self.save_record(&request.record, true)
            .map_err(|error| error.failure())
    }

    fn persist_retry_attempt(
        &mut self,
        request: voice_ports::RetryAttemptPersistRequest,
    ) -> PortResult<()> {
        self.save_record(&request.record, true)
            .map(|_| ())
            .map_err(|error| error.failure())
    }

    fn persist_retry_result(
        &mut self,
        request: voice_ports::RetryResultPersistRequest,
    ) -> PortResult<()> {
        self.save_record(&request.record, true)
            .map(|_| ())
            .map_err(|error| error.failure())
    }
}

impl AudioArtifactStorePort for HistorySqlite {
    fn stage(&mut self, reference: AudioReferenceId, bytes: &[u8]) -> PortResult<()> {
        if bytes.is_empty() {
            let path = self.checked_path(&self.temporary_dir(), reference);
            if path.exists() {
                fs::remove_file(path).map_err(|_| SqliteError::Filesystem.failure())?;
            }
            return Ok(());
        }
        fs::write(self.checked_path(&self.temporary_dir(), reference), bytes)
            .map_err(|_| SqliteError::Filesystem.failure())
    }

    fn commit(&mut self, reference: AudioReferenceId) -> PortResult<bool> {
        let temporary = self.checked_path(&self.temporary_dir(), reference);
        let committed = self.checked_path(&self.committed_dir(), reference);
        if !temporary.is_file()
            || fs::metadata(&temporary)
                .map(|value| value.len())
                .unwrap_or(0)
                == 0
        {
            return Ok(false);
        }
        fs::rename(temporary, committed).map_err(|_| SqliteError::Filesystem.failure())?;
        if self
            .connection
            .execute(
                "INSERT INTO audio_artifacts(audio_ref,artifact_name,nonempty) VALUES(?,?,1) ON CONFLICT(audio_ref) DO UPDATE SET artifact_name=excluded.artifact_name,nonempty=1",
                params![reference.to_string(), Self::artifact_name(reference)],
            )
            .is_err()
        {
            let _ = fs::remove_file(self.checked_path(&self.committed_dir(), reference));
            return Err(SqliteError::Database.failure());
        }
        Ok(true)
    }

    fn delete(&mut self, reference: AudioReferenceId) -> PortResult<()> {
        let tx = self
            .connection
            .transaction()
            .map_err(|_| SqliteError::Database.failure())?;
        tx.execute(
            "UPDATE dictation_records SET audio_ref=NULL,audio_durable=0,durable=CASE WHEN raw_text IS NOT NULL OR processed_text IS NOT NULL OR final_text IS NOT NULL OR partial_text IS NOT NULL THEN 1 ELSE 0 END WHERE audio_ref=?",
            [reference.to_string()],
        )
        .map_err(|_| SqliteError::Database.failure())?;
        Self::queue_audio(&tx, reference).map_err(|_| SqliteError::Database.failure())?;
        tx.commit().map_err(|_| SqliteError::Database.failure())?;
        let _ = self
            .process_deletion_queue()
            .map_err(|error| error.failure())?;
        Ok(())
    }

    fn startup_maintenance(&mut self) -> PortResult<AudioMaintenanceReport> {
        let mut temporary_removed = 0;
        if self.temporary_dir().is_dir() {
            for entry in
                fs::read_dir(self.temporary_dir()).map_err(|_| SqliteError::Filesystem.failure())?
            {
                let path = entry.map_err(|_| SqliteError::Filesystem.failure())?.path();
                if path.is_file() {
                    fs::remove_file(path).map_err(|_| SqliteError::Filesystem.failure())?;
                    temporary_removed += 1;
                }
            }
        }
        let (queued_deleted, queued_remaining) = self
            .process_deletion_queue()
            .map_err(|error| error.failure())?;
        Ok(AudioMaintenanceReport {
            temporary_removed,
            queued_deleted,
            queued_remaining,
        })
    }
}

impl HistoryMaintenancePort for HistorySqlite {
    fn delete_record(&mut self, record: DictationRecordId) -> PortResult<HistoryDeletionReport> {
        let tx = self
            .connection
            .transaction()
            .map_err(|_| SqliteError::Database.failure())?;
        let audio_ref: Option<Option<String>> = tx
            .query_row(
                "SELECT audio_ref FROM dictation_records WHERE id=?",
                [record.get() as i64],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| SqliteError::Database.failure())?;
        let deleted = tx
            .execute(
                "DELETE FROM dictation_records WHERE id=?",
                [record.get() as i64],
            )
            .map_err(|_| SqliteError::Database.failure())? as u64;
        let reference = audio_ref
            .flatten()
            .map(|value| Self::parse_reference(&value))
            .transpose()
            .map_err(|error| error.failure())?;
        if let Some(reference) = reference {
            Self::queue_audio(&tx, reference).map_err(|_| SqliteError::Database.failure())?;
        }
        tx.commit().map_err(|_| SqliteError::Database.failure())?;
        let (queued_deleted, queued_remaining) = self
            .process_deletion_queue()
            .map_err(|error| error.failure())?;
        Ok(HistoryDeletionReport {
            records: deleted,
            artifacts_queued: queued_remaining + queued_deleted,
        })
    }

    fn delete_all_records(&mut self) -> PortResult<HistoryDeletionReport> {
        let tx = self
            .connection
            .transaction()
            .map_err(|_| SqliteError::Database.failure())?;
        let refs: Vec<String> = {
            let mut statement = tx
                .prepare("SELECT audio_ref FROM dictation_records WHERE audio_ref IS NOT NULL")
                .map_err(|_| SqliteError::Database.failure())?;
            let rows = statement
                .query_map([], |row| row.get(0))
                .map_err(|_| SqliteError::Database.failure())?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|_| SqliteError::Database.failure())?
        };
        let count: u64 = tx
            .query_row("SELECT COUNT(*) FROM dictation_records", [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|_| SqliteError::Database.failure())? as u64;
        tx.execute("DELETE FROM dictation_records", [])
            .map_err(|_| SqliteError::Database.failure())?;
        let artifacts_queued = refs.len() as u64;
        for value in refs {
            let reference = Self::parse_reference(&value).map_err(|error| error.failure())?;
            Self::queue_audio(&tx, reference).map_err(|_| SqliteError::Database.failure())?;
        }
        tx.commit().map_err(|_| SqliteError::Database.failure())?;
        let _ = self
            .process_deletion_queue()
            .map_err(|error| error.failure())?;
        Ok(HistoryDeletionReport {
            records: count,
            artifacts_queued,
        })
    }

    fn apply_retention(&mut self, now: Timestamp) -> PortResult<RetentionReport> {
        let policy = self
            .load_retention_inner()
            .map_err(|_| SqliteError::Database.failure())?;
        let text_retention = u64::from(policy.text_days) * 86_400_000;
        let text_cutoff = now
            .milliseconds()
            .checked_sub(text_retention)
            .map_or(-1, |value| value as i64);
        let audio_retention = u64::from(policy.audio_days) * 86_400_000;
        let audio_cutoff = now
            .milliseconds()
            .checked_sub(audio_retention)
            .map_or(-1, |value| value as i64);
        let tx = self
            .connection
            .transaction()
            .map_err(|_| SqliteError::Database.failure())?;
        let text_cleared = tx.execute("UPDATE dictation_records SET raw_text=NULL,processed_text=NULL,final_text=NULL,partial_text=NULL WHERE created_at <= ? AND (raw_text IS NOT NULL OR processed_text IS NOT NULL OR final_text IS NOT NULL OR partial_text IS NOT NULL)", [text_cutoff]).map_err(|_| SqliteError::Database.failure())? as u64;
        tx.execute(
            "UPDATE recognition_attempts SET raw_text=NULL,partial_text=NULL WHERE record_id IN (SELECT id FROM dictation_records WHERE created_at <= ?)",
            [text_cutoff],
        )
        .map_err(|_| SqliteError::Database.failure())?;
        let mut refs = Vec::new();
        let mut statement = tx.prepare("SELECT audio_ref FROM dictation_records WHERE created_at <= ? AND audio_ref IS NOT NULL").map_err(|_| SqliteError::Database.failure())?;
        let rows = statement
            .query_map([audio_cutoff], |row| row.get::<_, String>(0))
            .map_err(|_| SqliteError::Database.failure())?;
        for row in rows {
            refs.push(row.map_err(|_| SqliteError::Database.failure())?);
        }
        drop(statement);
        let audio_cleared = tx.execute("UPDATE dictation_records SET audio_ref=NULL,audio_durable=0 WHERE created_at <= ? AND audio_ref IS NOT NULL", [audio_cutoff]).map_err(|_| SqliteError::Database.failure())? as u64;
        for value in refs {
            let reference = Self::parse_reference(&value).map_err(|error| error.failure())?;
            Self::queue_audio(&tx, reference).map_err(|_| SqliteError::Database.failure())?;
        }
        let records_deleted = tx.execute("DELETE FROM dictation_records WHERE raw_text IS NULL AND processed_text IS NULL AND final_text IS NULL AND partial_text IS NULL AND audio_ref IS NULL AND outcome IS NULL AND failure_code IS NULL AND NOT EXISTS (SELECT 1 FROM dictation_warnings WHERE dictation_warnings.record_id=dictation_records.id)", []).map_err(|_| SqliteError::Database.failure())? as u64;
        tx.commit().map_err(|_| SqliteError::Database.failure())?;
        let _ = self
            .process_deletion_queue()
            .map_err(|error| error.failure())?;
        Ok(RetentionReport {
            records_deleted,
            text_cleared,
            audio_cleared,
        })
    }

    fn backup(&mut self) -> PortResult<Vec<u8>> {
        if self.database_path.is_none() {
            return Err(SqliteError::Filesystem.failure());
        }
        let backup_path = self.backup_temporary_path();
        if backup_path.exists() {
            fs::remove_file(&backup_path).map_err(|_| SqliteError::Filesystem.failure())?;
        }
        self.connection
            .backup(rusqlite::MAIN_DB, &backup_path, None)
            .map_err(|_| SqliteError::Database.failure())?;
        let bytes = fs::read(&backup_path).map_err(|_| SqliteError::Filesystem.failure())?;
        fs::remove_file(backup_path).map_err(|_| SqliteError::Filesystem.failure())?;
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voice_ports::{
        AudioArtifactStorePort, ConfigurationStorePort, HistoryMaintenancePort,
        HistoryPersistRequest, HistoryStorePort, LibraryStorePort, ProcessingConfigurationPort,
        PromptStorePort,
    };

    fn temp_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("voxora-m4-{nanos}"))
    }

    #[test]
    fn migration_seeds_defaults_and_keeps_credentials_out_of_database() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let db_path = root.join("history.sqlite");
        let mut db = HistorySqlite::open(&db_path, root.join("audio")).unwrap();
        assert_eq!(
            db.connection()
                .query_row("PRAGMA foreign_keys", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(db.active_prompt().unwrap().get(), 1);
        assert_eq!(db.load_rule_defaults().unwrap().len(), 2);
        assert_eq!(
            db.load_processing_order().unwrap(),
            ProcessingOrder::default()
        );
        let config = LanguageModelConfiguration::new(
            ConfigurationId::new(1).unwrap(),
            "synthetic",
            BaseUrl::parse("https://example.test/v1").unwrap(),
            CredentialReferenceId::new(2).unwrap(),
            "model",
            DurationLimit::from_seconds(3).unwrap(),
            ReasoningMode::ProviderDefault,
        );
        db.save_llm_configuration(config).unwrap();
        let recognition = RecognitionConfiguration::new(
            ConfigurationId::new(4).unwrap(),
            "synthetic recognition",
            "synthetic_provider",
            Some(BaseUrl::parse("https://recognition.example.test/v1").unwrap()),
            Some(CredentialReferenceId::new(5).unwrap()),
            "synthetic-model",
        );
        db.save_recognition_configuration(recognition.clone())
            .unwrap();
        db.set_active_recognition_configuration(Some(recognition.id()))
            .unwrap();
        assert_eq!(
            db.load_recognition_configurations().unwrap(),
            vec![recognition]
        );
        assert_eq!(
            db.active_recognition_configuration().unwrap(),
            Some(ConfigurationId::new(4).unwrap())
        );
        drop(db);
        let bytes = fs::read(db_path).unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains("synthetic-secret"));
        assert!(!String::from_utf8_lossy(&bytes).contains("user:synthetic-secret@"));
        let reopened = HistorySqlite::open(root.join("history.sqlite"), root.join("audio"));
        assert!(reopened.is_ok());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn newer_schema_is_rejected_without_downgrade() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let db_path = root.join("newer.sqlite");
        let connection = Connection::open(&db_path).unwrap();
        connection
            .execute_batch("CREATE TABLE schema_version(version INTEGER NOT NULL); INSERT INTO schema_version VALUES(99);")
            .unwrap();
        drop(connection);
        assert!(matches!(
            HistorySqlite::open(&db_path, root.join("audio")),
            Err(SqliteError::UnsupportedSchema)
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn version_one_schema_migrates_forward_transactionally() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let db_path = root.join("version-one.sqlite");
        let connection = Connection::open(&db_path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE schema_version(version INTEGER NOT NULL); INSERT INTO schema_version VALUES(1);",
            )
            .unwrap();
        drop(connection);

        let mut db = HistorySqlite::open(&db_path, root.join("audio")).unwrap();

        assert_eq!(db.active_prompt().unwrap().get(), 1);
        assert_eq!(
            db.connection()
                .query_row("SELECT version FROM schema_version", [], |row| row
                    .get::<_, i64>(0),)
                .unwrap(),
            SCHEMA_VERSION
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn incompatible_preexisting_schema_rolls_back_migration() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let db_path = root.join("incompatible.sqlite");
        let connection = Connection::open(&db_path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE schema_version(version INTEGER NOT NULL); INSERT INTO schema_version VALUES(0); CREATE TABLE scalar_settings(key TEXT PRIMARY KEY NOT NULL);",
            )
            .unwrap();
        drop(connection);

        assert!(matches!(
            HistorySqlite::open(&db_path, root.join("audio")),
            Err(SqliteError::Migration)
        ));
        let connection = Connection::open(&db_path).unwrap();
        let columns: Vec<String> = connection
            .prepare("PRAGMA table_info(scalar_settings)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(columns, vec!["key"]);
        assert_eq!(
            connection
                .query_row("SELECT version FROM schema_version", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn prompts_profiles_hotwords_and_rule_order_round_trip_atomically() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let mut db = HistorySqlite::open(root.join("history.sqlite"), root.join("audio")).unwrap();
        let custom_id = PromptPresetId::new(10).unwrap();
        let mut custom = PromptPreset::custom(custom_id, "Synthetic Prompt", "Synthetic content");
        assert!(custom.set_shortcut(PromptShortcut::new("Ctrl + Shift + 9")));
        db.save_prompt(custom).unwrap();
        assert_eq!(
            db.activate_shortcut(&PromptShortcut::new("Ctrl+Shift+9").unwrap())
                .unwrap(),
            Some(custom_id)
        );
        let mut profile = ApplicationProfile::new(
            ApplicationProfileId::new(11).unwrap(),
            "synthetic-app",
            true,
        );
        profile.set_prompt(Some(custom_id));
        profile.set_rule_override(
            ProcessingRuleId::new(1).unwrap(),
            RuleOverride::ForceEnabled,
        );
        db.save_application_profile(profile).unwrap();
        let report = db.delete_prompt(custom_id, false).unwrap();
        assert_eq!((report.deleted, report.affected_profiles), (false, 1));
        let report = db.delete_prompt(custom_id, true).unwrap();
        assert_eq!((report.deleted, report.affected_profiles), (true, 1));
        assert_eq!(db.active_prompt().unwrap().get(), 1);
        assert_eq!(db.load_application_profiles().unwrap()[0].prompt(), None);

        let group = HotwordGroup::new(
            HotwordGroupId::new(12).unwrap(),
            "Synthetic Group",
            true,
            vec![Hotword::new(HotwordId::new(13).unwrap(), "Synthetic Term")],
        );
        db.save_hotword_group(group.clone()).unwrap();
        assert_eq!(db.load_hotword_groups().unwrap(), vec![group]);
        let order = ProcessingOrder::new(vec![
            ProcessingStepConfiguration::LanguageModel,
            ProcessingStepConfiguration::BuiltIn(
                BuiltInRuleId::ReplaceConversationalPunctuationWithSpaces,
            ),
            ProcessingStepConfiguration::BuiltIn(BuiltInRuleId::RemoveTrailingSentencePunctuation),
        ])
        .unwrap();
        db.save_processing_order(order.clone()).unwrap();
        assert_eq!(db.load_processing_order().unwrap(), order);
        assert!(
            db.save_prompt(PromptPreset::custom(
                PromptPresetId::new(1).unwrap(),
                "replacement",
                "replacement",
            ))
            .is_err()
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn artifact_store_stages_commits_and_cleans_temp() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let mut db = HistorySqlite::open(root.join("history.sqlite"), root.join("audio")).unwrap();
        let reference = AudioReferenceId::new(3).unwrap();
        db.stage(reference, b"synthetic-audio").unwrap();
        assert!(db.commit(reference).unwrap());
        let stored_name: String = db
            .connection()
            .query_row(
                "SELECT artifact_name FROM audio_artifacts WHERE audio_ref=?",
                [reference.to_string()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored_name, "audio-3.bin");
        let report = db.startup_maintenance().unwrap();
        assert_eq!(report.temporary_removed, 0);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn history_round_trip_keeps_raw_final_and_committed_audio_independent() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let db_path = root.join("history.sqlite");
        let mut db = HistorySqlite::open(&db_path, root.join("audio")).unwrap();
        let audio = AudioReferenceId::new(20).unwrap();
        db.stage(audio, b"nonempty synthetic audio").unwrap();
        assert!(db.commit(audio).unwrap());
        let record_id = DictationRecordId::new(21).unwrap();
        let mut record = DictationRecord::new(record_id, SessionId::new(22).unwrap());
        record.set_recorded_audio(RecordedAudio::new(audio, true));
        record.set_raw_transcript(RawTranscript::new("synthetic raw"));
        record.set_final_text(voice_core::FinalText::new("synthetic final"));
        record.set_hotword_usage(2, 3);
        record.set_outcome(TerminalOutcome::ManualDeliveryRequired);
        let report = db
            .persist(HistoryPersistRequest {
                record_id,
                record: record.clone(),
            })
            .unwrap();
        assert!(
            report
                .durable_materials
                .contains(&MaterialKind::RecordedAudio)
        );
        let loaded = db.load_record(record_id).unwrap().unwrap();
        assert_eq!(loaded.raw_transcript().unwrap().as_str(), "synthetic raw");
        assert_eq!(loaded.final_text().unwrap().as_str(), "synthetic final");
        assert_eq!(loaded.hotword_usage(), (2, 3));
        let backup = db.backup().unwrap();
        assert!(backup.starts_with(b"SQLite format 3\0"));
        assert!(!String::from_utf8_lossy(&backup).contains("synthetic-secret"));
        assert!(!db.backup_temporary_path().exists());
        db.delete_record(record_id).unwrap();
        assert!(!db.checked_path(&db.committed_dir(), audio).exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn record_recovery_preserves_created_at_and_replaces_audio_transactionally() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let mut db = HistorySqlite::open(root.join("history.sqlite"), root.join("audio")).unwrap();
        db.save_retention_policy(RetentionPolicy {
            text_enabled: true,
            audio_enabled: true,
            text_days: 1,
            audio_days: 1,
        })
        .unwrap();
        let old_audio = AudioReferenceId::new(70).unwrap();
        db.stage(old_audio, b"old audio").unwrap();
        assert!(db.commit(old_audio).unwrap());
        let record_id = DictationRecordId::new(71).unwrap();
        let mut record = DictationRecord::new(record_id, SessionId::new(72).unwrap());
        record.set_recorded_audio(RecordedAudio::new(old_audio, true));
        record.set_outcome(TerminalOutcome::Failed);
        db.persist(HistoryPersistRequest {
            record_id,
            record: record.clone(),
        })
        .unwrap();
        db.connection()
            .execute(
                "UPDATE dictation_records SET created_at=0 WHERE id=?",
                [record_id.get() as i64],
            )
            .unwrap();

        let new_audio = AudioReferenceId::new(73).unwrap();
        db.stage(new_audio, b"new audio").unwrap();
        assert!(db.commit(new_audio).unwrap());
        record.set_recorded_audio(RecordedAudio::new(new_audio, true));
        db.persist_recovery(voice_ports::RecoveryPersistRequest {
            correlation: voice_core::RecoveryCorrelation::new(
                voice_core::RecoveryId::new(74).unwrap(),
                record_id,
                SessionId::new(72).unwrap(),
            ),
            record: record.clone(),
        })
        .unwrap();

        let mut retry_record = record;
        let mut retry_attempt = RecognitionAttempt::new(
            RecognitionAttemptId::new(84).unwrap(),
            Revision::new(2).unwrap(),
            ConfigurationId::new(85).unwrap(),
        );
        retry_attempt.accept_final(RawTranscript::new("retry text"));
        retry_record.append_attempt(retry_attempt.clone());
        db.persist_retry_result(voice_ports::RetryResultPersistRequest {
            correlation: voice_core::RetryCorrelation::new(
                record_id,
                SessionId::new(72).unwrap(),
                retry_attempt.id(),
                Revision::new(2).unwrap(),
                voice_core::Phase::Recognizing,
            ),
            record: retry_record,
            attempt: retry_attempt,
        })
        .unwrap();

        let created_at: i64 = db
            .connection()
            .query_row(
                "SELECT created_at FROM dictation_records WHERE id=?",
                [record_id.get() as i64],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(created_at, 0);
        assert!(!db.checked_path(&db.committed_dir(), old_audio).exists());
        assert!(db.checked_path(&db.committed_dir(), new_audio).exists());
        assert_eq!(
            db.connection()
                .query_row("SELECT COUNT(*) FROM artifact_deletion_queue", [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap(),
            0
        );

        let report = db.apply_retention(Timestamp::new(86_400_000));
        assert_eq!(report.unwrap().audio_cleared, 1);
        assert!(!db.checked_path(&db.committed_dir(), new_audio).exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn text_only_recovery_update_preserves_created_at_and_delete_handles_null_audio() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let mut db = HistorySqlite::open(root.join("history.sqlite"), root.join("audio")).unwrap();
        let record_id = DictationRecordId::new(86).unwrap();
        let session_id = SessionId::new(87).unwrap();
        let mut record = DictationRecord::new(record_id, session_id);
        record.set_raw_transcript(RawTranscript::new("text-only recovery"));
        record.set_outcome(TerminalOutcome::Failed);
        db.persist(HistoryPersistRequest {
            record_id,
            record: record.clone(),
        })
        .unwrap();
        db.connection()
            .execute(
                "UPDATE dictation_records SET created_at=0 WHERE id=?",
                [record_id.get() as i64],
            )
            .unwrap();

        record.set_final_text(voice_core::FinalText::new("text-only final"));
        db.persist_recovery(voice_ports::RecoveryPersistRequest {
            correlation: voice_core::RecoveryCorrelation::new(
                voice_core::RecoveryId::new(88).unwrap(),
                record_id,
                session_id,
            ),
            record,
        })
        .unwrap();
        assert_eq!(
            db.connection()
                .query_row(
                    "SELECT created_at FROM dictation_records WHERE id=?",
                    [record_id.get() as i64],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap(),
            0
        );
        let loaded = db.load_record(record_id).unwrap().unwrap();
        assert!(loaded.recorded_audio().is_none());
        assert_eq!(loaded.final_text().unwrap().as_str(), "text-only final");

        let report = db.delete_record(record_id).unwrap();
        assert_eq!(report.records, 1);
        assert!(db.load_record(record_id).unwrap().is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn save_record_keeps_deletion_queue_when_filesystem_remove_fails() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let mut db = HistorySqlite::open(root.join("history.sqlite"), root.join("audio")).unwrap();
        let audio = AudioReferenceId::new(75).unwrap();
        db.stage(audio, b"audio that cannot be removed").unwrap();
        assert!(db.commit(audio).unwrap());
        let record_id = DictationRecordId::new(76).unwrap();
        let mut record = DictationRecord::new(record_id, SessionId::new(77).unwrap());
        record.set_recorded_audio(RecordedAudio::new(audio, true));
        record.set_outcome(TerminalOutcome::Failed);
        db.persist(HistoryPersistRequest {
            record_id,
            record: record.clone(),
        })
        .unwrap();
        let committed = db.checked_path(&db.committed_dir(), audio);
        fs::remove_file(&committed).unwrap();
        fs::create_dir(&committed).unwrap();
        record.set_recorded_audio(RecordedAudio::new(audio, true));
        db.persist_recovery(voice_ports::RecoveryPersistRequest {
            correlation: voice_core::RecoveryCorrelation::new(
                voice_core::RecoveryId::new(78).unwrap(),
                record_id,
                SessionId::new(77).unwrap(),
            ),
            record,
        })
        .unwrap();
        assert_eq!(
            db.connection()
                .query_row("SELECT COUNT(*) FROM artifact_deletion_queue", [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap(),
            1
        );
        fs::remove_dir(&committed).unwrap();
        let report = db.startup_maintenance().unwrap();
        assert_eq!((report.queued_deleted, report.queued_remaining), (1, 0));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn history_round_trip_restores_attempts_warnings_failures_and_all_text_fields() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let mut db = HistorySqlite::open(root.join("history.sqlite"), root.join("audio")).unwrap();
        let record_id = DictationRecordId::new(24).unwrap();
        let mut record = DictationRecord::new(record_id, SessionId::new(25).unwrap());
        record.set_raw_transcript(RawTranscript::new("synthetic raw"));
        record.set_processed_text(ProcessedText::new("synthetic processed"));
        record.set_final_text(voice_core::FinalText::new("synthetic final"));
        record.set_partial_transcript(PartialTranscript::new("synthetic partial"));
        record.set_warnings(vec![Warning::ProcessingFallback, Warning::LowVolume]);
        let failure = SanitizedFailure::from_boundary(
            FailureStage::Recognition,
            FailureCode::RecognitionProvider,
            voice_core::RetryMeaning::Retryable,
            voice_core::DeliveryCertainty::NotApplicable,
        );
        record.set_failure(Some(failure));
        record.set_outcome(TerminalOutcome::Failed);
        let mut attempt = RecognitionAttempt::new(
            RecognitionAttemptId::new(26).unwrap(),
            Revision::first(),
            ConfigurationId::new(27).unwrap(),
        );
        attempt.accept_partial(PartialTranscript::new("attempt partial"));
        attempt.fail(failure);
        record.append_attempt(attempt);

        db.persist(HistoryPersistRequest { record_id, record })
            .unwrap();
        let loaded = db.load_record(record_id).unwrap().unwrap();

        assert_eq!(
            loaded.processed_text().unwrap().as_str(),
            "synthetic processed"
        );
        assert_eq!(
            loaded.partial_transcript().unwrap().as_str(),
            "synthetic partial"
        );
        assert_eq!(
            loaded.warnings(),
            &[Warning::LowVolume, Warning::ProcessingFallback]
        );
        assert_eq!(loaded.failure(), Some(failure));
        assert_eq!(loaded.attempts().len(), 1);
        assert_eq!(loaded.attempts()[0].status(), AttemptStatus::Failed);
        assert_eq!(
            loaded.attempts()[0].partial_transcript().unwrap().as_str(),
            "attempt partial"
        );
        assert_eq!(loaded.attempts()[0].failure(), Some(&failure));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn load_never_claims_missing_audio_is_available_or_durable() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let mut db = HistorySqlite::open(root.join("history.sqlite"), root.join("audio")).unwrap();
        let audio = AudioReferenceId::new(64).unwrap();
        db.stage(audio, b"durability check audio").unwrap();
        assert!(db.commit(audio).unwrap());
        let record_id = DictationRecordId::new(65).unwrap();
        let mut record = DictationRecord::new(record_id, SessionId::new(66).unwrap());
        record.set_recorded_audio(RecordedAudio::new(audio, true));
        record.set_outcome(TerminalOutcome::Failed);
        db.persist(HistoryPersistRequest { record_id, record })
            .unwrap();
        fs::remove_file(db.checked_path(&db.committed_dir(), audio)).unwrap();

        let loaded = db.load_record(record_id).unwrap().unwrap();

        assert!(loaded.recorded_audio().is_none());
        assert_eq!(
            loaded.materials().state(MaterialKind::RecordedAudio),
            voice_core::MaterialState::Absent
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn capture_boundary_keeps_only_supplied_nonempty_audio() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let mut db = HistorySqlite::open(root.join("history.sqlite"), root.join("audio")).unwrap();
        let capture_failure = SanitizedFailure::from_boundary(
            FailureStage::Capture,
            FailureCode::DeviceFailure,
            voice_core::RetryMeaning::Retryable,
            voice_core::DeliveryCertainty::NotApplicable,
        );

        let missing_id = DictationRecordId::new(28).unwrap();
        let mut missing = DictationRecord::new(missing_id, SessionId::new(29).unwrap());
        missing.set_raw_transcript(RawTranscript::new("must not persist"));
        missing.set_final_text(voice_core::FinalText::new("must not persist"));
        missing.set_failure(Some(capture_failure));
        missing.set_outcome(TerminalOutcome::Failed);
        db.persist(HistoryPersistRequest {
            record_id: missing_id,
            record: missing,
        })
        .unwrap();
        let missing = db.load_record(missing_id).unwrap().unwrap();
        assert!(missing.raw_transcript().is_none());
        assert!(missing.final_text().is_none());
        assert!(missing.recorded_audio().is_none());

        let audio = AudioReferenceId::new(61).unwrap();
        db.stage(audio, b"capture partial audio").unwrap();
        assert!(db.commit(audio).unwrap());
        let supplied_id = DictationRecordId::new(62).unwrap();
        let mut supplied = DictationRecord::new(supplied_id, SessionId::new(63).unwrap());
        supplied.set_recorded_audio(RecordedAudio::new(audio, true));
        supplied.set_raw_transcript(RawTranscript::new("must not persist"));
        supplied.set_failure(Some(capture_failure));
        supplied.set_outcome(TerminalOutcome::Failed);
        let report = db
            .persist(HistoryPersistRequest {
                record_id: supplied_id,
                record: supplied,
            })
            .unwrap();
        assert_eq!(report.durable_materials, vec![MaterialKind::RecordedAudio]);
        let supplied = db.load_record(supplied_id).unwrap().unwrap();
        assert!(supplied.raw_transcript().is_none());
        assert!(supplied.recorded_audio().is_some());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn backup_uses_only_the_adapter_owned_temporary_path() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let database_path = root.join("history.sqlite");
        let adjacent = database_path.with_extension("backup.tmp");
        fs::write(&adjacent, b"unrelated").unwrap();
        let mut db = HistorySqlite::open(&database_path, root.join("audio")).unwrap();

        let backup = db.backup().unwrap();

        assert!(backup.starts_with(b"SQLite format 3\0"));
        assert_eq!(fs::read(adjacent).unwrap(), b"unrelated");
        assert!(!db.backup_temporary_path().exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn history_disabled_still_preserves_failure_but_omits_ordinary_success() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let mut db = HistorySqlite::open(root.join("history.sqlite"), root.join("audio")).unwrap();
        db.save_retention_policy(RetentionPolicy {
            text_enabled: false,
            audio_enabled: false,
            text_days: 7,
            audio_days: 9,
        })
        .unwrap();
        assert_eq!(db.load_retention_policy().unwrap().text_days, 7);

        let success_id = DictationRecordId::new(30).unwrap();
        let mut success = DictationRecord::new(success_id, SessionId::new(31).unwrap());
        success.set_final_text(voice_core::FinalText::new("ordinary success"));
        success.set_outcome(TerminalOutcome::DeliveredAutomatically);
        assert!(
            db.persist(HistoryPersistRequest {
                record_id: success_id,
                record: success,
            })
            .unwrap()
            .durable_materials
            .is_empty()
        );
        assert!(db.load_record(success_id).unwrap().is_none());

        let omitted_audio = AudioReferenceId::new(34).unwrap();
        db.stage(omitted_audio, b"ordinary omitted audio").unwrap();
        assert!(db.commit(omitted_audio).unwrap());
        let omitted_id = DictationRecordId::new(35).unwrap();
        let mut omitted = DictationRecord::new(omitted_id, SessionId::new(36).unwrap());
        omitted.set_recorded_audio(RecordedAudio::new(omitted_audio, true));
        omitted.set_outcome(TerminalOutcome::DeliveredAutomatically);
        assert!(
            db.persist(HistoryPersistRequest {
                record_id: omitted_id,
                record: omitted,
            })
            .unwrap()
            .durable_materials
            .is_empty()
        );
        assert!(db.load_record(omitted_id).unwrap().is_none());
        assert!(!db.checked_path(&db.committed_dir(), omitted_audio).exists());

        let failure_id = DictationRecordId::new(32).unwrap();
        let mut failure = DictationRecord::new(failure_id, SessionId::new(33).unwrap());
        failure.set_raw_transcript(RawTranscript::new("recovery text"));
        failure.set_outcome(TerminalOutcome::Failed);
        db.persist(HistoryPersistRequest {
            record_id: failure_id,
            record: failure,
        })
        .unwrap();
        assert_eq!(
            db.load_record(failure_id)
                .unwrap()
                .unwrap()
                .raw_transcript()
                .unwrap()
                .as_str(),
            "recovery text"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn retention_clears_text_and_audio_independently_and_keeps_required_metadata() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let mut db = HistorySqlite::open(root.join("history.sqlite"), root.join("audio")).unwrap();
        db.save_retention_policy(RetentionPolicy {
            text_enabled: true,
            audio_enabled: true,
            text_days: 1,
            audio_days: 10,
        })
        .unwrap();
        let audio = AudioReferenceId::new(40).unwrap();
        db.stage(audio, b"retention audio").unwrap();
        assert!(db.commit(audio).unwrap());
        let record_id = DictationRecordId::new(41).unwrap();
        let mut record = DictationRecord::new(record_id, SessionId::new(42).unwrap());
        record.set_recorded_audio(RecordedAudio::new(audio, true));
        record.set_raw_transcript(RawTranscript::new("retention raw"));
        record.set_final_text(voice_core::FinalText::new("retention final"));
        record.set_outcome(TerminalOutcome::ManualDeliveryRequired);
        let mut attempt = RecognitionAttempt::new(
            RecognitionAttemptId::new(43).unwrap(),
            Revision::first(),
            ConfigurationId::new(44).unwrap(),
        );
        attempt.accept_final(RawTranscript::new("attempt raw"));
        record.append_attempt(attempt);
        db.persist(HistoryPersistRequest { record_id, record })
            .unwrap();
        db.connection()
            .execute(
                "UPDATE dictation_records SET created_at=0 WHERE id=?",
                [record_id.get() as i64],
            )
            .unwrap();

        let first = db.apply_retention(Timestamp::new(2 * 86_400_000)).unwrap();
        assert_eq!((first.text_cleared, first.audio_cleared), (1, 0));
        let loaded = db.load_record(record_id).unwrap().unwrap();
        assert!(loaded.raw_transcript().is_none());
        assert!(loaded.final_text().is_none());
        assert!(loaded.attempts()[0].raw_transcript().is_none());
        assert!(loaded.recorded_audio().is_some());

        let second = db.apply_retention(Timestamp::new(11 * 86_400_000)).unwrap();
        assert_eq!((second.text_cleared, second.audio_cleared), (0, 1));
        let loaded = db.load_record(record_id).unwrap().unwrap();
        assert!(loaded.recorded_audio().is_none());
        assert_eq!(
            loaded.outcome(),
            Some(TerminalOutcome::ManualDeliveryRequired)
        );
        assert!(!db.checked_path(&db.committed_dir(), audio).exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn deletion_queue_retries_filesystem_failure_and_rejects_traversal() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let mut db = HistorySqlite::open(root.join("history.sqlite"), root.join("audio")).unwrap();
        let audio = AudioReferenceId::new(50).unwrap();
        db.stage(audio, b"queued audio").unwrap();
        assert!(db.commit(audio).unwrap());
        let record_id = DictationRecordId::new(51).unwrap();
        let mut record = DictationRecord::new(record_id, SessionId::new(52).unwrap());
        record.set_recorded_audio(RecordedAudio::new(audio, true));
        record.set_outcome(TerminalOutcome::Failed);
        db.persist(HistoryPersistRequest { record_id, record })
            .unwrap();

        let committed = db.checked_path(&db.committed_dir(), audio);
        fs::remove_file(&committed).unwrap();
        fs::create_dir(&committed).unwrap();
        let report = db.delete_record(record_id).unwrap();
        assert_eq!((report.records, report.artifacts_queued), (1, 1));
        let queued: i64 = db
            .connection()
            .query_row("SELECT COUNT(*) FROM artifact_deletion_queue", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(queued, 1);
        fs::remove_dir(&committed).unwrap();
        let retry = db.startup_maintenance().unwrap();
        assert_eq!((retry.queued_deleted, retry.queued_remaining), (1, 0));

        let unrelated = root.join("unrelated.txt");
        fs::write(&unrelated, b"keep").unwrap();
        db.connection()
            .execute(
                "INSERT INTO artifact_deletion_queue(artifact_name) VALUES('../unrelated.txt')",
                [],
            )
            .unwrap();
        let traversal = db.startup_maintenance().unwrap();
        assert_eq!(traversal.queued_remaining, 1);
        assert_eq!(fs::read(&unrelated).unwrap(), b"keep");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn direct_audio_delete_uses_the_durable_queue() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let mut db = HistorySqlite::open(root.join("history.sqlite"), root.join("audio")).unwrap();
        let audio = AudioReferenceId::new(55).unwrap();
        db.stage(audio, b"queued direct delete").unwrap();
        assert!(db.commit(audio).unwrap());
        let record_id = DictationRecordId::new(89).unwrap();
        let mut record = DictationRecord::new(record_id, SessionId::new(90).unwrap());
        record.set_recorded_audio(RecordedAudio::new(audio, true));
        record.set_raw_transcript(RawTranscript::new("preserve this text"));
        record.set_outcome(TerminalOutcome::Failed);
        db.persist(HistoryPersistRequest { record_id, record })
            .unwrap();
        let committed = db.checked_path(&db.committed_dir(), audio);
        fs::remove_file(&committed).unwrap();
        fs::create_dir(&committed).unwrap();

        db.delete(audio).unwrap();

        assert_eq!(
            db.connection()
                .query_row(
                    "SELECT COUNT(*) FROM audio_artifacts WHERE audio_ref=?",
                    [audio.to_string()],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap(),
            0
        );
        let persisted_audio_state: (Option<String>, i64, i64) = db
            .connection()
            .query_row(
                "SELECT audio_ref,audio_durable,durable FROM dictation_records WHERE id=?",
                [record_id.get() as i64],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(persisted_audio_state, (None, 0, 1));
        let loaded = db.load_record(record_id).unwrap().unwrap();
        assert!(loaded.recorded_audio().is_none());
        assert_eq!(
            loaded.raw_transcript().unwrap().as_str(),
            "preserve this text"
        );
        assert_eq!(
            db.connection()
                .query_row("SELECT COUNT(*) FROM artifact_deletion_queue", [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap(),
            1
        );
        fs::remove_dir(&committed).unwrap();
        let retry = db.startup_maintenance().unwrap();
        assert_eq!((retry.queued_deleted, retry.queued_remaining), (1, 0));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn real_history_and_audio_maintenance_services_use_one_adapter_sequentially() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let mut db = HistorySqlite::open(root.join("history.sqlite"), root.join("audio")).unwrap();
        {
            let mut history = voice_application::HistoryMaintenanceService::new(&mut db);
            assert_eq!(history.delete_all_records().unwrap().records, 0);
        }
        {
            let mut audio = voice_application::AudioMaintenanceService::new(&mut db);
            assert_eq!(audio.startup().unwrap().queued_remaining, 0);
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn configuration_service_rejects_credential_url_before_real_history_save_and_backup() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let database_path = root.join("history.sqlite");
        let mut db = HistorySqlite::open(&database_path, root.join("audio")).unwrap();
        let rejected = "https://user:synthetic-secret@example.test/v1";
        let error = voice_application::ConfigurationService::new(&mut db)
            .save_llm_configuration(
                ConfigurationId::new(91).unwrap(),
                "synthetic",
                rejected,
                CredentialReferenceId::new(92).unwrap(),
                "synthetic-model",
                DurationLimit::from_seconds(3).unwrap(),
                ReasoningMode::ProviderDefault,
            )
            .unwrap_err();
        assert_eq!(
            error,
            voice_application::ConfigurationError::BaseUrl(voice_core::BaseUrlError::UserInfo)
        );
        assert!(db.load_llm_configurations().unwrap().is_empty());
        drop(db);
        let database_bytes = fs::read(&database_path).unwrap();
        let database_text = String::from_utf8_lossy(&database_bytes);
        assert!(!database_text.contains(rejected));
        assert!(!database_text.contains("synthetic-secret"));
        let mut db = HistorySqlite::open(&database_path, root.join("audio")).unwrap();
        let backup = db.backup().unwrap();
        let backup_text = String::from_utf8_lossy(&backup);
        assert!(!backup_text.contains(rejected));
        assert!(!backup_text.contains("synthetic-secret"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn startup_cleanup_removes_only_owned_temporary_files() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let mut db = HistorySqlite::open(root.join("history.sqlite"), root.join("audio")).unwrap();
        let committed = AudioReferenceId::new(60).unwrap();
        db.stage(committed, b"committed").unwrap();
        assert!(db.commit(committed).unwrap());
        fs::write(db.temporary_dir().join("orphan.tmp"), b"orphan").unwrap();
        fs::create_dir(db.temporary_dir().join("nested")).unwrap();
        let unrelated = root.join("outside-audio.txt");
        fs::write(&unrelated, b"keep").unwrap();
        let report = db.startup_maintenance().unwrap();
        assert_eq!(report.temporary_removed, 1);
        assert!(db.checked_path(&db.committed_dir(), committed).exists());
        assert!(db.temporary_dir().join("nested").exists());
        assert_eq!(fs::read(unrelated).unwrap(), b"keep");
        let second = db.startup_maintenance().unwrap();
        assert_eq!((second.temporary_removed, second.queued_deleted), (0, 0));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn delete_all_cascades_attempts_warnings_and_audio_artifacts() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let mut db = HistorySqlite::open(root.join("history.sqlite"), root.join("audio")).unwrap();
        let audio = AudioReferenceId::new(79).unwrap();
        db.stage(audio, b"delete all audio").unwrap();
        assert!(db.commit(audio).unwrap());
        let record_id = DictationRecordId::new(80).unwrap();
        let mut record = DictationRecord::new(record_id, SessionId::new(81).unwrap());
        record.set_recorded_audio(RecordedAudio::new(audio, true));
        record.set_warnings(vec![Warning::LowVolume]);
        let mut attempt = RecognitionAttempt::new(
            RecognitionAttemptId::new(82).unwrap(),
            Revision::first(),
            ConfigurationId::new(83).unwrap(),
        );
        attempt.accept_final(RawTranscript::new("delete all attempt"));
        record.append_attempt(attempt);
        record.set_outcome(TerminalOutcome::ManualDeliveryRequired);
        db.persist(HistoryPersistRequest { record_id, record })
            .unwrap();

        let report = db.delete_all_records().unwrap();
        assert_eq!((report.records, report.artifacts_queued), (1, 1));
        assert_eq!(db.load_record(record_id).unwrap(), None);
        assert_eq!(
            db.connection()
                .query_row("SELECT COUNT(*) FROM recognition_attempts", [], |row| row
                    .get::<_, i64>(
                    0
                ))
                .unwrap(),
            0
        );
        assert_eq!(
            db.connection()
                .query_row("SELECT COUNT(*) FROM dictation_warnings", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert!(!db.checked_path(&db.committed_dir(), audio).exists());
        let _ = fs::remove_dir_all(root);
    }
}
