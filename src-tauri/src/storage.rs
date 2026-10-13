use std::{path::Path, time::Duration};

use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::Serialize;

const LEARNER_MIGRATION_1: &str = include_str!("../migrations/learner/0001.sql");
const LEARNER_MIGRATION_2: &str = include_str!("../migrations/learner/0002.sql");
pub const MAX_DRAFT_BYTES: usize = 1_048_576;

#[derive(Debug, Serialize)]
pub struct Draft {
    pub id: String,
    pub title: String,
    pub source: String,
    pub language: String,
    pub updated_at: String,
}

pub fn open(path: &Path) -> Result<Connection, String> {
    let connection = Connection::open(path)
        .map_err(|error| format!("Could not open {}: {error}", path.display()))?;
    connection
        .busy_timeout(Duration::from_secs(3))
        .map_err(|error| format!("Could not configure {}: {error}", path.display()))?;
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .map_err(|error| format!("Could not enable database constraints: {error}"))?;
    connection
        .pragma_update(None, "journal_mode", "WAL")
        .map_err(|error| format!("Could not enable reliable database writes: {error}"))?;
    connection
        .pragma_update(None, "synchronous", "NORMAL")
        .map_err(|error| format!("Could not configure database durability: {error}"))?;
    Ok(connection)
}

pub fn migrate_learner(connection: &mut Connection) -> Result<(), String> {
    let version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|error| format!("Could not read learner database version: {error}"))?;
    if version > 3 {
        return Err(format!(
            "Learner database schema {version} is newer than this app supports."
        ));
    }
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| format!("Could not begin learner database migration: {error}"))?;
    if version == 0 {
        transaction
            .execute_batch(LEARNER_MIGRATION_1)
            .map_err(|error| format!("Could not create learner draft storage: {error}"))?;
        transaction
            .pragma_update(None, "user_version", 1)
            .map_err(|error| format!("Could not record learner database version: {error}"))?;
    }
    if version < 2 {
        transaction
            .execute_batch(LEARNER_MIGRATION_2)
            .map_err(|error| format!("Could not upgrade learner draft storage: {error}"))?;
        transaction
            .pragma_update(None, "user_version", 2)
            .map_err(|error| format!("Could not record learner database version: {error}"))?;
    }
    if version < 3 {
        transaction
            .execute_batch(include_str!("../migrations/learner/0003.sql"))
            .map_err(|e| e.to_string())?;
        transaction
            .pragma_update(None, "user_version", 3)
            .map_err(|e| e.to_string())?;
    }
    transaction
        .commit()
        .map_err(|error| format!("Could not commit learner database migration: {error}"))
}

pub fn get_draft(connection: &Connection) -> Result<Option<Draft>, String> {
    connection
        .query_row(
            "SELECT id, title, source, language, updated_at FROM drafts WHERE id = 'open-lab-default'",
            [],
            |row| {
                Ok(Draft {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    source: row.get(2)?,
                    language: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            },
        )
        .optional()
        .map_err(|error| format!("Could not restore the open-lab draft: {error}"))
}

pub fn save_draft(connection: &Connection, source: &str, language: &str) -> Result<(), String> {
    validate_draft(source, language)?;
    connection
        .execute(
            "INSERT INTO drafts (id, title, source, language, updated_at)
             VALUES ('open-lab-default', 'Scratch project', ?1, ?2, CURRENT_TIMESTAMP)
             ON CONFLICT(id) DO UPDATE SET
               source = excluded.source,
               language = excluded.language,
               updated_at = CURRENT_TIMESTAMP",
            params![source, language],
        )
        .map_err(|error| format!("Could not save the open-lab draft: {error}"))?;
    Ok(())
}

pub fn import_browser_draft(connection: &Connection, source: &str) -> Result<bool, String> {
    validate_draft(source, "C17")?;
    let changed = connection
        .execute(
            "INSERT OR IGNORE INTO drafts (id, title, source, language, updated_at)
             VALUES ('open-lab-default', 'Scratch project', ?1, 'C17', CURRENT_TIMESTAMP)",
            [source],
        )
        .map_err(|error| format!("Could not migrate the existing browser draft: {error}"))?;
    Ok(changed == 1)
}

fn validate_draft(source: &str, language: &str) -> Result<(), String> {
    if source.len() > MAX_DRAFT_BYTES {
        return Err(format!(
            "Draft exceeds the {} byte local storage limit.",
            MAX_DRAFT_BYTES
        ));
    }
    if !matches!(language, "C17" | "C23") {
        return Err("The open-lab draft language must be C17 or C23.".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_upgrade_keeps_existing_draft_bytes() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE drafts (id TEXT PRIMARY KEY, title TEXT NOT NULL, source TEXT NOT NULL);
                 PRAGMA user_version = 1;",
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO drafts (id, title, source) VALUES (?1, ?2, ?3)",
                params![
                    "open-lab-default",
                    "Scratch project",
                    "#include <stdio.h>\nint main(void) { return 0; }"
                ],
            )
            .unwrap();
        migrate_learner(&mut connection).unwrap();
        let draft = get_draft(&connection).unwrap().unwrap();
        assert_eq!(
            draft.source,
            "#include <stdio.h>\nint main(void) { return 0; }"
        );
        assert_eq!(draft.language, "C17");
        let version: u32 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 3);
    }

    #[test]
    fn browser_import_does_not_replace_a_native_draft() {
        let mut connection = Connection::open_in_memory().unwrap();
        migrate_learner(&mut connection).unwrap();
        save_draft(&connection, "int native = 1;", "C23").unwrap();
        assert!(!import_browser_draft(&connection, "int browser = 2;").unwrap());
        assert_eq!(
            get_draft(&connection).unwrap().unwrap().source,
            "int native = 1;"
        );
    }
}
