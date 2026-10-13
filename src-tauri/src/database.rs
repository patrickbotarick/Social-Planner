use crate::error::AppError;
use rusqlite::{params, Connection, TransactionBehavior};
use serde::Serialize;
use std::{path::Path, time::Duration};

const MIGRATIONS: &[(i64, &str, &str)] = &[(
    1,
    "foundation",
    include_str!("../migrations/001_foundation.sql"),
)];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub timezone: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FoundationStatus {
    pub schema_version: i64,
    pub database_path: String,
    pub integrity: &'static str,
    pub settings: AppSettings,
}

pub struct Repository {
    connection: Connection,
    database_path: String,
}

impl Repository {
    pub fn open(path: &Path) -> Result<Self, AppError> {
        let mut connection = Connection::open(path)?;
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        check_integrity(&connection)?;
        migrate(&mut connection, MIGRATIONS)?;
        check_integrity(&connection)?;
        Ok(Self {
            connection,
            database_path: path.to_string_lossy().into_owned(),
        })
    }

    pub fn settings(&self) -> Result<AppSettings, AppError> {
        Ok(self.connection.query_row(
            "SELECT timezone, created_at, updated_at FROM app_settings WHERE id = 1",
            [],
            |row| {
                Ok(AppSettings {
                    timezone: row.get(0)?,
                    created_at: row.get(1)?,
                    updated_at: row.get(2)?,
                })
            },
        )?)
    }

    pub fn status(&self) -> Result<FoundationStatus, AppError> {
        check_integrity(&self.connection)?;
        let schema_version = self.connection.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?;
        Ok(FoundationStatus {
            schema_version,
            database_path: self.database_path.clone(),
            integrity: "ok",
            settings: self.settings()?,
        })
    }
}

fn check_integrity(connection: &Connection) -> Result<(), AppError> {
    let result: String = connection
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(|_| AppError::integrity())?;
    if result != "ok" {
        return Err(AppError::integrity());
    }
    Ok(())
}

fn migrate(connection: &mut Connection, migrations: &[(i64, &str, &str)]) -> Result<(), AppError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let has_ledger: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'schema_migrations')",
        [],
        |row| row.get(0),
    )?;
    if !has_ledger {
        let existing_tables: i64 = transaction.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )?;
        if existing_tables != 0 {
            return Err(AppError::schema());
        }
    }
    transaction.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            source_sql TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );",
    )?;
    let applied = {
        let mut statement = transaction
            .prepare("SELECT version, name, source_sql FROM schema_migrations ORDER BY version")?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    for (index, (version, name, source)) in applied.iter().enumerate() {
        match migrations.get(index) {
            Some((expected_version, expected_name, expected_source))
                if version == expected_version
                    && name.as_str() == *expected_name
                    && source.as_str() == *expected_source => {}
            _ => return Err(AppError::schema()),
        }
    }
    for (version, name, sql) in migrations.iter().skip(applied.len()) {
        transaction.execute_batch(sql)?;
        transaction.execute(
            "INSERT INTO schema_migrations (version, name, source_sql) VALUES (?1, ?2, ?3)",
            params![version, name, sql],
        )?;
    }
    transaction.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialization_is_idempotent_and_settings_persist_after_reopening() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("test.sqlite3");
        {
            let repository = Repository::open(&path).unwrap();
            assert_eq!(repository.status().unwrap().schema_version, 1);
            assert_eq!(repository.settings().unwrap().timezone, "America/Sao_Paulo");
            // Exercita persistência sem expor edição de configurações no IPC desta fase.
            repository
                .connection
                .execute(
                    "UPDATE app_settings SET timezone = ?1 WHERE id = 1",
                    ["UTC"],
                )
                .unwrap();
        }
        let reopened = Repository::open(&path).unwrap();
        assert_eq!(reopened.settings().unwrap().timezone, "UTC");
        let count: i64 = reopened
            .connection
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn failed_migration_rolls_back_all_ddl_and_metadata() {
        let mut connection = Connection::open_in_memory().unwrap();
        let migrations = [(1, "broken", "CREATE TABLE example (id INTEGER); INVALID SQL;")];
        assert!(migrate(&mut connection, &migrations).is_err());
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn changed_or_future_migrations_are_rejected_without_reset() {
        let mut connection = Connection::open_in_memory().unwrap();
        migrate(&mut connection, MIGRATIONS).unwrap();
        connection
            .execute("UPDATE schema_migrations SET source_sql = 'changed'", [])
            .unwrap();
        assert_eq!(
            migrate(&mut connection, MIGRATIONS).unwrap_err().code,
            "SCHEMA_UNSUPPORTED"
        );
        connection
            .execute("UPDATE schema_migrations SET version = 99", [])
            .unwrap();
        assert_eq!(
            migrate(&mut connection, MIGRATIONS).unwrap_err().code,
            "SCHEMA_UNSUPPORTED"
        );
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM app_settings", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn corrupt_database_is_not_recreated() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("corrupt.sqlite3");
        let original = b"this is not a SQLite database";
        std::fs::write(&path, original).unwrap();
        assert!(Repository::open(&path).is_err());
        assert_eq!(std::fs::read(path).unwrap(), original);
    }

    #[test]
    fn unversioned_existing_database_is_preserved() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch("CREATE TABLE important_data (id INTEGER);")
            .unwrap();
        assert_eq!(
            migrate(&mut connection, MIGRATIONS).unwrap_err().code,
            "SCHEMA_UNSUPPORTED"
        );
        assert!(connection.prepare("SELECT * FROM important_data").is_ok());
    }

    #[test]
    fn settings_constraints_prevent_duplicate_and_empty_preferences() {
        let mut connection = Connection::open_in_memory().unwrap();
        migrate(&mut connection, MIGRATIONS).unwrap();
        assert!(connection
            .execute(
                "INSERT INTO app_settings (id, timezone) VALUES (2, 'UTC')",
                [],
            )
            .is_err());
        assert!(connection
            .execute("UPDATE app_settings SET timezone = ' ' WHERE id = 1", [])
            .is_err());
    }
}
