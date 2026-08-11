# Voxora Dictation Context

Voxora turns a user-initiated voice recording into text for insertion into another application. This glossary defines the shared product language independently of UI, platform APIs, and provider implementations.

## Dictation

**Dictation Session**:
One user-initiated attempt that begins with audio capture and ends in insertion, cancellation, or a reported failure.
_Avoid_: Recording task, transcription job

**Recorded Audio**:
The audio captured during a Dictation Session.
_Avoid_: Audio history, temporary audio

**Partial Transcript**:
A provisional recognition result that may be replaced or corrected before recognition completes; it is not shown during normal recording and is preserved only as an explicitly incomplete result when final recognition fails.
_Avoid_: Raw transcript, final text

**Raw Transcript**:
The recognition result before optional text processing.
_Avoid_: Original text, draft text

**Processed Text**:
Text produced from a Raw Transcript by an enabled processing pipeline.
_Avoid_: AI text, polished transcript

**Final Text**:
The text selected for insertion or copying: Processed Text when processing succeeds, otherwise the Raw Transcript.
_Avoid_: Output text, result text

**Insertion Target**:
An eligible input destination in an application other than Voxora.
_Avoid_: Active window, HWND, focused app

**Recording Overlay**:
Voxora's non-target transient interface that presents recording feedback and then a generic processing state.
_Avoid_: Recording window, transcript preview

**Result Panel**:
A fallback interface that preserves Final Text for manual copying when automatic insertion is unavailable or unsafe.
_Avoid_: Mandatory preview, insertion target

## Configuration

**Recognition Provider**:
A supported cloud or local recognition capability, such as Doubao or a local model family.
_Avoid_: ASR account, recognition configuration

**Recognition Configuration**:
A complete, user-named ASR option backed by one Recognition Provider; it may be cloud-based or local, and multiple configurations may use the same provider.
_Avoid_: Recognition provider, Recognition Profile

**Language Model Configuration**:
A user-named, selectable set of settings for one compatible language-model endpoint; multiple configurations may use the same provider.
_Avoid_: LLM provider, API option

**Active Language Model Configuration**:
The optional, globally selected Language Model Configuration used across the application; when none is selected, LLM processing is unavailable.
_Avoid_: Profile model, application model

**Prompt Preset**:
A named Prompt used only for LLM processing; it may be supplied by Voxora or created by the user, and may be copied into an independent user-editable preset.
_Avoid_: LLM configuration, system rule

**Built-in Prompt Preset**:
A Prompt Preset supplied by Voxora that cannot be edited or deleted directly but can be copied.
_Avoid_: Default processing rule, locked user prompt

**Custom Prompt Preset**:
A user-editable Prompt Preset created directly or copied from another preset and optionally assigned a global shortcut.
_Avoid_: Built-in prompt, temporary prompt

**Active Prompt Preset**:
The Prompt Preset persistently selected as the global default, including when selected through its shortcut; Voxora always has an Active Prompt Preset.
_Avoid_: Temporary prompt, session prompt

**Effective Prompt**:
The LLM instruction assembled for a Dictation Session from its resolved Prompt Preset and Voxora-managed context such as the enabled Hotwords.
_Avoid_: Prompt preset, raw prompt

**Processing Pipeline**:
The globally ordered sequence of Built-in Processing Rules and at most one optional LLM-processing step applied to a Raw Transcript.
_Avoid_: Rule chain, LLM processing

**Built-in Processing Rule**:
A project-maintained text transformation that users may configure and arrange but cannot author as executable code.
_Avoid_: Custom rule, user regex, user script, plugin script

**Remove Trailing Sentence Punctuation**:
A Built-in Processing Rule that removes sentence-ending periods, question marks, exclamation marks, or ellipses while preserving closing quotation marks.
_Avoid_: Remove all punctuation, trim text

**Replace Conversational Punctuation With Spaces**:
A Built-in Processing Rule that replaces common sentence, pause, question, and exclamation punctuation with normalized spaces while preserving punctuation embedded in decimals, domains, abbreviations, and technical tokens.
_Avoid_: Remove all punctuation, strip symbols

**Reasoning Mode**:
A Language Model Configuration preference that requests provider-default, disabled, or enabled model reasoning behavior.
_Avoid_: Thinking switch, chain-of-thought setting

**Recording Shortcut**:
A global keyboard gesture assigned to either Push-to-Talk or Toggle recording behavior.
_Avoid_: Hotkey mode, trigger

**Default Processing Rules**:
The globally selected and ordered Built-in Processing Rules used when no application-specific override changes them.
_Avoid_: Processing Profile, default prompt

**Processing Overrides**:
Application-specific settings that inherit, force-enable, or force-disable Built-in Processing Rules and may select a Prompt Preset without changing the global processing order or Language Model Configuration.
_Avoid_: Processing Profile, application pipeline

**Application Profile**:
A rule that associates an application identity with Processing Overrides and an optional Prompt Preset selection.
_Avoid_: App configuration, window rule

## Vocabulary Assistance

**Hotword Library**:
A single user-managed collection of Hotwords organized into groups and supplied to compatible recognition or language-model processing capabilities.
_Avoid_: ASR hotwords, Prompt glossary

**Hotword Group**:
A named, globally enabled or disabled organizational group within the Hotword Library.
_Avoid_: Hotword library, recognition profile

**Hotword**:
A term or phrase in the Hotword Library, without provider-specific weights, pronunciations, or aliases.
_Avoid_: Keyword rule, vocabulary alias

**Hotword Candidate**:
A term suggested from the user's transcription history that does not affect processing until the user accepts it into a Hotword Library.
_Avoid_: Automatic hotword, learned vocabulary
