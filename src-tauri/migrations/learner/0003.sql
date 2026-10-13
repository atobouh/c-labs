CREATE TABLE learning_state (id TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
CREATE TABLE attempts (id TEXT PRIMARY KEY, package_id TEXT NOT NULL, submission_hash TEXT NOT NULL, content_hash TEXT NOT NULL, rubric_version INTEGER NOT NULL, result_json TEXT NOT NULL, files_json TEXT NOT NULL, created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
CREATE TABLE exercise_attempts (id INTEGER PRIMARY KEY, record_id TEXT NOT NULL, answer_json TEXT NOT NULL, correct INTEGER NOT NULL, category TEXT NOT NULL, hint_level INTEGER NOT NULL, created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
CREATE TABLE chapter_gates (chapter_id TEXT PRIMARY KEY, attempt_id TEXT NOT NULL REFERENCES attempts(id), content_hash TEXT NOT NULL);
