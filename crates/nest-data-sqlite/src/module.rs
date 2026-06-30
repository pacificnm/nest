//! Optional Nest module that registers a SQLite connection.

use nest_core::{AppBuilder, Module, ModuleId, NestResult};
use nest_data::{DataService, Migration, DATA_MODULE_ID};

use crate::config::SqliteConfig;
use crate::connection::SqliteConnection;
use crate::migration::apply_migrations;

/// Module id for [`SqliteDataModule`].
pub const SQLITE_DATA_MODULE_ID: ModuleId = ModuleId("nest-data-sqlite");

/// Registers a SQLite connection with [`DataService`].
pub struct SqliteDataModule {
    connection_id: nest_data::ConnectionId,
    config: SqliteConfig,
    migrations: Vec<Box<dyn Migration>>,
}

impl SqliteDataModule {
    /// Registers the primary SQLite connection.
    pub fn primary(path: impl AsRef<std::path::Path>) -> Self {
        Self::named(nest_data::ConnectionId::PRIMARY, path)
    }

    /// Registers a named SQLite connection.
    pub fn named(
        connection_id: impl Into<nest_data::ConnectionId>,
        path: impl AsRef<std::path::Path>,
    ) -> Self {
        Self {
            connection_id: connection_id.into(),
            config: SqliteConfig::file(path),
            migrations: Vec::new(),
        }
    }

    /// Uses an in-memory database (primarily for tests).
    pub fn memory() -> Self {
        Self {
            connection_id: nest_data::ConnectionId::new(nest_data::ConnectionId::PRIMARY),
            config: SqliteConfig::memory(),
            migrations: Vec::new(),
        }
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

impl Module for SqliteDataModule {
    fn id(&self) -> ModuleId {
        SQLITE_DATA_MODULE_ID
    }

    fn dependencies(&self) -> &'static [ModuleId] {
        &[DATA_MODULE_ID]
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let conn =
            SqliteConnection::open_named(self.connection_id.clone(), &self.config).map_err(|e| {
                nest_error::NestError::data(e.to_string()).with_source(e)
            })?;

        if !self.migrations.is_empty() {
            apply_migrations(&conn, &self.migrations).map_err(|e| {
                nest_error::NestError::data(e.to_string()).with_source(e)
            })?;
        }

        let data = app.service_mut::<DataService>()?;
        data.register_connection(conn.clone().as_data_connection())?;
        data.set_active(&self.connection_id)?;

        app.register_service(conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notes::{notes_migration, Note, NoteId, NotesRepository};
    use nest_data::{DataModule, ListQuery, Repository};

    struct NotesModule;

    impl Module for NotesModule {
        fn id(&self) -> ModuleId {
            ModuleId("test-notes")
        }

        fn dependencies(&self) -> &'static [ModuleId] {
            &[SQLITE_DATA_MODULE_ID]
        }

        fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
            let conn = app.service_mut::<SqliteConnection>()?.clone();
            app.register_service(NotesRepository::new(conn))
        }
    }

    #[test]
    fn module_registers_sqlite_and_data_service() {
        let built = nest_core::AppBuilder::new()
            .module(DataModule)
            .module(SqliteDataModule::memory().with_migration(Box::new(notes_migration())))
            .build()
            .unwrap();

        let data = built.context.service::<DataService>().unwrap();
        assert_eq!(data.active_id().unwrap().as_str(), "primary");
        assert!(built.context.has_service::<SqliteConnection>());
    }

    #[test]
    fn missing_data_module_dependency_fails() {
        let result = nest_core::AppBuilder::new()
            .module(SqliteDataModule::memory())
            .build();
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(
            err.code(),
            Some(nest_error::codes::NEST_MODULE_DEPENDENCY_MISSING)
        );
    }

    #[test]
    fn notes_repository_via_modules() {
        let built = nest_core::AppBuilder::new()
            .module(DataModule)
            .module(SqliteDataModule::memory().with_migration(Box::new(notes_migration())))
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
            .unwrap();
        assert_eq!(repo.list(ListQuery::new()).unwrap().len(), 1);
        repo.delete(note.id).unwrap();
    }

    #[test]
    fn second_named_connection() {
        struct CacheModule;

        impl Module for CacheModule {
            fn id(&self) -> ModuleId {
                ModuleId("test-cache")
            }

            fn dependencies(&self) -> &'static [ModuleId] {
                &[DATA_MODULE_ID]
            }

            fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
                let conn = SqliteConnection::open_named("cache", &SqliteConfig::memory())
                    .map_err(|e| nest_error::NestError::data(e.to_string()).with_source(e))?;
                let data = app.service_mut::<DataService>()?;
                data.register_connection(conn.as_data_connection())?;
                Ok(())
            }
        }

        let built = nest_core::AppBuilder::new()
            .module(DataModule)
            .module(SqliteDataModule::memory())
            .module(CacheModule)
            .build()
            .unwrap();

        let data = built.context.service::<DataService>().unwrap();
        let ids: Vec<_> = data.list_connections().into_iter().map(|id| id.to_string()).collect();
        assert!(ids.iter().any(|id| id == "primary"));
        assert!(ids.iter().any(|id| id == "cache"));
    }
}
