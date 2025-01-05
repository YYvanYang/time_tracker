// src/storage/migrations.rs

use crate::error::{Result, TimeTrackerError};
use rusqlite::Connection;
use std::collections::HashMap;

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "Initial schema",
        up_sql: include_str!("../../migrations/001_initial_schema.sql"),
        down_sql: include_str!("../../migrations/001_initial_schema_down.sql"),
    },
    Migration {
        version: 2,
        description: "Add productivity tracking",
        up_sql: include_str!("../../migrations/002_add_productivity.sql"),
        down_sql: include_str!("../../migrations/002_add_productivity_down.sql"),
    },
    Migration {
        version: 3,
        description: "Add indexes",
        up_sql: include_str!("../../migrations/003_add_indexes.sql"),
        down_sql: include_str!("../../migrations/003_add_indexes_down.sql"),
    },
];

pub struct Migration {
    pub version: i32,
    pub description: &'static str,
    pub up_sql: &'static str,
    pub down_sql: &'static str,
}

#[derive(Debug)]
pub struct MigrationRecord {
    pub version: i32,
    pub description: String,
    pub applied_at: chrono::DateTime<chrono::Local>,
}

pub fn run_migrations(conn: &Connection) -> Result<()> {
    // 创建迁移记录表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // 获取已应用的迁移版本
    let applied_versions: HashMap<i32, String> = conn
        .prepare("SELECT version, description FROM schema_migrations")?
        .query_map([], |row| {
            Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    // 在事务中执行迁移
    let tx = conn.transaction()?;

    for migration in MIGRATIONS {
        if !applied_versions.contains_key(&migration.version) {
            log::info!(
                "Running migration {} - {}",
                migration.version,
                migration.description
            );

            match tx.execute_batch(migration.up_sql) {
                Ok(_) => {
                    // 记录迁移
                    tx.execute(
                        "INSERT INTO schema_migrations (version, description) VALUES (?1, ?2)",
                        [&migration.version, &migration.description],
                    )?;
                    log::info!("Migration {} completed successfully", migration.version);
                }
                Err(e) => {
                    log::error!("Migration {} failed: {}", migration.version, e);
                    tx.rollback()?;
                    return Err(TimeTrackerError::Database(format!(
                        "Migration {} failed: {}",
                        migration.version,
                        e
                    )));
                }
            }
        } else if applied_versions[&migration.version] != migration.description {
            // 迁移描述不匹配，可能表示数据库被篡改
            return Err(TimeTrackerError::Database(format!(
                "Migration {} description mismatch. Expected '{}', found '{}'",
                migration.version,
                migration.description,
                applied_versions[&migration.version]
            )));
        }
    }

    tx.commit()?;
    Ok(())
}

pub fn rollback_migration(conn: &Connection, version: i32) -> Result<()> {
    let tx = conn.transaction()?;

    if let Some(migration) = MIGRATIONS.iter().find(|m| m.version == version) {
        // 检查是否为最后应用的迁移
        let last_version: i32 = tx.query_row(
            "SELECT MAX(version) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?;

        if version != last_version {
            return Err(TimeTrackerError::Database(
                "Can only rollback the last applied migration".into(),
            ));
        }

        log::info!(
            "Rolling back migration {} - {}",
            migration.version,
            migration.description
        );

        match tx.execute_batch(migration.down_sql) {
            Ok(_) => {
                tx.execute(
                    "DELETE FROM schema_migrations WHERE version = ?1",
                    [version],
                )?;
                log::info!("Rollback of migration {} completed", version);
            }
            Err(e) => {
                log::error!("Rollback of migration {} failed: {}", version, e);
                tx.rollback()?;
                return Err(TimeTrackerError::Database(format!(
                    "Rollback failed: {}",
                    e
                )));
            }
        }

        tx.commit()?;
        Ok(())
    } else {
        Err(TimeTrackerError::Database(format!(
            "Migration version {} not found",
            version
        )))
    }
}

pub fn get_migration_history(conn: &Connection) -> Result<Vec<MigrationRecord>> {
    let mut stmt = conn.prepare(
        "SELECT version, description, applied_at 
         FROM schema_migrations 
         ORDER BY version ASC"
    )?;

    let records = stmt.query_map([], |row| {
        Ok(MigrationRecord {
            version: row.get(0)?,
            description: row.get(1)?,
            applied_at: row.get(2)?,
        })
    })?;

    records
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| TimeTrackerError::Database(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_db() -> (Connection, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let conn = Connection::open(temp_dir.path().join("test.db")).unwrap();
        (conn, temp_dir)
    }

    #[test]
    fn test_migration_execution() -> Result<()> {
        let (conn, _temp_dir) = create_test_db();

        // 运行迁移
        run_migrations(&conn)?;

        // 验证迁移记录
        let history = get_migration_history(&conn)?;
        assert_eq!(history.len(), MIGRATIONS.len());
        
        // 验证表是否创建
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")?
            .query_map([], |row| row.get(0))?
            .collect::<std::result::Result<_, _>>()?;

        assert!(tables.contains(&"app_usage".to_string()));
        assert!(tables.contains(&"pomodoro_records".to_string()));

        Ok(())
    }

    #[test]
    fn test_rollback_migration() -> Result<()> {
        let (conn, _temp_dir) = create_test_db();

        // 运行所有迁移
        run_migrations(&conn)?;

        // 回滚最后一个迁移
        let last_version = MIGRATIONS.last().unwrap().version;
        rollback_migration(&conn, last_version)?;

        // 验证迁移历史
        let history = get_migration_history(&conn)?;
        assert_eq!(history.len(), MIGRATIONS.len() - 1);

        // 验证不能回滚非最后一个迁移
        let result = rollback_migration(&conn, 1);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_idempotency() -> Result<()> {
        let (conn, _temp_dir) = create_test_db();

        // 运行迁移两次
        run_migrations(&conn)?;
        run_migrations(&conn)?;

        // 验证迁移只应用一次
        let history = get_migration_history(&conn)?;
        assert_eq!(history.len(), MIGRATIONS.len());

        Ok(())
    }
}