use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

use thiserror::Error;

use crate::{PostgresDb, Result};

#[derive(Error, Debug)]
pub enum MigrateError {
    #[error("migration {0}: {1}")]
    Migration(String, String),

    #[error("duplicate migration version: {0}")]
    DuplicateVersion(String),

    #[error("missing migration file: {0}")]
    MissingFile(String),

    #[error("postgres error: {0}")]
    Postgres(#[from] crate::PostgresError),
}

pub struct Migration {
    pub version: String,
    pub description: String,
    pub sql_up: String,
    pub sql_down: Option<String>,
}

pub struct Migrator {
    migrations: BTreeMap<String, Migration>,
    table_name: String,
}

impl Migrator {
    pub fn new() -> Self {
        Self {
            migrations: BTreeMap::new(),
            table_name: "_klyron_migrations".to_string(),
        }
    }

    pub fn with_table(mut self, table: &str) -> Self {
        self.table_name = table.to_string();
        self
    }

    pub fn add_migration(&mut self, version: &str, description: &str, sql_up: &str, sql_down: Option<&str>) {
        if self.migrations.contains_key(version) {
            panic!("duplicate migration version: {}", version);
        }
        self.migrations.insert(version.to_string(), Migration {
            version: version.to_string(),
            description: description.to_string(),
            sql_up: sql_up.to_string(),
            sql_down: sql_down.map(|s| s.to_string()),
        });
    }

    pub fn add_migration_file(&mut self, path: &Path) -> std::result::Result<(), MigrateError> {
        let stem = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| MigrateError::MissingFile(path.display().to_string()))?;

        let parts: Vec<&str> = stem.splitn(2, '_').collect();
        if parts.len() < 2 {
            return Err(MigrateError::Migration(
                stem.to_string(),
                "filename must be <version>_<description>.sql".to_string(),
            ));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| MigrateError::Migration(stem.to_string(), e.to_string()))?;

        let (sql_up, sql_down) = if let Some(pos) = content.find("-- DOWN") {
            let up = content[..pos].trim().to_string();
            let down = content[pos + 8..].trim().to_string();
            (up, Some(down))
        } else {
            (content.trim().to_string(), None)
        };

        self.add_migration(parts[0], parts[1], &sql_up, sql_down.as_deref());
        Ok(())
    }

    pub fn load_migrations_dir(&mut self, dir: &Path) -> std::result::Result<(), MigrateError> {
        if !dir.exists() {
            return Ok(());
        }
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .map_err(|e| MigrateError::Migration("directory".to_string(), e.to_string()))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "sql").unwrap_or(false))
            .collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            self.add_migration_file(&entry.path())?;
        }
        Ok(())
    }

    pub async fn run(&self, db: &PostgresDb) -> Result<Vec<String>> {
        self.ensure_migrations_table(db).await?;

        let applied = self.get_applied_versions(db).await?;
        let mut executed = Vec::new();

        for (version, migration) in &self.migrations {
            if !applied.contains(version) {
                let start = Instant::now();
                db.execute(&migration.sql_up, &[]).await?;
                self.record_migration(db, version, &migration.description, start.elapsed()).await?;
                executed.push(version.clone());
            }
        }

        Ok(executed)
    }

    pub async fn rollback(&self, db: &PostgresDb, target_version: &str) -> Result<Vec<String>> {
        let applied = self.get_applied_versions(db).await?;
        let mut reverted = Vec::new();

        for (version, migration) in self.migrations.iter().rev() {
            if !applied.contains(version) {
                continue;
            }
            if version.as_str() <= target_version {
                break;
            }
            if let Some(ref down) = migration.sql_down {
                db.execute(down, &[]).await?;
                self.remove_migration(db, version).await?;
                reverted.push(version.clone());
            }
        }

        Ok(reverted)
    }

    pub fn pending(&self, applied: &[String]) -> Vec<&Migration> {
        self.migrations
            .values()
            .filter(|m| !applied.contains(&m.version))
            .collect()
    }

    async fn ensure_migrations_table(&self, db: &PostgresDb) -> Result<()> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                version VARCHAR(255) PRIMARY KEY,
                description TEXT NOT NULL,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                duration_ms BIGINT NOT NULL DEFAULT 0
            )",
            self.table_name
        );
        db.execute(&sql, &[]).await?;
        Ok(())
    }

    async fn get_applied_versions(&self, db: &PostgresDb) -> Result<Vec<String>> {
        let sql = format!("SELECT version FROM {} ORDER BY version", self.table_name);
        let rows = db.query(&sql, &[]).await?;
        Ok(rows.iter().filter_map(|r| r.get_str(0).map(String::from)).collect())
    }

    async fn record_migration(&self, db: &PostgresDb, version: &str, description: &str, duration: std::time::Duration) -> Result<()> {
        let sql = format!(
            "INSERT INTO {} (version, description, duration_ms) VALUES ($1, $2, $3)",
            self.table_name
        );
        db.execute(&sql, &[&version.to_string(), &description.to_string(), &(duration.as_millis() as i64)]).await?;
        Ok(())
    }

    async fn remove_migration(&self, db: &PostgresDb, version: &str) -> Result<()> {
        let sql = format!("DELETE FROM {} WHERE version = $1", self.table_name);
        db.execute(&sql, &[&version.to_string()]).await?;
        Ok(())
    }
}

impl Default for Migrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_migrator_construction() {
        let m = Migrator::new();
        assert!(m.migrations.is_empty());
    }

    #[test]
    fn test_add_migration() {
        let mut m = Migrator::new();
        m.add_migration("001", "create_users", "CREATE TABLE users (id SERIAL PRIMARY KEY)", None);
        assert_eq!(m.migrations.len(), 1);
    }

    #[test]
    #[should_panic(expected = "duplicate migration version")]
    fn test_duplicate_migration_panics() {
        let mut m = Migrator::new();
        m.add_migration("001", "first", "SELECT 1", None);
        m.add_migration("001", "second", "SELECT 2", None);
    }

    #[test]
    fn test_migration_with_down() {
        let mut m = Migrator::new();
        m.add_migration("001", "test", "CREATE TABLE t (id INT)", Some("DROP TABLE t"));
        assert!(m.migrations.get("001").unwrap().sql_down.is_some());
    }

    #[test]
    fn test_pending_migrations() {
        let mut m = Migrator::new();
        m.add_migration("001", "first", "SELECT 1", None);
        m.add_migration("002", "second", "SELECT 2", None);
        let applied = vec!["001".to_string()];
        let pending = m.pending(&applied);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].version, "002");
    }
}
