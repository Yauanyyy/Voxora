use crate::{
    ApplicationProfileId, ConfigurationId, CredentialReferenceId, DurationLimit, HotwordGroupId,
    HotwordId, ProcessingRuleId, PromptPresetId,
};
use std::fmt;

/// Immutable built-in Prompt catalog entries.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BuiltInPromptId {
    OriginalTextCleanup,
    ConciseExpression,
    FormalExpression,
}

impl BuiltInPromptId {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::OriginalTextCleanup => "original_text_cleanup",
            Self::ConciseExpression => "concise_expression",
            Self::FormalExpression => "formal_expression",
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::OriginalTextCleanup => "Original Text Cleanup",
            Self::ConciseExpression => "Concise Expression",
            Self::FormalExpression => "Formal Expression",
        }
    }

    #[must_use]
    pub const fn content(self) -> &'static str {
        match self {
            Self::OriginalTextCleanup => {
                "Clean up the dictated text while preserving its original meaning, facts, tone, language, and level of detail. Correct obvious recognition, grammar, and punctuation issues. Return only the revised text."
            }
            Self::ConciseExpression => {
                "Rewrite the dictated text concisely while preserving its meaning and essential details. Remove repetition and filler. Return only the revised text."
            }
            Self::FormalExpression => {
                "Rewrite the dictated text in a clear, formal style while preserving its meaning and facts. Return only the revised text."
            }
        }
    }

    #[must_use]
    pub const fn all() -> [Self; 3] {
        [
            Self::OriginalTextCleanup,
            Self::ConciseExpression,
            Self::FormalExpression,
        ]
    }
}

/// A normalized non-empty global Prompt shortcut.
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PromptShortcut(String);

