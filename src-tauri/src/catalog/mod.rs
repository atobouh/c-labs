mod importer;

use std::path::Path;

use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use serde::Serialize;
use serde_json::Value;

use self::importer::{ImportedCatalog, ImportedItem, SourceRecord};

const CATALOG_MIGRATION_1: &str = include_str!("../../migrations/catalog/0001.sql");

type CatalogItemColumns = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
);
type LexiconColumns = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
);

#[derive(Debug, Serialize)]
pub struct CatalogOverview {
    pub revision: u32,
    pub course_count: usize,
    pub chapter_count: usize,
    pub record_count: usize,
    pub lexicon_count: usize,
    pub courses: Vec<CourseSummary>,
}

#[derive(Debug, Serialize)]
pub struct CourseSummary {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub chapter_count: usize,
    pub pilot_chapter_count: usize,
    pub chapters: Vec<ChapterSummary>,
}

#[derive(Debug, Serialize)]
pub struct ChapterSummary {
    pub id: String,
    pub order: u32,
    pub title: String,
    pub summary: String,
    pub project_title: String,
    pub outline_status: String,
    pub content_status: String,
    pub lesson_count: usize,
    pub exercise_count: usize,
    pub animation_count: usize,
    pub lab_count: usize,
    pub project_count: usize,
}

#[derive(Debug, Serialize)]
pub struct SearchHit {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub chapter_id: Option<String>,
    pub excerpt: String,
}

#[derive(Debug, Serialize)]
pub struct CatalogItemDetail {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub chapter_id: String,
    pub status: String,
    pub artifact: String,
    pub body_text: String,
    pub payload: Value,
    pub citations: Vec<Citation>,
}

#[derive(Debug, Serialize)]
pub struct LexiconEntry {
    pub id: String,
    pub term: String,
    pub category: String,
    pub definition: String,
    pub example: String,
    pub misconception: String,
    pub related_ids: Vec<String>,
    pub citations: Vec<Citation>,
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct Citation {
    pub source_id: String,
    pub source_title: String,
    pub source_url: String,
    pub locator: String,
    pub claim: String,
}

pub fn sync(connection: &mut Connection, resource_root: &Path) -> Result<(), String> {
    let imported = importer::load(resource_root)?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error("Could not begin catalog migration"))?;
    migrate(&transaction)?;

    let current: Option<(u32, String)> = transaction
        .query_row(
            "SELECT content_revision, package_sha256 FROM catalog_meta WHERE singleton = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(db_error("Could not read catalog version"))?;
    if current.as_ref().is_some_and(|(revision, hash)| {
        *revision == imported.revision && hash == &imported.package_sha256
    }) {
        transaction
            .commit()
            .map_err(db_error("Could not finish catalog check"))?;
        return Ok(());
    }

    write_catalog(&transaction, &imported)?;
    transaction
        .commit()
        .map_err(db_error("Could not commit catalog update"))
}

fn migrate(transaction: &Transaction<'_>) -> Result<(), String> {
    let version: u32 = transaction
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(db_error("Could not read catalog schema version"))?;
    if version > 1 {
        return Err(format!(
            "Catalog database schema {version} is newer than this app supports."
        ));
    }
    if version == 0 {
        transaction
            .execute_batch(CATALOG_MIGRATION_1)
            .map_err(db_error("Could not create the catalog schema"))?;
        transaction
            .pragma_update(None, "user_version", 1)
            .map_err(db_error("Could not record the catalog schema version"))?;
    }
    Ok(())
}

fn write_catalog(transaction: &Transaction<'_>, imported: &ImportedCatalog) -> Result<(), String> {
    transaction
        .execute_batch(
            "DELETE FROM search_documents;
             DELETE FROM lexicon_entries;
             DELETE FROM catalog_items;
             DELETE FROM chapters;
             DELETE FROM courses;
             DELETE FROM sources;
             DELETE FROM catalog_meta;",
        )
        .map_err(db_error("Could not prepare the local catalog update"))?;

    for source in &imported.sources {
        insert_source(transaction, source)?;
    }
    for course in &imported.courses {
        transaction
            .execute(
                "INSERT INTO courses (id, title, summary, position) VALUES (?1, ?2, ?3, ?4)",
                params![course.id, course.title, course.summary, course.position],
            )
            .map_err(db_error("Could not import a course"))?;
        insert_search_document(
            transaction,
            &course.id,
            "course",
            &course.title,
            None,
            &course.summary,
            &format!("{} {}", course.title, course.summary),
        )?;
        for chapter in &course.chapters {
            transaction
                .execute(
                    "INSERT INTO chapters
                     (id, course_id, position, title, summary, project_title, outline_status, content_status)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        chapter.id,
                        course.id,
                        chapter.position,
                        chapter.title,
                        chapter.summary,
                        chapter.project_title,
                        chapter.outline_status,
                        chapter.content_status,
                    ],
                )
                .map_err(db_error("Could not import a course chapter"))?;
            insert_search_document(
                transaction,
                &chapter.id,
                "chapter",
                &chapter.title,
                Some(&chapter.id),
                &chapter.summary,
                &format!(
                    "{} {} {}",
                    chapter.title, chapter.summary, chapter.project_title
                ),
            )?;
        }
    }

