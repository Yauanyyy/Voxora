CREATE TABLE IF NOT EXISTS scalar_settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS llm_configurations (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    credential_ref INTEGER NOT NULL,
    model TEXT NOT NULL,
    timeout_ms INTEGER NOT NULL,
    reasoning TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS recognition_configurations (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    provider_code TEXT NOT NULL,
    base_url TEXT,
    credential_ref INTEGER,
    model TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS retention_policy (
    id INTEGER PRIMARY KEY NOT NULL CHECK (id = 1),
    text_enabled INTEGER NOT NULL,
    audio_enabled INTEGER NOT NULL,
    text_days INTEGER NOT NULL,
    audio_days INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS prompt_presets (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    content TEXT NOT NULL,
    built_in TEXT,
    shortcut TEXT UNIQUE
);

CREATE TABLE IF NOT EXISTS processing_rule_defaults (
    rule_code TEXT PRIMARY KEY NOT NULL,
    enabled INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS processing_steps (
    position INTEGER PRIMARY KEY NOT NULL,
    step_code TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS hotword_groups (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    enabled INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS hotwords (
    id INTEGER PRIMARY KEY NOT NULL,
    group_id INTEGER NOT NULL REFERENCES hotword_groups(id) ON DELETE CASCADE,
    text TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS application_profiles (
    id INTEGER PRIMARY KEY NOT NULL,
    identity TEXT NOT NULL,
    enabled INTEGER NOT NULL,
    prompt_id INTEGER REFERENCES prompt_presets(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS application_profile_rule_overrides (
    profile_id INTEGER NOT NULL REFERENCES application_profiles(id) ON DELETE CASCADE,
    rule_code TEXT NOT NULL,
    override_code TEXT NOT NULL,
    PRIMARY KEY(profile_id, rule_code)
);

CREATE TABLE IF NOT EXISTS dictation_records (
    id INTEGER PRIMARY KEY NOT NULL,
    session_id INTEGER NOT NULL,
    outcome TEXT,
    raw_text TEXT,
    processed_text TEXT,
    final_text TEXT,
    partial_text TEXT,
    audio_ref TEXT,
    audio_durable INTEGER NOT NULL DEFAULT 0,
    durable INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    failure_stage TEXT,
    failure_code TEXT,
    failure_retry TEXT,
    failure_certainty TEXT,
    hotwords_used INTEGER NOT NULL DEFAULT 0,
    hotwords_total INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS dictation_warnings (
    record_id INTEGER NOT NULL REFERENCES dictation_records(id) ON DELETE CASCADE,
    warning_code TEXT NOT NULL,
    PRIMARY KEY(record_id, warning_code)
);

CREATE TABLE IF NOT EXISTS audio_artifacts (
    audio_ref TEXT PRIMARY KEY NOT NULL,
    artifact_name TEXT NOT NULL UNIQUE,
    nonempty INTEGER NOT NULL CHECK (nonempty = 1)
);

CREATE TABLE IF NOT EXISTS recognition_attempts (
    record_id INTEGER NOT NULL REFERENCES dictation_records(id) ON DELETE CASCADE,
    id INTEGER NOT NULL,
    revision INTEGER NOT NULL,
    configuration_id INTEGER NOT NULL,
    status TEXT NOT NULL,
    raw_text TEXT,
    partial_text TEXT,
    failure_stage TEXT,
    failure_code TEXT,
    failure_retry TEXT,
    failure_certainty TEXT,
    PRIMARY KEY(record_id, id)
);

CREATE TABLE IF NOT EXISTS artifact_deletion_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    artifact_name TEXT NOT NULL UNIQUE,
    attempts INTEGER NOT NULL DEFAULT 0
);
