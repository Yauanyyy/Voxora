#![allow(clippy::missing_errors_doc)]

use voice_core::{
    BaseUrl, BaseUrlError, HotwordGroup, HotwordSelection, LanguageModelConfiguration,
    PromptPreset, PromptPresetId, PromptShortcut, RecognitionConfiguration, RetentionPolicy,
    RuleOverride, select_hotwords,
};
use voice_ports::{
    AudioArtifactStorePort, AudioMaintenanceReport, ConfigurationStorePort, HistoryDeletionReport,
    HistoryMaintenancePort, PortResult, PromptDeleteReport, PromptStorePort, RetentionReport,
};

/// Bounded configuration service: validation happens before the port call.
pub struct ConfigurationService<'a> {
    configuration: &'a mut dyn ConfigurationStorePort,
}

impl<'a> ConfigurationService<'a> {
    pub fn new(configuration: &'a mut dyn ConfigurationStorePort) -> Self {
        Self { configuration }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn save_llm_configuration(
        &mut self,
        id: voice_core::ConfigurationId,
        name: impl Into<String>,
        base_url: &str,
        credential_reference: voice_core::CredentialReferenceId,
        model: impl Into<String>,
        timeout: voice_core::DurationLimit,
        reasoning_mode: voice_core::ReasoningMode,
    ) -> Result<(), ConfigurationError> {
        let base_url = BaseUrl::parse(base_url).map_err(ConfigurationError::BaseUrl)?;
        self.configuration
            .save_llm_configuration(LanguageModelConfiguration::new(
                id,
                name,
                base_url,
                credential_reference,
                model,
                timeout,
                reasoning_mode,
            ))
            .map_err(ConfigurationError::Persistence)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn save_recognition_configuration(
        &mut self,
        id: voice_core::ConfigurationId,
        name: impl Into<String>,
        provider_code: impl Into<String>,
        base_url: Option<&str>,
        credential_reference: Option<voice_core::CredentialReferenceId>,
        model: impl Into<String>,
    ) -> Result<(), ConfigurationError> {
        let base_url = base_url
            .map(BaseUrl::parse)
            .transpose()
            .map_err(ConfigurationError::BaseUrl)?;
        self.configuration
            .save_recognition_configuration(RecognitionConfiguration::new(
                id,
                name,
                provider_code,
                base_url,
                credential_reference,
                model,
            ))
            .map_err(ConfigurationError::Persistence)
    }

    pub fn set_active_recognition_configuration(
        &mut self,
        id: Option<voice_core::ConfigurationId>,
    ) -> Result<(), ConfigurationError> {
        self.configuration
            .set_active_recognition_configuration(id)
            .map_err(ConfigurationError::Persistence)
    }

    pub fn set_active_llm_configuration(
        &mut self,
        id: Option<voice_core::ConfigurationId>,
    ) -> Result<(), ConfigurationError> {
        self.configuration
            .set_active_llm_configuration(id)
            .map_err(ConfigurationError::Persistence)
    }

    pub fn save_retention_policy(
        &mut self,
        policy: RetentionPolicy,
    ) -> Result<(), ConfigurationError> {
        self.configuration
            .save_retention_policy(policy)
            .map_err(ConfigurationError::Persistence)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigurationError {
    BaseUrl(BaseUrlError),
    Persistence(voice_core::SanitizedFailure),
}

/// Bounded history-maintenance service. It owns only the history port borrow.
pub struct HistoryMaintenanceService<'a> {
    history: &'a mut dyn HistoryMaintenancePort,
}

impl<'a> HistoryMaintenanceService<'a> {
    pub fn new(history: &'a mut dyn HistoryMaintenancePort) -> Self {
        Self { history }
    }

    pub fn delete_record(
        &mut self,
        record: voice_core::DictationRecordId,
    ) -> PortResult<HistoryDeletionReport> {
        self.history.delete_record(record)
    }

    pub fn delete_all_records(&mut self) -> PortResult<HistoryDeletionReport> {
        self.history.delete_all_records()
    }

    pub fn apply_retention(&mut self, now: voice_core::Timestamp) -> PortResult<RetentionReport> {
        self.history.apply_retention(now)
    }

    pub fn backup(&mut self) -> PortResult<Vec<u8>> {
        self.history.backup()
    }
}

/// Bounded audio-maintenance service. It owns only the audio-artifact port borrow.
pub struct AudioMaintenanceService<'a> {
    audio: &'a mut dyn AudioArtifactStorePort,
}

impl<'a> AudioMaintenanceService<'a> {
    pub fn new(audio: &'a mut dyn AudioArtifactStorePort) -> Self {
        Self { audio }
    }

    pub fn startup(&mut self) -> PortResult<AudioMaintenanceReport> {
        self.audio.startup_maintenance()
    }
}

/// Prompt use cases including deterministic copy naming and explicit deletion confirmation.
pub struct PromptService<'a> {
    prompts: &'a mut dyn PromptStorePort,
}

impl<'a> PromptService<'a> {
    pub fn new(prompts: &'a mut dyn PromptStorePort) -> Self {
        Self { prompts }
    }

