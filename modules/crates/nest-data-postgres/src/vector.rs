//! pgvector similarity search helpers.

use nest_data::DataResult;
use pgvector::Vector;
use sqlx::AssertSqlSafe;
use sqlx::{PgPool, Row};

use crate::error::sqlx_result;

/// Default embedding dimension (OpenAI `text-embedding-3-small`).
pub const DEFAULT_EMBEDDING_DIM: usize = 1536;

/// A row returned from vector similarity search.
#[derive(Debug, Clone, PartialEq)]
pub struct SimilarityHit {
    /// Row identifier.
    pub id: String,
    /// Cosine distance (`<=>` operator); lower is more similar.
    pub distance: f32,
}

/// Configurable vector similarity search over a table.
pub struct VectorSearch {
    pool: PgPool,
    table: String,
    id_column: String,
    embedding_column: String,
    project_id_column: Option<String>,
}

impl VectorSearch {
    /// Creates a search helper for the given table and columns.
    pub fn new(
        pool: &PgPool,
        table: impl Into<String>,
        id_column: impl Into<String>,
        embedding_column: impl Into<String>,
    ) -> Self {
        Self {
            pool: pool.clone(),
            table: table.into(),
            id_column: id_column.into(),
            embedding_column: embedding_column.into(),
            project_id_column: None,
        }
    }

    /// Scopes similarity search to rows matching a project id column.
    pub fn with_project_scope(mut self, column: impl Into<String>) -> Self {
        self.project_id_column = Some(column.into());
        self
    }

    /// Finds the nearest rows by cosine distance to the query embedding.
    pub async fn search_similar(
        &self,
        embedding: &[f32],
        limit: u32,
        project_id: Option<&str>,
    ) -> DataResult<Vec<SimilarityHit>> {
        let query_vec = Vector::from(embedding.to_vec());
        let sql = if let Some(project_col) = &self.project_id_column {
            if project_id.is_some() {
                format!(
                    "SELECT {id}::text AS id, {emb} <=> $1 AS distance
                     FROM {table}
                     WHERE {project_col} = $2 AND {emb} IS NOT NULL
                     ORDER BY {emb} <=> $1
                     LIMIT $3",
                    id = self.id_column,
                    emb = self.embedding_column,
                    table = self.table,
                    project_col = project_col,
                )
            } else {
                format!(
                    "SELECT {id}::text AS id, {emb} <=> $1 AS distance
                     FROM {table}
                     WHERE {emb} IS NOT NULL
                     ORDER BY {emb} <=> $1
                     LIMIT $2",
                    id = self.id_column,
                    emb = self.embedding_column,
                    table = self.table,
                )
            }
        } else {
            format!(
                "SELECT {id}::text AS id, {emb} <=> $1 AS distance
                 FROM {table}
                 WHERE {emb} IS NOT NULL
                 ORDER BY {emb} <=> $1
                 LIMIT $2",
                id = self.id_column,
                emb = self.embedding_column,
                table = self.table,
            )
        };

        let rows = if let Some(_project_col) = &self.project_id_column {
            if let Some(project_id) = project_id {
                sqlx_result(
                    sqlx::query(AssertSqlSafe(sql.clone()))
                        .bind(query_vec)
                        .bind(project_id)
                        .bind(limit as i64)
                        .fetch_all(&self.pool)
                        .await,
                )?
            } else {
                sqlx_result(
                    sqlx::query(AssertSqlSafe(sql.clone()))
                        .bind(query_vec)
                        .bind(limit as i64)
                        .fetch_all(&self.pool)
                        .await,
                )?
            }
        } else {
            sqlx_result(
                sqlx::query(AssertSqlSafe(sql))
                    .bind(query_vec)
                    .bind(limit as i64)
                    .fetch_all(&self.pool)
                    .await,
            )?
        };

        Ok(rows
            .into_iter()
            .map(|row| SimilarityHit {
                id: row.get("id"),
                distance: row.get::<f64, _>("distance") as f32,
            })
            .collect())
    }
}

/// Migration enabling the pgvector extension.
pub fn enable_vector_migration() -> nest_data::SqlMigration {
    nest_data::SqlMigration::new(
        "000_enable_vector",
        "CREATE EXTENSION IF NOT EXISTS vector;",
        "DROP EXTENSION IF EXISTS vector;",
    )
}

/// Sample table migration for vector search integration tests.
pub fn vector_samples_migration() -> nest_data::SqlMigration {
    nest_data::SqlMigration::new(
        "002_vector_samples",
        format!(
            "CREATE TABLE nest_vector_samples (
                id UUID PRIMARY KEY,
                project_id TEXT NOT NULL,
                label TEXT NOT NULL,
                embedding vector({DEFAULT_EMBEDDING_DIM})
            );"
        ),
        "DROP TABLE nest_vector_samples;",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PostgresConfig;
    use crate::connection::PostgresConnection;
    use crate::migration::apply_migrations;
    use nest_data::Migration;
    use uuid::Uuid;

    fn sample_embedding(seed: f32) -> Vec<f32> {
        let mut values = vec![0.0_f32; DEFAULT_EMBEDDING_DIM];
        values[0] = seed;
        values[1] = 1.0 - seed;
        values
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL, PostgreSQL, and pgvector"]
    async fn similarity_search_orders_by_distance() {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
        let conn = PostgresConnection::connect(&PostgresConfig::new(url))
            .await
            .unwrap();
        let migrations: Vec<Box<dyn Migration>> = vec![
            Box::new(enable_vector_migration()),
            Box::new(vector_samples_migration()),
        ];
        apply_migrations(conn.pool(), &migrations)
            .await
            .unwrap();

        let project_id = "proj-test";
        let near_id = Uuid::new_v4();
        let far_id = Uuid::new_v4();
        let query = sample_embedding(0.9);

        sqlx::query(
            "INSERT INTO nest_vector_samples (id, project_id, label, embedding) VALUES ($1, $2, $3, $4)",
        )
        .bind(near_id)
        .bind(project_id)
        .bind("near")
        .bind(Vector::from(sample_embedding(0.95)))
        .execute(conn.pool())
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO nest_vector_samples (id, project_id, label, embedding) VALUES ($1, $2, $3, $4)",
        )
        .bind(far_id)
        .bind(project_id)
        .bind("far")
        .bind(Vector::from(sample_embedding(0.1)))
        .execute(conn.pool())
        .await
        .unwrap();

        let search = VectorSearch::new(conn.pool(), "nest_vector_samples", "id", "embedding")
            .with_project_scope("project_id");

        let hits = search
            .search_similar(&query, 2, Some(project_id))
            .await
            .unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].id, near_id.to_string());
        assert!(hits[0].distance <= hits[1].distance);

        sqlx::query("DELETE FROM nest_vector_samples WHERE project_id = $1")
            .bind(project_id)
            .execute(conn.pool())
            .await
            .unwrap();
    }
}