impl PromptShortcut {
    /// Normalize a shortcut by trimming each `+`-separated component and
    /// joining it with one ASCII `+`.
    pub fn new(value: impl AsRef<str>) -> Option<Self> {
        let mut parts = Vec::new();
        for part in value.as_ref().split('+') {
            let part = part.trim();
            if part.is_empty() {
                return None;
            }
            parts.push(part.to_owned());
        }
        if parts.is_empty() {
            None
        } else {
            Some(Self(parts.join("+")))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for PromptShortcut {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PromptShortcut(<redacted>)")
    }
}

impl fmt::Display for PromptShortcut {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A Prompt Preset stored by the persistence adapter.
#[derive(Clone, Eq, PartialEq)]
pub struct PromptPreset {
    id: PromptPresetId,
    name: String,
    content: String,
    built_in: Option<BuiltInPromptId>,
    shortcut: Option<PromptShortcut>,
}

impl fmt::Debug for PromptPreset {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PromptPreset")
            .field("id", &self.id)
            .field("built_in", &self.built_in)
            .field("shortcut", &self.shortcut)
            .field("name", &"<redacted>")
            .field("content", &"<redacted>")
            .finish()
    }
}

impl PromptPreset {
    #[must_use]
    pub fn custom(id: PromptPresetId, name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            content: content.into(),
            built_in: None,
            shortcut: None,
        }
    }

    #[must_use]
    pub fn built_in(id: PromptPresetId, built_in: BuiltInPromptId) -> Self {
        Self {
            id,
            name: built_in.name().to_owned(),
            content: built_in.content().to_owned(),
            built_in: Some(built_in),
            shortcut: None,
        }
    }

    #[must_use]
    pub const fn id(&self) -> PromptPresetId {
        self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    #[must_use]
    pub const fn built_in_kind(&self) -> Option<BuiltInPromptId> {
        self.built_in
    }

    #[must_use]
    pub const fn is_built_in(&self) -> bool {
        self.built_in.is_some()
    }

    #[must_use]
    pub fn shortcut(&self) -> Option<&PromptShortcut> {
        self.shortcut.as_ref()
    }

    /// Built-ins are immutable; custom presets accept a normalized shortcut.
    pub fn set_shortcut(&mut self, shortcut: Option<PromptShortcut>) -> bool {
        if self.is_built_in() {
            return false;
        }
        self.shortcut = shortcut;
        true
    }

    /// Built-ins cannot be edited or deleted.
    pub fn edit(&mut self, name: impl Into<String>, content: impl Into<String>) -> bool {
        if self.is_built_in() {
            return false;
        }
        self.name = name.into();
        self.content = content.into();
        true
    }
}

/// The two project-maintained local text rules.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BuiltInRuleId {
    RemoveTrailingSentencePunctuation,
    ReplaceConversationalPunctuationWithSpaces,
}

impl BuiltInRuleId {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::RemoveTrailingSentencePunctuation => "remove_trailing_sentence_punctuation",
            Self::ReplaceConversationalPunctuationWithSpaces => {
                "replace_conversational_punctuation_with_spaces"
            }
        }
    }

    #[must_use]
    pub const fn all() -> [Self; 2] {
        [
            Self::RemoveTrailingSentencePunctuation,
            Self::ReplaceConversationalPunctuationWithSpaces,
        ]
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RuleOverride {
    Inherit,
    ForceEnabled,
    ForceDisabled,
}

#[must_use]
pub fn apply_rule(rule: BuiltInRuleId, input: &str) -> String {
    match rule {
        BuiltInRuleId::RemoveTrailingSentencePunctuation => {
            remove_trailing_sentence_punctuation(input)
        }
        BuiltInRuleId::ReplaceConversationalPunctuationWithSpaces => {
            replace_conversational_punctuation_with_spaces(input)
        }
    }
}

const CLOSING_QUOTES: &[char] = &[
    '\"', '\'', '”', '’', '»', '›', '」', '』', '】', '》', '〕', '〉',
];
const TRAILING_SENTENCE_PUNCTUATION: &[char] = &['.', '?', '!', '…', '。', '？', '！'];

#[must_use]
pub fn remove_trailing_sentence_punctuation(input: &str) -> String {
    let (body, trailing_whitespace) = split_trailing_whitespace(input);
    let mut quote_start = body.len();
    while quote_start > 0 {
        let Some((index, character)) = body[..quote_start].char_indices().next_back() else {
            break;
        };
        if CLOSING_QUOTES.contains(&character) {
            quote_start = index;
        } else {
            break;
        }
    }
    let (mut core, quotes) = body.split_at(quote_start);
    while let Some(character) = core.chars().next_back() {
        if !TRAILING_SENTENCE_PUNCTUATION.contains(&character) {
            break;
        }
        let end = core.len() - character.len_utf8();
        core = &core[..end];
    }
    let mut result = String::with_capacity(input.len());
    result.push_str(core);
    result.push_str(quotes);
    result.push_str(trailing_whitespace);
    result
}

fn split_trailing_whitespace(input: &str) -> (&str, &str) {
    let mut split = input.len();
    while split > 0 {
        let Some((index, character)) = input[..split].char_indices().next_back() else {
            break;
        };
        if character.is_whitespace() {
            split = index;
        } else {
            break;
        }
    }
    (&input[..split], &input[split..])
}

fn is_ascii_alphanumeric(character: Option<char>) -> bool {
    character.is_some_and(|value| value.is_ascii_alphanumeric())
}

fn candidate_punctuation(character: char) -> bool {
    matches!(
        character,
        '.' | ',' | ';' | ':' | '!' | '?' | '、' | '，' | '；' | '：' | '。' | '！' | '？' | '…'
    )
}

fn preserve_candidate(chars: &[char], index: usize, token_is_url: bool) -> bool {
    let character = chars[index];
    if token_is_url {
        return true;
    }
    let previous = index
        .checked_sub(1)
        .and_then(|position| chars.get(position))
        .copied();
    let next = chars.get(index + 1).copied();
    match character {
        '.' => is_ascii_alphanumeric(previous) && is_ascii_alphanumeric(next),
        ',' => {
            previous.is_some_and(|value| value.is_ascii_digit())
                && next.is_some_and(|value| value.is_ascii_digit())
        }
        ':' => {
            (previous.is_some_and(|value| value.is_ascii_digit())
                && next.is_some_and(|value| value.is_ascii_digit()))
                || matches!(next, Some('/' | '\\'))
        }
        _ => false,
    }
}

#[must_use]
pub fn replace_conversational_punctuation_with_spaces(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut pending_space = false;
    for token in input.split_inclusive(char::is_whitespace) {
        let (word, whitespace) = split_trailing_whitespace(token);
        let chars: Vec<char> = word.chars().collect();
        let token_is_url = word.contains("://");
        for (index, character) in chars.iter().copied().enumerate() {
            if candidate_punctuation(character) && !preserve_candidate(&chars, index, token_is_url)
            {
                pending_space = true;
                continue;
            }
            if pending_space && !output.is_empty() {
                output.push(' ');
            }
            pending_space = false;
            output.push(character);
        }
        if !whitespace.is_empty() {
            pending_space = true;
        }
    }
    output.trim().to_owned()
}

#[derive(Clone, Eq, PartialEq)]
pub struct Hotword {
    id: HotwordId,
    text: String,
}

impl fmt::Debug for Hotword {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Hotword")
            .field("id", &self.id)
            .field("text", &"<redacted>")
            .finish()
    }
}

