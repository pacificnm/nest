//! Optional Nest module that registers a PostgreSQL connection pool.

use nest_core::{AppBuilder, Module, ModuleId, NestResult};
use nest_data::{DataService, Migration, DATA_MODULE_ID};

use crate::config::PostgresConfig;
use crate::connection::PostgresConnection;
use crate::migration::apply_migrations;
use crate::runtime::block_on;

/// Module id for [`PostgresDataModule`].
pub const POSTGRES_DATA_MODULE_ID: ModuleId = ModuleId("nest-data-postgres");

/// Registers a PostgreSQL pool with [`DataService`].
pub struct PostgresDataModule {
    connection_id: nest_data::ConnectionId,
    config: PostgresConfig,
    migrations: Vec<Box<dyn Migration>>,
}

impl PostgresDataModule {
    /// Registers the primary PostgreSQL connection from config.
    pub fn new(config: PostgresConfig) -> Self {
        Self::named(nest_data::ConnectionId::PRIMARY, config)
    }

    /// Registers a named PostgreSQL connection.
    pub fn named(
        connection_id: impl Into<nest_data::ConnectionId>,
        config: PostgresConfig,
    ) -> Self {
        Self {
            connection_id: connection_id.into(),
            config,
            migrations: Vec::new(),
        }
    }

    /// Builds config from `DATABASE_URL` (or another env var).
    pub fn from_env(var: &str) -> nest_data::DataResult<Self> {
        Ok(Self::new(PostgresConfig::from_env(var)?))
    }

    /// Adds migrations applied before the connection is registered.
    pub fn with_migration(mut self, migration: Box<dyn Migration>) -> Self {
        self.migrations.push(migration);
        self
    }

    /// Adds multiple migrations.
    pub fn with_migrations(
        mut self,
        migrations: impl IntoIterator<Item = Box<dyn Migration>>,
    ) -> Self {
        self.migrations.extend(migrations);
        self
    }
}

impl Module for PostgresDataModule {
    fn id(&self) -> ModuleId {
        POSTGRES_DATA_MODULE_ID
    }

    fn dependencies(&self) -> &'static [ModuleId] {
        &[DATA_MODULE_ID]
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let conn = block_on(PostgresConnection::connect_named(
            self.connection_id.clone(),
            &self.config,
        ))
        .map_err(|e| nest_error::NestError::data(e.to_string()).with_source(e))?;

        if !self.migrations.is_empty() {
            block_on(apply_migrations(conn.pool(), &self.migrations))
                .map_err(|e| nest_error::NestError::data(e.to_string()).with_source(e))?;
        }

        let data = app.service_mut::<DataService>()?;
        data.register_connection(conn.clone().as_data_connection())?;
        data.set_active(conn.connection_id())?;

        app.register_service(conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notes::{notes_migration, Note, NoteId, NotesRepository};
    use nest_data::AsyncRepository;
    use nest_data::{DataModule, ListQuery};

    struct NotesModule;

    impl Module for NotesModule {
        fn id(&self) -> ModuleId {
            ModuleId("test-notes")
        }

        fn dependencies(&self) -> &'static [ModuleId] {
            &[POSTGRES_DATA_MODULE_ID]
        }

        fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
            let conn = app.service_mut::<PostgresConnection>()?.clone();
            app.register_service(NotesRepository::new(conn))
        }
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL and PostgreSQL"]
    async fn module_registers_postgres_and_data_service() {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
        let built = nest_core::AppBuilder::new()
            .module(DataModule)
            .module(
                PostgresDataModule::new(PostgresConfig::new(url))
                    .with_migration(Box::new(notes_migration())),
            )
            .build()
            .unwrap();

        let data = built.context.service::<DataService>().unwrap();
        assert_eq!(data.active_id().unwrap().as_str(), "primary");
        assert!(built.context.has_service::<PostgresConnection>());
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL and PostgreSQL"]
    async fn notes_repository_via_modules() {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
        let built = nest_core::AppBuilder::new()
            .module(DataModule)
            .module(
                PostgresDataModule::new(PostgresConfig::new(url))
                    .with_migration(Box::new(notes_migration())),
            )
            .module(NotesModule)
            .build()
            .unwrap();

        let repo = built.context.service::<NotesRepository>().unwrap();
        let note = repo
            .insert(Note {
                id: NoteId(0),
                title: "Test".into(),
                body: "Body".into(),
            })
            .await
            .unwrap();
        assert!(note.id.0 > 0);
        assert_eq!(repo.list(ListQuery::new()).await.unwrap().len(), 1);
        repo.delete(note.id).await.unwrap();
    }

    #[test]
    fn missing_data_module_dependency_fails() {
        let url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://localhost/test".into());
        let result = nest_core::AppBuilder::new()
            .module(PostgresDataModule::new(PostgresConfig::new(url)))
            .build();
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(
            err.code(),
            Some(nest_error::codes::NEST_MODULE_DEPENDENCY_MISSING)
        );
    }
}