    for item in &imported.items {
        insert_item(transaction, item)?;
    }
    for entry in &imported.lexicon {
        transaction
            .execute(
                "INSERT INTO lexicon_entries
                 (id, term, category, definition, example, misconception, related_ids_json, citations_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    entry.id,
                    entry.term,
                    entry.category,
                    entry.definition,
                    entry.example,
                    entry.misconception,
                    entry.related_ids_json,
                    entry.citations_json,
                ],
            )
            .map_err(db_error("Could not import a lexicon entry"))?;
        insert_search_document(
            transaction,
            &entry.id,
            "lexicon",
            &entry.term,
            Some("foundations-arrays-memory"),
            &entry.definition,
            &entry.search_text,
        )?;
    }
    transaction
        .execute(
            "INSERT INTO catalog_meta (singleton, content_revision, package_sha256) VALUES (1, ?1, ?2)",
            params![imported.revision, imported.package_sha256],
        )
        .map_err(db_error("Could not mark the imported catalog revision"))?;
    Ok(())
}

fn insert_source(transaction: &Transaction<'_>, source: &SourceRecord) -> Result<(), String> {
    transaction
        .execute(
            "INSERT INTO sources (id, title, author, version, url, redistribution, status, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                source.id,
                source.title,
                source.author,
                source.version,
                source.url,
                source.redistribution,
                source.status,
                source.notes,
            ],
        )
        .map_err(db_error("Could not import a source record"))?;
    Ok(())
}

fn insert_item(transaction: &Transaction<'_>, item: &ImportedItem) -> Result<(), String> {
    transaction
        .execute(
            "INSERT INTO catalog_items
             (id, chapter_id, kind, title, artifact, status, body_text, payload_json, citations_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                item.id,
                item.chapter_id,
                item.kind,
                item.title,
                item.artifact,
                item.status,
                item.body_text,
                item.payload_json,
                item.citations_json,
            ],
        )
        .map_err(db_error("Could not import a learning record"))?;
    insert_search_document(
        transaction,
        &item.id,
        &item.kind,
        &item.title,
        Some(&item.chapter_id),
        &item.body_text,
        &item.search_text,
    )
}

fn insert_search_document(
    transaction: &Transaction<'_>,
    id: &str,
    kind: &str,
    title: &str,
    chapter_id: Option<&str>,
    excerpt: &str,
    search_text: &str,
) -> Result<(), String> {
    transaction
        .execute(
            "INSERT INTO search_documents (id, kind, title, chapter_id, excerpt, search_text)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, kind, title, chapter_id, excerpt, search_text],
        )
        .map_err(db_error("Could not index a local catalog record"))?;
    Ok(())
}