impl Hotword {
    #[must_use]
    pub fn new(id: HotwordId, text: impl Into<String>) -> Self {
        Self {
            id,
            text: text.into(),
        }
    }
    #[must_use]
    pub const fn id(&self) -> HotwordId {
        self.id
    }
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct HotwordGroup {
    id: HotwordGroupId,
    name: String,
    enabled: bool,
    items: Vec<Hotword>,
}

impl fmt::Debug for HotwordGroup {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HotwordGroup")
            .field("id", &self.id)
            .field("name", &"<redacted>")
            .field("enabled", &self.enabled)
            .field("items", &format_args!("<{} items>", self.items.len()))
            .finish()
    }
}

impl HotwordGroup {
    #[must_use]
    pub fn new(
        id: HotwordGroupId,
        name: impl Into<String>,
        enabled: bool,
        items: Vec<Hotword>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            enabled,
            items,
        }
    }
    #[must_use]
    pub const fn id(&self) -> HotwordGroupId {
        self.id
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn enabled(&self) -> bool {
        self.enabled
    }
    #[must_use]
    pub fn items(&self) -> &[Hotword] {
        &self.items
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct HotwordSelection {
    selected: Vec<Hotword>,
    used: usize,
    total: usize,
}

impl fmt::Debug for HotwordSelection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HotwordSelection")
            .field("used", &self.used)
            .field("total", &self.total)
            .field("selected", &format_args!("<{} items>", self.selected.len()))
            .finish()
    }
}

impl HotwordSelection {
    #[must_use]
    pub fn selected(&self) -> &[Hotword] {
        &self.selected
    }
    #[must_use]
    pub const fn used(&self) -> usize {
        self.used
    }
    #[must_use]
    pub const fn total(&self) -> usize {
        self.total
    }
}

#[must_use]
pub fn select_hotwords(
    groups: &[HotwordGroup],
    max_items: usize,
    max_utf8_bytes: usize,
) -> HotwordSelection {
    let mut ordered: Vec<(&HotwordGroup, &Hotword)> = groups
        .iter()
        .filter(|group| group.enabled)
        .flat_map(|group| group.items.iter().map(move |item| (group, item)))
        .collect();
    ordered.sort_by(|(left_group, left), (right_group, right)| {
        left_group
            .id
            .cmp(&right_group.id)
            .then_with(|| left.id.cmp(&right.id))
    });
    let total = ordered.len();
    let mut selected = Vec::new();
    let mut bytes = 0usize;
    for (_, item) in ordered {
        if selected.len() >= max_items {
            break;
        }
        let item_bytes = item.text.len();
        if item_bytes > max_utf8_bytes.saturating_sub(bytes) {
            continue;
        }
        selected.push(item.clone());
        bytes += item_bytes;
    }
    HotwordSelection {
        used: selected.len(),
        total,
        selected,
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BaseUrlError {
    Relative,
    UnsupportedScheme,
    MissingHost,
    UserInfo,
    Query,
    Fragment,
    InvalidPort,
    InsecureEndpoint,
    InvalidCharacter,
}

impl BaseUrlError {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Relative => "relative",
            Self::UnsupportedScheme => "unsupported_scheme",
            Self::MissingHost => "missing_host",
            Self::UserInfo => "userinfo",
            Self::Query => "query",
            Self::Fragment => "fragment",
            Self::InvalidPort => "invalid_port",
            Self::InsecureEndpoint => "insecure_endpoint",
            Self::InvalidCharacter => "invalid_character",
        }
    }
}

/// A validated endpoint containing only scheme, host, optional port, and path.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct BaseUrl(String);