    pub fn copy_prompt(
        &mut self,
        source: PromptPresetId,
        id: PromptPresetId,
    ) -> PortResult<PromptPreset> {
        let prompts = self.prompts.list_prompts()?;
        let original = prompts
            .iter()
            .find(|prompt| prompt.id() == source)
            .cloned()
            .ok_or_else(|| {
                voice_core::SanitizedFailure::from_boundary(
                    voice_core::FailureStage::Persistence,
                    voice_core::FailureCode::PersistenceUnavailable,
                    voice_core::RetryMeaning::NotRetryable,
                    voice_core::DeliveryCertainty::NotApplicable,
                )
            })?;
        let base = format!("{} Copy", original.name());
        let mut name = base.clone();
        let mut suffix = 2u32;
        while prompts.iter().any(|prompt| prompt.name() == name) {
            name = format!("{base} {suffix}");
            suffix = suffix.saturating_add(1);
        }
        let copy = PromptPreset::custom(id, name, original.content());
        self.prompts.save_prompt(copy.clone())?;
        Ok(copy)
    }

    pub fn activate_shortcut(
        &mut self,
        shortcut: &PromptShortcut,
    ) -> PortResult<Option<PromptPresetId>> {
        self.prompts.activate_shortcut(shortcut)
    }

    pub fn delete_prompt(
        &mut self,
        prompt: PromptPresetId,
        confirm: bool,
    ) -> PortResult<PromptDeleteReport> {
        self.prompts.delete_prompt(prompt, confirm)
    }
}

/// Stable limit-aware Hotword selection owned by the application boundary.
#[must_use]
pub fn choose_hotwords(
    groups: &[HotwordGroup],
    max_items: usize,
    max_bytes: usize,
) -> HotwordSelection {
    select_hotwords(groups, max_items, max_bytes)
}

/// Resolve a profile's rule setting without allowing profiles to reorder rules.
#[must_use]
pub fn resolve_rule(default_enabled: bool, override_value: RuleOverride) -> bool {
    match override_value {
        RuleOverride::Inherit => default_enabled,
        RuleOverride::ForceEnabled => true,
        RuleOverride::ForceDisabled => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    use voice_core::{FailureCode, FailureStage, PromptPresetId, RetryMeaning, SanitizedFailure};
    use voice_ports::{
        ConfigurationStorePort, FakeAudioArtifactStore, FakeHistoryMaintenance, FakePromptStore,
        PortCall,
    };

    #[derive(Default)]
    struct CountingConfigurationStore {
        save_calls: usize,
    }

    impl ConfigurationStorePort for CountingConfigurationStore {
        fn load_recognition_configurations(
            &mut self,
        ) -> PortResult<Vec<voice_core::RecognitionConfiguration>> {
            Ok(Vec::new())
        }
        fn save_recognition_configuration(
            &mut self,
            _configuration: voice_core::RecognitionConfiguration,
        ) -> PortResult<()> {
            self.save_calls += 1;
            Ok(())
        }
        fn set_active_recognition_configuration(
            &mut self,
            _configuration: Option<voice_core::ConfigurationId>,
        ) -> PortResult<()> {
            Ok(())
        }
        fn active_recognition_configuration(
            &mut self,
        ) -> PortResult<Option<voice_core::ConfigurationId>> {
            Ok(None)
        }
        fn load_llm_configurations(
            &mut self,
        ) -> PortResult<Vec<voice_core::LanguageModelConfiguration>> {
            Ok(Vec::new())
        }
        fn save_llm_configuration(
            &mut self,
            _configuration: voice_core::LanguageModelConfiguration,
        ) -> PortResult<()> {
            self.save_calls += 1;
            Ok(())
        }
        fn set_active_llm_configuration(
            &mut self,
            _configuration: Option<voice_core::ConfigurationId>,
        ) -> PortResult<()> {
            Ok(())
        }
        fn active_llm_configuration(&mut self) -> PortResult<Option<voice_core::ConfigurationId>> {
            Ok(None)
        }
        fn load_retention_policy(&mut self) -> PortResult<voice_core::RetentionPolicy> {
            Ok(voice_core::RetentionPolicy::default())
        }
        fn save_retention_policy(
            &mut self,
            _policy: voice_core::RetentionPolicy,
        ) -> PortResult<()> {
            self.save_calls += 1;
            Ok(())
        }
    }

    #[test]
    fn rule_resolution_is_deterministic() {
        assert!(super::resolve_rule(true, voice_core::RuleOverride::Inherit));
        assert!(!super::resolve_rule(
            true,
            voice_core::RuleOverride::ForceDisabled
        ));
        assert!(super::resolve_rule(
            false,
            voice_core::RuleOverride::ForceEnabled
        ));
    }

    #[test]
    fn prompt_copy_uses_next_name_without_shortcut_or_activation() {
        let source_id = PromptPresetId::new(1).unwrap();
        let active_id = PromptPresetId::new(9).unwrap();
        let mut source = PromptPreset::custom(source_id, "Synthetic", "Synthetic content");
        assert!(source.set_shortcut(PromptShortcut::new("Ctrl + 9")));
        let mut store = FakePromptStore {
            prompts: vec![
                source,
                PromptPreset::custom(
                    PromptPresetId::new(2).unwrap(),
                    "Synthetic Copy",
                    "existing",
                ),
                PromptPreset::custom(
                    PromptPresetId::new(3).unwrap(),
                    "Synthetic Copy 2",
                    "existing",
                ),
            ],
            active: active_id,
        };

        let copy = PromptService::new(&mut store)
            .copy_prompt(source_id, PromptPresetId::new(4).unwrap())
            .unwrap();

        assert_eq!(copy.name(), "Synthetic Copy 3");
        assert_eq!(copy.content(), "Synthetic content");
        assert!(copy.shortcut().is_none());
        assert_eq!(store.active, active_id);
        assert_eq!(store.prompts.last(), Some(&copy));
    }

    #[test]
    fn maintenance_delegates_to_each_port_and_propagates_failures() {
        let record = voice_core::DictationRecordId::new(7).unwrap();
        let now = voice_core::Timestamp::new(42);
        let failure = SanitizedFailure::from_boundary(
            FailureStage::Persistence,
            FailureCode::PersistenceUnavailable,
            RetryMeaning::Retryable,
            voice_core::DeliveryCertainty::NotApplicable,
        );
        let mut history = FakeHistoryMaintenance {
            delete_record_results: VecDeque::from([Err(failure)]),
            ..FakeHistoryMaintenance::default()
        };
        let mut audio = FakeAudioArtifactStore::default();
        {
            let mut history_service = HistoryMaintenanceService::new(&mut history);

            assert_eq!(history_service.delete_record(record), Err(failure));
            assert_eq!(history_service.delete_all_records().unwrap().records, 0);
            assert_eq!(
                history_service
                    .apply_retention(now)
                    .unwrap()
                    .records_deleted,
                0
            );
            assert!(history_service.backup().unwrap().is_empty());
        }

        let mut audio_service = AudioMaintenanceService::new(&mut audio);
        assert_eq!(audio_service.startup().unwrap().queued_remaining, 0);
        assert_eq!(
            history.calls,
            vec![
                PortCall::HistoryDeleteRecord(record),
                PortCall::HistoryDeleteAll,
                PortCall::HistoryRetention(now),
                PortCall::HistoryBackup,
            ]
        );
        assert_eq!(audio.calls, vec![PortCall::AudioStartupMaintenance]);
    }

    #[test]
    fn rejected_credential_bearing_base_url_does_not_save() {
        let mut store = CountingConfigurationStore::default();
        let mut service = ConfigurationService::new(&mut store);
        let error = service
            .save_llm_configuration(
                voice_core::ConfigurationId::new(9).unwrap(),
                "synthetic",
                "https://user:synthetic-secret@example.test/v1",
                voice_core::CredentialReferenceId::new(10).unwrap(),
                "synthetic-model",
                voice_core::DurationLimit::from_seconds(3).unwrap(),
                voice_core::ReasoningMode::ProviderDefault,
            )
            .unwrap_err();
        assert_eq!(error, ConfigurationError::BaseUrl(BaseUrlError::UserInfo));
        assert_eq!(store.save_calls, 0);
    }
}
