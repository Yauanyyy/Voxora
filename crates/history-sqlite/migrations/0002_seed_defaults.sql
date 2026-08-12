INSERT OR IGNORE INTO retention_policy(id, text_enabled, audio_enabled, text_days, audio_days)
VALUES (1, 1, 1, 30, 30);

INSERT OR IGNORE INTO scalar_settings(key, value) VALUES ('active_prompt', '1');
INSERT OR IGNORE INTO scalar_settings(key, value) VALUES ('active_llm', '');
INSERT OR IGNORE INTO scalar_settings(key, value) VALUES ('active_recognition', '');

INSERT OR IGNORE INTO prompt_presets(id, name, content, built_in, shortcut) VALUES
    (1, 'Original Text Cleanup', 'Clean up the dictated text while preserving its original meaning, facts, tone, language, and level of detail. Correct obvious recognition, grammar, and punctuation issues. Return only the revised text.', 'original_text_cleanup', NULL),
    (2, 'Concise Expression', 'Rewrite the dictated text concisely while preserving its meaning and essential details. Remove repetition and filler. Return only the revised text.', 'concise_expression', NULL),
    (3, 'Formal Expression', 'Rewrite the dictated text in a clear, formal style while preserving its meaning and facts. Return only the revised text.', 'formal_expression', NULL);

INSERT OR IGNORE INTO processing_rule_defaults(rule_code, enabled) VALUES
    ('remove_trailing_sentence_punctuation', 0),
    ('replace_conversational_punctuation_with_spaces', 0);

INSERT OR IGNORE INTO processing_steps(position, step_code) VALUES
    (0, 'remove_trailing_sentence_punctuation'),
    (1, 'replace_conversational_punctuation_with_spaces');