impl BaseUrl {
    /// Parse and validate a persisted endpoint.
    ///
    /// # Errors
    ///
    /// Returns a stable [`BaseUrlError`] without retaining or echoing the input.
    pub fn parse(value: impl AsRef<str>) -> Result<Self, BaseUrlError> {
        let value = value.as_ref();
        if value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        {
            return Err(BaseUrlError::InvalidCharacter);
        }
        if value
            .split_once("://")
            .and_then(|(_, remainder)| remainder.split(['/', '?', '#']).next())
            .is_some_and(|authority| authority.ends_with(':'))
        {
            return Err(BaseUrlError::InvalidPort);
        }
        let parsed = url::Url::parse(value).map_err(|error| match error {
            url::ParseError::RelativeUrlWithoutBase => BaseUrlError::Relative,
            url::ParseError::EmptyHost => BaseUrlError::MissingHost,
            url::ParseError::InvalidPort => BaseUrlError::InvalidPort,
            _ => BaseUrlError::InvalidCharacter,
        })?;
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(BaseUrlError::UnsupportedScheme);
        }
        if parsed.host().is_none() {
            return Err(BaseUrlError::MissingHost);
        }
        if parsed.port() == Some(0) {
            return Err(BaseUrlError::InvalidPort);
        }
        if !parsed.username().is_empty() || parsed.password().is_some() {
            return Err(BaseUrlError::UserInfo);
        }
        if parsed.query().is_some() {
            return Err(BaseUrlError::Query);
        }
        if parsed.fragment().is_some() {
            return Err(BaseUrlError::Fragment);
        }
        if parsed.scheme() == "http"
            && !parsed.host().is_some_and(|host| match host {
                url::Host::Domain(domain) => domain.eq_ignore_ascii_case("localhost"),
                url::Host::Ipv4(address) => address.is_loopback(),
                url::Host::Ipv6(address) => address.is_loopback(),
            })
        {
            return Err(BaseUrlError::InsecureEndpoint);
        }
        Ok(Self(parsed.into()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for BaseUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BaseUrl(<redacted>)")
    }
}

impl fmt::Display for BaseUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<base-url>")
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct LanguageModelConfiguration {
    id: ConfigurationId,
    name: String,
    base_url: BaseUrl,
    credential_reference: CredentialReferenceId,
    model: String,
    timeout: DurationLimit,
    reasoning_mode: ReasoningMode,
}

/// Portable recognition settings. Provider adapters interpret `provider_code`
/// and `model`; persistence sees only validated endpoints and opaque credentials.
#[derive(Clone, Eq, PartialEq)]
pub struct RecognitionConfiguration {
    id: ConfigurationId,
    name: String,
    provider_code: String,
    base_url: Option<BaseUrl>,
    credential_reference: Option<CredentialReferenceId>,
    model: String,
}

impl fmt::Debug for RecognitionConfiguration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RecognitionConfiguration")
            .field("id", &self.id)
            .field("name", &"<redacted>")
            .field("provider_code", &"<redacted>")
            .field("base_url", &self.base_url)
            .field("credential_reference", &self.credential_reference)
            .field("model", &"<redacted>")
            .finish()
    }
}

impl RecognitionConfiguration {
    #[must_use]
    pub fn new(
        id: ConfigurationId,
        name: impl Into<String>,
        provider_code: impl Into<String>,
        base_url: Option<BaseUrl>,
        credential_reference: Option<CredentialReferenceId>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            provider_code: provider_code.into(),
            base_url,
            credential_reference,
            model: model.into(),
        }
    }

    #[must_use]
    pub const fn id(&self) -> ConfigurationId {
        self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn provider_code(&self) -> &str {
        &self.provider_code
    }

    #[must_use]
    pub const fn base_url(&self) -> Option<&BaseUrl> {
        self.base_url.as_ref()
    }

    #[must_use]
    pub const fn credential_reference(&self) -> Option<CredentialReferenceId> {
        self.credential_reference
    }

    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }
}

/// One entry in the global processing order. The optional LLM step remains
/// provider-independent and may appear at most once.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ProcessingStepConfiguration {
    BuiltIn(BuiltInRuleId),
    LanguageModel,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessingOrder(Vec<ProcessingStepConfiguration>);

impl ProcessingOrder {
    #[must_use]
    pub fn new(steps: Vec<ProcessingStepConfiguration>) -> Option<Self> {
        for rule in BuiltInRuleId::all() {
            if steps
                .iter()
                .filter(|step| **step == ProcessingStepConfiguration::BuiltIn(rule))
                .count()
                != 1
            {
                return None;
            }
        }
        if steps
            .iter()
            .filter(|step| **step == ProcessingStepConfiguration::LanguageModel)
            .count()
            > 1
        {
            return None;
        }
        Some(Self(steps))
    }

    #[must_use]
    pub fn steps(&self) -> &[ProcessingStepConfiguration] {
        &self.0
    }
}