pub fn overview(connection: &Connection) -> Result<CatalogOverview, String> {
    let revision: u32 = connection
        .query_row(
            "SELECT content_revision FROM catalog_meta WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .map_err(db_error("The local catalog has not been imported"))?;
    let mut statement = connection
        .prepare("SELECT id, title, summary FROM courses ORDER BY position")
        .map_err(db_error("Could not read the course catalog"))?;
    let raw_courses: Vec<(String, String, String)> = statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(db_error("Could not read course rows"))?
        .collect::<rusqlite::Result<_>>()
        .map_err(db_error("Could not decode course rows"))?;
    drop(statement);

    let mut courses = Vec::with_capacity(raw_courses.len());
    let mut chapter_count = 0usize;
    for (id, title, summary) in raw_courses {
        let mut statement = connection
            .prepare(
                "SELECT c.id, c.position, c.title, c.summary, c.project_title,
                        c.outline_status, c.content_status,
                        SUM(CASE WHEN i.kind = 'lesson' THEN 1 ELSE 0 END),
                        SUM(CASE WHEN i.kind = 'exercise' THEN 1 ELSE 0 END),
                        SUM(CASE WHEN i.kind = 'animation' THEN 1 ELSE 0 END),
                        SUM(CASE WHEN i.kind = 'lab' THEN 1 ELSE 0 END),
                        SUM(CASE WHEN i.kind = 'project' THEN 1 ELSE 0 END)
                 FROM chapters c
                 LEFT JOIN catalog_items i ON i.chapter_id = c.id
                 WHERE c.course_id = ?1
                 GROUP BY c.id
                 ORDER BY c.position",
            )
            .map_err(db_error("Could not read chapter catalog"))?;
        let chapters: Vec<ChapterSummary> = statement
            .query_map([&id], |row| {
                Ok(ChapterSummary {
                    id: row.get(0)?,
                    order: row.get(1)?,
                    title: row.get(2)?,
                    summary: row.get(3)?,
                    project_title: row.get(4)?,
                    outline_status: row.get(5)?,
                    content_status: row.get(6)?,
                    lesson_count: row.get::<_, Option<u32>>(7)?.unwrap_or_default() as usize,
                    exercise_count: row.get::<_, Option<u32>>(8)?.unwrap_or_default() as usize,
                    animation_count: row.get::<_, Option<u32>>(9)?.unwrap_or_default() as usize,
                    lab_count: row.get::<_, Option<u32>>(10)?.unwrap_or_default() as usize,
                    project_count: row.get::<_, Option<u32>>(11)?.unwrap_or_default() as usize,
                })
            })
            .map_err(db_error("Could not read chapter rows"))?
            .collect::<rusqlite::Result<_>>()
            .map_err(db_error("Could not decode chapter rows"))?;
        chapter_count += chapters.len();
        let pilot_chapter_count = chapters
            .iter()
            .filter(|chapter| chapter.content_status == "pilot-catalog")
            .count();
        courses.push(CourseSummary {
            id,
            title,
            summary,
            chapter_count: chapters.len(),
            pilot_chapter_count,
            chapters,
        });
    }
    let record_count = count(connection, "catalog_items")?;
    let lexicon_count = count(connection, "lexicon_entries")?;
    Ok(CatalogOverview {
        revision,
        course_count: courses.len(),
        chapter_count,
        record_count,
        lexicon_count,
        courses,
    })
}

pub fn search(connection: &Connection, query: &str) -> Result<Vec<SearchHit>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    if query.chars().count() > 120 {
        return Err("Search text is limited to 120 characters.".into());
    }
    let escaped = query
        .replace('!', "!!")
        .replace('%', "!%")
        .replace('_', "!_");
    let pattern = format!("%{escaped}%");
    let mut statement = connection
        .prepare(
            "SELECT id, kind, title, chapter_id, excerpt
             FROM search_documents
             WHERE lower(search_text) LIKE lower(?1) ESCAPE '!'
             ORDER BY CASE kind WHEN 'course' THEN 0 WHEN 'chapter' THEN 1 ELSE 2 END, title COLLATE NOCASE
             LIMIT 24",
        )
        .map_err(db_error("Could not prepare local catalog search"))?;
    statement
        .query_map([pattern], |row| {
            Ok(SearchHit {
                id: row.get(0)?,
                kind: row.get(1)?,
                title: row.get(2)?,
                chapter_id: row.get(3)?,
                excerpt: row.get(4)?,
            })
        })
        .map_err(db_error("Could not search local course data"))?
        .collect::<rusqlite::Result<_>>()
        .map_err(db_error("Could not decode search results"))
}

