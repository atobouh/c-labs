CREATE TABLE catalog_meta (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    content_revision INTEGER NOT NULL,
    package_sha256 TEXT NOT NULL,
    imported_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE courses (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    position INTEGER NOT NULL UNIQUE CHECK (position > 0)
);

CREATE TABLE chapters (
    id TEXT PRIMARY KEY,
    course_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position > 0),
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    project_title TEXT NOT NULL,
    outline_status TEXT NOT NULL,
    content_status TEXT NOT NULL CHECK (content_status IN ('outline', 'pilot-catalog')),
    UNIQUE (course_id, position)
);

CREATE TABLE catalog_items (
    id TEXT PRIMARY KEY,
    chapter_id TEXT NOT NULL REFERENCES chapters(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    title TEXT NOT NULL,
    artifact TEXT NOT NULL,
    status TEXT NOT NULL,
    body_text TEXT NOT NULL,
    payload_json TEXT NOT NULL CHECK (json_valid(payload_json)),
    citations_json TEXT NOT NULL CHECK (json_valid(citations_json))
);

CREATE TABLE lexicon_entries (
    id TEXT PRIMARY KEY,
    term TEXT NOT NULL,
    category TEXT NOT NULL,
    definition TEXT NOT NULL,
    example TEXT NOT NULL,
    misconception TEXT NOT NULL,
    related_ids_json TEXT NOT NULL CHECK (json_valid(related_ids_json)),
    citations_json TEXT NOT NULL CHECK (json_valid(citations_json))
);

CREATE TABLE sources (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    version TEXT NOT NULL,
    url TEXT NOT NULL,
    redistribution TEXT NOT NULL,
    status TEXT NOT NULL,
    notes TEXT NOT NULL
);

CREATE TABLE search_documents (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    title TEXT NOT NULL,
    chapter_id TEXT,
    excerpt TEXT NOT NULL,
    search_text TEXT NOT NULL
);

CREATE INDEX search_documents_kind_idx ON search_documents(kind);
CREATE INDEX chapters_course_order_idx ON chapters(course_id, position);
CREATE INDEX catalog_items_chapter_kind_idx ON catalog_items(chapter_id, kind);