impl Default for ProcessingOrder {
    fn default() -> Self {
        Self(vec![
            ProcessingStepConfiguration::BuiltIn(BuiltInRuleId::RemoveTrailingSentencePunctuation),
            ProcessingStepConfiguration::BuiltIn(
                BuiltInRuleId::ReplaceConversationalPunctuationWithSpaces,
            ),
        ])
    }
}

impl fmt::Debug for LanguageModelConfiguration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LanguageModelConfiguration")
            .field("id", &self.id)
            .field("name", &"<redacted>")
            .field("base_url", &self.base_url)
            .field("credential_reference", &self.credential_reference)
            .field("model", &"<redacted>")
            .field("timeout", &self.timeout)
            .field("reasoning_mode", &self.reasoning_mode)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReasoningMode {
    ProviderDefault,
    Disabled,
    Enabled,
}

impl LanguageModelConfiguration {
    pub fn new(
        id: ConfigurationId,
        name: impl Into<String>,
        base_url: BaseUrl,
        credential_reference: CredentialReferenceId,
        model: impl Into<String>,
        timeout: DurationLimit,
        reasoning_mode: ReasoningMode,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            base_url,
            credential_reference,
            model: model.into(),
            timeout,
            reasoning_mode,
        }
    }
    #[must_use]
    pub const fn id(&self) -> ConfigurationId {
        self.id
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn base_url(&self) -> &BaseUrl {
        &self.base_url
    }
    #[must_use]
    pub const fn credential_reference(&self) -> CredentialReferenceId {
        self.credential_reference
    }
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }
    #[must_use]
    pub const fn timeout(&self) -> DurationLimit {
        self.timeout
    }
    #[must_use]
    pub const fn reasoning_mode(&self) -> ReasoningMode {
        self.reasoning_mode
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RetentionPolicy {
    pub text_enabled: bool,
    pub audio_enabled: bool,
    pub text_days: u32,
    pub audio_days: u32,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            text_enabled: true,
            audio_enabled: true,
            text_days: 30,
            audio_days: 30,
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ApplicationProfile {
    id: ApplicationProfileId,
    identity: String,
    enabled: bool,
    prompt: Option<PromptPresetId>,
    rule_overrides: Vec<(ProcessingRuleId, RuleOverride)>,
}

impl fmt::Debug for ApplicationProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApplicationProfile")
            .field("id", &self.id)
            .field("identity", &"<redacted>")
            .field("enabled", &self.enabled)
            .field("prompt", &self.prompt)
            .field("rule_overrides", &self.rule_overrides)
            .finish()
    }
}