pub fn item(connection: &Connection, id: &str) -> Result<Option<CatalogItemDetail>, String> {
    let row: Option<CatalogItemColumns> = connection
        .query_row(
            "SELECT id, kind, title, chapter_id, status, artifact, body_text, payload_json, citations_json
             FROM catalog_items WHERE id = ?1",
            [id],
            |row| {
                Ok((
                    row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?,
                    row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?,
                ))
            },
        )
        .optional()
        .map_err(db_error("Could not read the requested course record"))?;
    let Some((
        id,
        kind,
        title,
        chapter_id,
        status,
        artifact,
        body_text,
        payload_json,
        citations_json,
    )) = row
    else {
        return Ok(None);
    };
    let payload = serde_json::from_str(&payload_json)
        .map_err(|error| format!("Stored record '{id}' is invalid: {error}"))?;
    let citations = serde_json::from_str(&citations_json)
        .map_err(|error| format!("Stored citations for '{id}' are invalid: {error}"))?;
    Ok(Some(CatalogItemDetail {
        id,
        kind,
        title,
        chapter_id,
        status,
        artifact,
        body_text,
        payload,
        citations,
    }))
}

pub fn lexicon(connection: &Connection) -> Result<Vec<LexiconEntry>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, term, category, definition, example, misconception,
                    related_ids_json, citations_json
             FROM lexicon_entries ORDER BY term COLLATE NOCASE",
        )
        .map_err(db_error("Could not read the local lexicon"))?;
    let raw: Vec<LexiconColumns> = statement
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
            ))
        })
        .map_err(db_error("Could not query lexicon entries"))?
        .collect::<rusqlite::Result<_>>()
        .map_err(db_error("Could not decode lexicon entries"))?;
    raw.into_iter()
        .map(
            |(id, term, category, definition, example, misconception, related, citations)| {
                Ok(LexiconEntry {
                    id,
                    term,
                    category,
                    definition,
                    example,
                    misconception,
                    related_ids: serde_json::from_str(&related)
                        .map_err(|error| error.to_string())?,
                    citations: serde_json::from_str(&citations)
                        .map_err(|error| error.to_string())?,
                })
            },
        )
        .collect()
}

fn count(connection: &Connection, table: &str) -> Result<usize, String> {
    let sql = match table {
        "catalog_items" => "SELECT COUNT(*) FROM catalog_items",
        "lexicon_entries" => "SELECT COUNT(*) FROM lexicon_entries",
        _ => return Err("Unknown catalog table.".into()),
    };
    connection
        .query_row(sql, [], |row| row.get::<_, u32>(0))
        .map(|count| count as usize)
        .map_err(db_error("Could not count local course data"))
}

fn db_error(context: &'static str) -> impl FnOnce(rusqlite::Error) -> String {
    move |error| format!("{context}: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_catalog_migrations_import_search_and_lookup_pilot_records() {
        let root = repository_root();
        let temp = temp_dir("catalog-happy-path");
        let mut connection = Connection::open(temp.join("catalog.sqlite")).unwrap();
        sync(&mut connection, &root).unwrap();

        let overview = overview(&connection).unwrap();
        assert_eq!(overview.course_count, 3);
        assert_eq!(overview.chapter_count, 24);
        assert_eq!(overview.record_count, 14);
        assert_eq!(overview.lexicon_count, 8);
        assert_eq!(
            overview.courses[0].chapters[3].content_status,
            "pilot-catalog"
        );
        assert_eq!(overview.courses[0].chapters[3].lesson_count, 4);
        assert_eq!(overview.courses[0].chapters[3].lab_count, 3);

        let results = search(&connection, "one-past").unwrap();
        assert!(results.iter().any(|hit| hit.id == "exercise-one-past"));
        assert!(results.iter().any(|hit| hit.kind == "lexicon"));
        assert!(search(&connection, "does-not-exist").unwrap().is_empty());

        let item = item(&connection, "lesson-valid-range").unwrap().unwrap();
        assert!(item.body_text.contains("one-past"));
        assert!(
            item.citations
                .iter()
                .any(|citation| citation.source_id == "wg14-c17")
        );
        assert_eq!(lexicon(&connection).unwrap().len(), 8);
        drop(connection);
        std::fs::remove_dir_all(temp).unwrap();
    }

    fn repository_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_owned()
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("machine-room-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }
}
