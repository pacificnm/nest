//! Example notes repository demonstrating the nest-data contracts.

use nest_data::{DataError, DataResult, ListQuery, Repository};

use crate::connection::SqliteConnection;
use crate::error::sqlite_result;

/// Note identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NoteId(pub i64);

/// Example note entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    /// Primary key.
    pub id: NoteId,
    /// Note title.
    pub title: String,
    /// Note body.
    pub body: String,
}

/// SQLite-backed notes repository (example).
pub struct NotesRepository {
    db: SqliteConnection,
}

impl NotesRepository {
    /// Creates a repository over the given connection.
    pub fn new(db: SqliteConnection) -> Self {
        Self { db }
    }
}

impl Repository<Note, NoteId> for NotesRepository {
    fn get(&self, id: NoteId) -> DataResult<Option<Note>> {
        self.db.with_connection(|conn| {
            let mut stmt = sqlite_result(conn.prepare(
                "SELECT id, title, body FROM notes WHERE id = ?1",
            ))?;
            let mut rows = sqlite_result(stmt.query([id.0]))?;
            if let Some(row) = sqlite_result(rows.next())? {
                Ok(Some(Note {
                    id: NoteId(sqlite_result(row.get(0))?),
                    title: sqlite_result(row.get(1))?,
                    body: sqlite_result(row.get(2))?,
                }))
            } else {
                Ok(None)
            }
        })
    }

    fn list(&self, query: ListQuery) -> DataResult<Vec<Note>> {
        let sql = match (query.limit, query.offset) {
            (Some(limit), Some(offset)) => {
                format!("SELECT id, title, body FROM notes LIMIT {limit} OFFSET {offset}")
            }
            (Some(limit), None) => format!("SELECT id, title, body FROM notes LIMIT {limit}"),
            (None, Some(offset)) => format!("SELECT id, title, body FROM notes OFFSET {offset}"),
            (None, None) => "SELECT id, title, body FROM notes".to_string(),
        };

        self.db.with_connection(|conn| {
            let mut stmt = sqlite_result(conn.prepare(&sql))?;
            let rows = sqlite_result(stmt.query_map([], |row| {
                Ok(Note {
                    id: NoteId(row.get(0)?),
                    title: row.get(1)?,
                    body: row.get(2)?,
                })
            }))?;
            let mut notes = Vec::new();
            for row in rows {
                notes.push(sqlite_result(row)?);
            }
            Ok(notes)
        })
    }

    fn insert(&self, entity: Note) -> DataResult<Note> {
        self.db.with_connection(|conn| {
            sqlite_result(conn.execute(
                "INSERT INTO notes (title, body) VALUES (?1, ?2)",
                [&entity.title, &entity.body],
            ))?;
            let id = conn.last_insert_rowid();
            Ok(Note {
                id: NoteId(id),
                title: entity.title,
                body: entity.body,
            })
        })
    }

    fn update(&self, entity: Note) -> DataResult<Note> {
        let rows = self.db.with_connection(|conn| {
            sqlite_result(conn.execute(
                "UPDATE notes SET title = ?1, body = ?2 WHERE id = ?3",
                [&entity.title, &entity.body, &entity.id.0.to_string()],
            ))
        })?;
        if rows == 0 {
            return Err(DataError::not_found(format!("note not found: {}", entity.id.0)));
        }
        Ok(entity)
    }

    fn delete(&self, id: NoteId) -> DataResult<()> {
        let rows = self.db.with_connection(|conn| {
            sqlite_result(conn.execute("DELETE FROM notes WHERE id = ?1", [id.0]))
        })?;
        if rows == 0 {
            return Err(DataError::not_found(format!("note not found: {}", id.0)));
        }
        Ok(())
    }
}

/// SQL migration that creates the notes table.
pub fn notes_migration() -> nest_data::SqlMigration {
    nest_data::SqlMigration::new(
        "001_create_notes",
        "CREATE TABLE notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            body TEXT NOT NULL
        );",
        "DROP TABLE notes;",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SqliteConfig;
    use crate::connection::SqliteConnection;
    use crate::migration::SqliteMigrationRunner;
    use nest_data::{Migration, MigrationRunner};

    fn setup() -> NotesRepository {
        let conn = SqliteConnection::open(&SqliteConfig::memory()).unwrap();
        let migrations: Vec<Box<dyn Migration>> = vec![Box::new(notes_migration())];
        SqliteMigrationRunner::new(conn.clone(), migrations)
            .apply_all()
            .unwrap();
        NotesRepository::new(conn)
    }

    #[test]
    fn crud_round_trip() {
        let repo = setup();
        let created = repo
            .insert(Note {
                id: NoteId(0),
                title: "Hello".into(),
                body: "World".into(),
            })
            .unwrap();
        assert!(created.id.0 > 0);

        let fetched = repo.get(created.id.clone()).unwrap().unwrap();
        assert_eq!(fetched.title, "Hello");

        let updated = repo
            .update(Note {
                id: created.id.clone(),
                title: "Updated".into(),
                body: "Body".into(),
            })
            .unwrap();
        assert_eq!(updated.title, "Updated");

        assert_eq!(repo.list(ListQuery::new()).unwrap().len(), 1);
        repo.delete(created.id.clone()).unwrap();
        assert!(repo.get(created.id).unwrap().is_none());
    }
}
