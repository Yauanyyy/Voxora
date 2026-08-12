use std::fmt;

use crate::{ConfigurationId, RawTranscript};

/// Errors raised while constructing a portable processing plan.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ProcessingPlanError {
    MultipleLanguageModelSteps,
}

impl fmt::Display for ProcessingPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MultipleLanguageModelSteps => {
                formatter.write_str("processing plan contains multiple language-model steps")
            }
        }
    }
}

impl std::error::Error for ProcessingPlanError {}

/// A portable ordered processing step description.  M3 models the order but
/// deliberately leaves the actual punctuation catalog/algorithms to M4.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessingStep {
    BuiltIn {
        rule_id: ConfigurationId,
        enabled: bool,
    },
    LanguageModel {
        configuration_id: Option<ConfigurationId>,
        enabled: bool,
    },
}

impl ProcessingStep {
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        match self {
            Self::BuiltIn { enabled, .. } => *enabled,
            Self::LanguageModel {
                configuration_id,
                enabled,
            } => *enabled && configuration_id.is_some(),
        }
    }
}

/// The global pipeline order plus the initial Raw Transcript.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessingPlan {
    raw_transcript: RawTranscript,
    steps: Vec<ProcessingStep>,
}

impl ProcessingPlan {
    /// Construct a plan while enforcing the single optional LLM-step rule.
    ///
    /// # Errors
    ///
    /// Returns `MultipleLanguageModelSteps` when the ordered plan contains more
    /// than one language-model step, regardless of whether those steps are enabled.
    pub fn try_new(
        raw_transcript: RawTranscript,
        steps: Vec<ProcessingStep>,
    ) -> Result<Self, ProcessingPlanError> {
        if steps
            .iter()
            .filter(|step| matches!(step, ProcessingStep::LanguageModel { .. }))
            .count()
            > 1
        {
            return Err(ProcessingPlanError::MultipleLanguageModelSteps);
        }
        Ok(Self {
            raw_transcript,
            steps,
        })
    }

    /// Alias retained for callers that use the shorter constructor name.
    ///
    /// # Errors
    ///
    /// Returns the same validation error as [`Self::try_new`].
    pub fn new(
        raw_transcript: RawTranscript,
        steps: Vec<ProcessingStep>,
    ) -> Result<Self, ProcessingPlanError> {
        Self::try_new(raw_transcript, steps)
    }

    #[must_use]
    pub const fn raw_transcript(&self) -> &RawTranscript {
        &self.raw_transcript
    }

    #[must_use]
    pub fn steps(&self) -> &[ProcessingStep] {
        &self.steps
    }

    /// Rebind the plan to the recognition result while preserving the configured order.
    #[must_use]
    pub fn with_raw_transcript(&self, raw_transcript: RawTranscript) -> Self {
        Self {
            raw_transcript,
            steps: self.steps.clone(),
        }
    }
}
