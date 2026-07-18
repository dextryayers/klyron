use std::collections::BTreeMap;
use std::path::Path;

use crate::error::{MySqlError, Result};
use crate::pool::MySqlDb;

pub struct Migration {
    pub version: i64,
    pub name: String,
    pub up: String,
    pub down: Option<String>,
}

pub struct Migrator {
    pub table: String,
}

impl Default for Migrator {
    fn default() -> Self {
        Self { table: "_migrations".into() }
 }
}

impl Migrator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_table(table: &str) -> Self {
        Self { table: table.to_string() }
    }

    pub async fn ensure_table(&self, db: &MySqlDb) -> Result<()> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                version BIGINT PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            self.table
        );
        db.execute(&sql).await?;
        Ok(())
    }

    pub async fn applied_versions(&self, db: &MySqlDb) -> Result<BTreeMap<i64, String>> {
        let sql = format!("SELECT version, name FROM {} ORDER BY version", self.table);
        let rows = db.query(&sql).await?;
        let mut map = BTreeMap::new();
        for row in &rows {
            if let Some(ver) = row.get_int(0) {
                let name = row.get_str(1).unwrap_or("").to_string();
                map.insert(ver, name);
            }
        }
        Ok(map)
    }

    pub async fn run_migration(&self, db: &MySqlDb, m: &Migration) -> Result<()> {
        db.execute(&m.up).await?;
        let sql = format!(
            "INSERT INTO {} (version, name) VALUES ({}, '{}')",
            self.table, m.version, m.name.replace('\'', "''")
        );
        db.execute(&sql).await?;
        Ok(())
    }

    pub async fn rollback_migration(&self, db: &MySqlDb, m: &Migration) -> Result<()> {
        if let Some(ref down) = m.down {
            db.execute(down).await?;
        }
        let sql = format!("DELETE FROM {} WHERE version = {}", self.table, m.version);
        db.execute(&sql).await?;
        Ok(())
    }

    pub async fn migrate(&self, db: &MySqlDb, migrations: &[Migration]) -> Result<()> {
        self.ensure_table(db).await?;
        let applied = self.applied_versions(db).await?;
        for m in migrations {
            if !applied.contains_key(&m.version) {
                self.run_migration(db, m).await?;
            }
        }
        Ok(())
    }

    pub async fn migrate_up(&self, db: &MySqlDb, migrations: &[Migration], target: i64) -> Result<()> {
        self.ensure_table(db).await?;
        let applied = self.applied_versions(db).await?;
        for m in migrations {
            if m.version > target {
                break;
            }
            if !applied.contains_key(&m.version) {
                self.run_migration(db, m).await?;
            }
        }
        Ok(())
    }

    pub async fn migrate_down(&self, db: &MySqlDb, migrations: &[Migration], target: i64) -> Result<()> {
        let applied = self.applied_versions(db).await?;
        for m in migrations.iter().rev() {
            if m.version <= target {
                break;
            }
            if applied.contains_key(&m.version) {
                self.rollback_migration(db, m).await?;
            }
        }
        Ok(())
    }

    pub fn read_migrations(dir: &Path) -> Result<Vec<Migration>> {
        let mut migrations = Vec::new();
        if !dir.exists() {
            return Ok(migrations);
        }
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .map_err(|e| MySqlError::Migration(format!("read dir: {e}")))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "sql"))
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let fname = entry.file_name();
            let fname_str = fname.to_string_lossy();
            let parts: Vec<&str> = fname_str.splitn(3, '_').collect();
            if parts.len() < 2 {
                continue;
            }
            let version: i64 = parts[0].parse().map_err(|_| MySqlError::Migration(format!("invalid version in {fname_str}")))?;
            let name = if parts.len() > 2 {
                parts[2].trim_end_matches(".sql").to_string()
            } else {
                parts[1].trim_end_matches(".sql").to_string()
            };
            let content = std::fs::read_to_string(entry.path())
                .map_err(|e| MySqlError::Migration(format!("read {fname_str}: {e}")))?;
            let mut up = content;
            let _down: Option<String> = None;
            if let Some(idx) = up.find("-- DOWN") {
                let rest = up.split_off(idx);
                up = up.trim().to_string();
                let down = rest.trim_start_matches("-- DOWN").trim().to_string();
                migrations.push(Migration { version, name, up, down: Some(down) });
            } else {
                migrations.push(Migration { version, name, up: up.trim().to_string(), down: None });
            }
        }
        Ok(migrations)
    }
}
