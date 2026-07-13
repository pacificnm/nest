//! Example async notes repository demonstrating nest-data contracts.

use async_trait::async_trait;
use nest_data::{AsyncRepository, DataError, DataResult, ListQuery};
use sqlx::Row;

use crate::connection::PostgresConnection;
use crate::error::sqlx_result;

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

/// PostgreSQL-backed notes repository (example).
pub struct NotesRepository {
    db: PostgresConnection,
}

impl NotesRepository {
    /// Creates a repository over the given connection.
    pub fn new(db: PostgresConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AsyncRepository<Note, NoteId> for NotesRepository {
    async fn get(&self, id: NoteId) -> DataResult<Option<Note>> {
        let row = sqlx_result(
            sqlx::query("SELECT id, title, body FROM notes WHERE id = $1")
                .bind(id.0)
                .fetch_optional(self.db.pool())
                .await,
        )?;
        Ok(row.map(|row| Note {
            id: NoteId(row.get("id")),
            title: row.get("title"),
            body: row.get("body"),
        }))
    }

    async fn list(&self, query: ListQuery) -> DataResult<Vec<Note>> {
        let rows = match (query.limit, query.offset) {
            (Some(limit), Some(offset)) => sqlx_result(
                sqlx::query("SELECT id, title, body FROM notes LIMIT $1 OFFSET $2")
                    .bind(limit as i64)
                    .bind(offset as i64)
                    .fetch_all(self.db.pool())
                    .await,
            )?,
            (Some(limit), None) => sqlx_result(
                sqlx::query("SELECT id, title, body FROM notes LIMIT $1")
                    .bind(limit as i64)
                    .fetch_all(self.db.pool())
                    .await,
            )?,
            (None, Some(offset)) => sqlx_result(
                sqlx::query("SELECT id, title, body FROM notes OFFSET $1")
                    .bind(offset as i64)
                    .fetch_all(self.db.pool())
                    .await,
            )?,
            (None, None) => sqlx_result(
                sqlx::query("SELECT id, title, body FROM notes")
                    .fetch_all(self.db.pool())
                    .await,
            )?,
        };
        Ok(rows
            .into_iter()
            .map(|row| Note {
                id: NoteId(row.get("id")),
                title: row.get("title"),
                body: row.get("body"),
            })
            .collect())
    }

    async fn insert(&self, entity: Note) -> DataResult<Note> {
        let row = sqlx_result(
            sqlx::query("INSERT INTO notes (title, body) VALUES ($1, $2) RETURNING id")
                .bind(&entity.title)
                .bind(&entity.body)
                .fetch_one(self.db.pool())
                .await,
        )?;
        Ok(Note {
            id: NoteId(row.get("id")),
            title: entity.title,
            body: entity.body,
        })
    }

    async fn update(&self, entity: Note) -> DataResult<Note> {
        let result = sqlx_result(
            sqlx::query("UPDATE notes SET title = $1, body = $2 WHERE id = $3")
                .bind(&entity.title)
                .bind(&entity.body)
                .bind(entity.id.0)
                .execute(self.db.pool())
                .await,
        )?;
        if result.rows_affected() == 0 {
            return Err(DataError::not_found(format!(
                "note not found: {}",
                entity.id.0
            )));
        }
        Ok(entity)
    }

    async fn delete(&self, id: NoteId) -> DataResult<()> {
        let result = sqlx_result(
            sqlx::query("DELETE FROM notes WHERE id = $1")
                .bind(id.0)
                .execute(self.db.pool())
                .await,
        )?;
        if result.rows_affected() == 0 {
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
            id BIGSERIAL PRIMARY KEY,
            title TEXT NOT NULL,
            body TEXT NOT NULL
        );",
        "DROP TABLE notes;",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PostgresConfig;
    use crate::connection::PostgresConnection;
    use crate::migration::PostgresMigrationRunner;
    use nest_data::{Migration, MigrationRunner};

    async fn setup() -> NotesRepository {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
        let conn = PostgresConnection::connect(&PostgresConfig::new(url))
            .await
            .unwrap();
        let migrations: Vec<Box<dyn Migration>> = vec![Box::new(notes_migration())];
        PostgresMigrationRunner::new(conn.clone(), migrations)
            .apply_all()
            .unwrap();
        NotesRepository::new(conn)
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL and PostgreSQL"]
    async fn crud_round_trip() {
        let repo = setup().await;
        let created = repo
            .insert(Note {
                id: NoteId(0),
                title: "Hello".into(),
                body: "World".into(),
            })
            .await
            .unwrap();
        assert!(created.id.0 > 0);

        let fetched = repo.get(created.id.clone()).await.unwrap().unwrap();
        assert_eq!(fetched.title, "Hello");

        let updated = repo
            .update(Note {
                id: created.id.clone(),
                title: "Updated".into(),
                body: "Body".into(),
            })
            .await
            .unwrap();
        assert_eq!(updated.title, "Updated");

        assert_eq!(repo.list(ListQuery::new()).await.unwrap().len(), 1);
        repo.delete(created.id.clone()).await.unwrap();
        assert!(repo.get(created.id).await.unwrap().is_none());
    }
}