impl ApplicationProfile {
    #[must_use]
    pub fn new(id: ApplicationProfileId, identity: impl Into<String>, enabled: bool) -> Self {
        Self {
            id,
            identity: identity.into(),
            enabled,
            prompt: None,
            rule_overrides: Vec::new(),
        }
    }
    #[must_use]
    pub const fn id(&self) -> ApplicationProfileId {
        self.id
    }
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }
    #[must_use]
    pub const fn enabled(&self) -> bool {
        self.enabled
    }
    #[must_use]
    pub const fn prompt(&self) -> Option<PromptPresetId> {
        self.prompt
    }
    pub fn set_prompt(&mut self, prompt: Option<PromptPresetId>) {
        self.prompt = prompt;
    }
    #[must_use]
    pub fn rule_overrides(&self) -> &[(ProcessingRuleId, RuleOverride)] {
        &self.rule_overrides
    }
    pub fn set_rule_override(&mut self, rule: ProcessingRuleId, value: RuleOverride) {
        if let Some(existing) = self.rule_overrides.iter_mut().find(|(id, _)| *id == rule) {
            existing.1 = value;
        } else {
            self.rule_overrides.push((rule, value));
        }
        self.rule_overrides.sort_by_key(|left| left.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_prompts_and_copy_shortcuts_are_immutable_and_normalized() {
        let built_in = PromptPreset::built_in(
            PromptPresetId::new(1).unwrap(),
            BuiltInPromptId::OriginalTextCleanup,
        );
        assert!(built_in.is_built_in());
        assert!(!format!("{:?}", built_in.content()).is_empty());
        assert_eq!(
            PromptShortcut::new(" Ctrl + Shift + P ").unwrap().as_str(),
            "Ctrl+Shift+P"
        );
        assert!(PromptShortcut::new(" + ").is_none());
    }

    #[test]
    fn punctuation_rules_preserve_quotes_and_technical_tokens() {
        assert_eq!(
            remove_trailing_sentence_punctuation("Hello!!!\"  "),
            "Hello\"  "
        );
        assert_eq!(
            replace_conversational_punctuation_with_spaces("Wait!!!  now..."),
            "Wait now"
        );
        assert_eq!(
            replace_conversational_punctuation_with_spaces(
                "v1.2.3 1,234 12:30 https://x.test/a?b=1"
            ),
            "v1.2.3 1,234 12:30 https://x.test/a?b=1"
        );
        assert_eq!(
            remove_trailing_sentence_punctuation("你好？！』\t"),
            "你好』\t"
        );
        assert_eq!(
            replace_conversational_punctuation_with_spaces(
                "你好，世界！ U.S.A. example.test C:\\temp；版本v2.1"
            ),
            "你好 世界 U.S.A example.test C:\\temp 版本v2.1"
        );
    }

    #[test]
    fn hotword_selection_is_stable_and_reports_omissions() {
        let groups = vec![
            HotwordGroup::new(
                HotwordGroupId::new(2).unwrap(),
                "b",
                true,
                vec![Hotword::new(HotwordId::new(2).unwrap(), "two")],
            ),
            HotwordGroup::new(
                HotwordGroupId::new(1).unwrap(),
                "a",
                true,
                vec![Hotword::new(HotwordId::new(1).unwrap(), "one")],
            ),
        ];
        let selection = select_hotwords(&groups, 1, 10);
        assert_eq!(selection.selected()[0].text(), "one");
        assert_eq!((selection.used(), selection.total()), (1, 2));
    }

    #[test]
    fn hotword_selection_respects_item_and_utf8_byte_limits() {
        let groups = vec![
            HotwordGroup::new(
                HotwordGroupId::new(2).unwrap(),
                "disabled",
                false,
                vec![Hotword::new(HotwordId::new(1).unwrap(), "ignored")],
            ),
            HotwordGroup::new(
                HotwordGroupId::new(1).unwrap(),
                "enabled",
                true,
                vec![
                    Hotword::new(HotwordId::new(1).unwrap(), "oversized"),
                    Hotword::new(HotwordId::new(2).unwrap(), "词"),
                    Hotword::new(HotwordId::new(3).unwrap(), "ab"),
                ],
            ),
        ];

        let by_bytes = select_hotwords(&groups, 3, 5);
        assert_eq!(
            by_bytes
                .selected()
                .iter()
                .map(Hotword::text)
                .collect::<Vec<_>>(),
            vec!["词", "ab"]
        );
        assert_eq!((by_bytes.used(), by_bytes.total()), (2, 3));

        let by_items = select_hotwords(&groups, 1, usize::MAX);
        assert_eq!(by_items.selected()[0].text(), "oversized");
        assert_eq!((by_items.used(), by_items.total()), (1, 3));
        assert_eq!(select_hotwords(&groups, 0, usize::MAX).used(), 0);
    }

    #[test]
    fn base_url_validation_is_fail_closed_and_sanitized() {
        for valid in [
            "https://example.test/v1",
            "https://example.test:8443/v1",
            "http://localhost:1234",
            "http://127.0.0.1/path",
            "http://[::1]:8080/path",
        ] {
            assert!(BaseUrl::parse(valid).is_ok(), "expected valid Base URL");
        }

        for (invalid, expected) in [
            ("relative/path", BaseUrlError::Relative),
            ("https://", BaseUrlError::MissingHost),
            ("ftp://example.test", BaseUrlError::UnsupportedScheme),
            ("http://example.test", BaseUrlError::InsecureEndpoint),
            ("https://user@example.test", BaseUrlError::UserInfo),
            ("https://user:pass@example.test", BaseUrlError::UserInfo),
            ("https://example.test?secret=1", BaseUrlError::Query),
            ("https://example.test/#private", BaseUrlError::Fragment),
            ("https://example.test:", BaseUrlError::InvalidPort),
            ("https://example.test:0", BaseUrlError::InvalidPort),
            ("https://example.test:65536", BaseUrlError::InvalidPort),
            ("https://example.test:invalid", BaseUrlError::InvalidPort),
            ("https://example.test/a b", BaseUrlError::InvalidCharacter),
        ] {
            let error = BaseUrl::parse(invalid).expect_err("expected invalid Base URL");
            assert_eq!(error, expected);
            assert!(!format!("{error:?}").contains(invalid));
        }
    }
}
