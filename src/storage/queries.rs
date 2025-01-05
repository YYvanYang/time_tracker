// src/storage/queries.rs

use super::models::*;
use crate::error::{Result, TimeTrackerError};
use chrono::{DateTime, Local, NaiveDateTime, Timelike};
use rusqlite::{params, Connection, Row, Statement};
use std::collections::HashMap;
use std::time::Duration;
use test_log;

pub trait FromRow {
    fn from_row(row: &Row) -> rusqlite::Result<Self>
    where
        Self: Sized;
}

impl FromRow for AppUsageRecord {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            app_name: row.get(1)?,
            window_title: row.get(2)?,
            start_time: DateTime::from_naive_utc_and_offset(
                NaiveDateTime::parse_from_str(&row.get::<_, String>(3)?, "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
                *Local::now().offset(),
            ),
            duration: Duration::from_secs(row.get::<_, i64>(4)? as u64),
            category: row.get(5)?,
            is_productive: row.get(6)?,
        })
    }
}

impl FromRow for PomodoroRecord {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            start_time: DateTime::from_naive_utc_and_offset(
                NaiveDateTime::parse_from_str(&row.get::<_, String>(1)?, "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
                *Local::now().offset(),
            ),
            end_time: DateTime::from_naive_utc_and_offset(
                NaiveDateTime::parse_from_str(&row.get::<_, String>(2)?, "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
                *Local::now().offset(),
            ),
            status: if row.get::<_, String>(3)? == "completed" {
                PomodoroStatus::Completed
            } else {
                PomodoroStatus::Interrupted
            },
            notes: row.get(4)?,
            tags: Vec::new(), // Tags需要单独查询
            project_id: row.get(5)?,
        })
    }
}

// 查询构建器
pub struct QueryBuilder<'a, T> {
    conn: &'a Connection,
    table: &'static str,
    conditions: Vec<String>,
    params: Vec<Box<dyn rusqlite::ToSql + 'a>>,
    order_by: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T: FromRow> QueryBuilder<'a, T> {
    pub fn new(conn: &'a Connection, table: &'static str) -> Self {
        Self {
            conn,
            table,
            conditions: Vec::new(),
            params: Vec::new(),
            order_by: None,
            limit: None,
            offset: None,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn where_eq<V: rusqlite::ToSql + 'a>(mut self, field: &str, value: V) -> Self {
        self.conditions.push(format!("{} = ?", field));
        self.params.push(Box::new(value));
        self
    }

    pub fn where_gt<V: rusqlite::ToSql + 'a>(mut self, field: &str, value: V) -> Self {
        self.conditions.push(format!("{} > ?", field));
        self.params.push(Box::new(value));
        self
    }

    pub fn where_lt<V: rusqlite::ToSql + 'a>(mut self, field: &str, value: V) -> Self {
        self.conditions.push(format!("{} < ?", field));
        self.params.push(Box::new(value));
        self
    }

    pub fn order_by(mut self, order: &str) -> Self {
        self.order_by = Some(order.to_string());
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn build_query(&self) -> String {
        let mut query = format!("SELECT * FROM {}", self.table);
        
        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.conditions.join(" AND "));
        }

        if let Some(ref order) = self.order_by {
            query.push_str(" ORDER BY ");
            query.push_str(order);
        }

        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        query
    }

    pub fn execute(&self) -> Result<Vec<T>> {
        let query = self.build_query();
        let mut stmt = self.conn.prepare(&query)?;
        
        let params: Vec<&dyn rusqlite::ToSql> = self.params
            .iter()
            .map(|p| p.as_ref())
            .collect();

        let rows = stmt.query_map(params.as_slice(), |row| T::from_row(row))?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| TimeTrackerError::Database(e.to_string()))
    }

    pub fn execute_one(&self) -> Result<Option<T>> {
        let mut results = self.limit(1).execute()?;
        Ok(results.pop())
    }
}

// 应用使用记录查询
pub fn get_app_usage_by_date_range(
    conn: &Connection,
    start: DateTime<Local>,
    end: DateTime<Local>,
) -> Result<Vec<AppUsageRecord>> {
    QueryBuilder::<AppUsageRecord>::new(conn, "app_usage")
        .where_gt("start_time", start.to_rfc3339())
        .where_lt("start_time", end.to_rfc3339())
        .order_by("start_time DESC")
        .execute()
}

pub fn get_daily_app_usage(
    conn: &Connection,
    date: DateTime<Local>,
) -> Result<Vec<AppUsageRecord>> {
    let start = date.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let end = date.date_naive().and_hms_opt(23, 59, 59).unwrap();

    QueryBuilder::<AppUsageRecord>::new(conn, "app_usage")
        .where_gt("start_time", start.to_string())
        .where_lt("start_time", end.to_string())
        .execute()
}

// 番茄钟记录查询
pub fn get_pomodoro_records_by_date_range(
    conn: &Connection,
    start: DateTime<Local>,
    end: DateTime<Local>,
) -> Result<Vec<PomodoroRecord>> {
    let mut records = QueryBuilder::<PomodoroRecord>::new(conn, "pomodoro_records")
        .where_gt("start_time", start.to_rfc3339())
        .where_lt("start_time", end.to_rfc3339())
        .order_by("start_time DESC")
        .execute()?;

    // 加载每个记录的标签
    for record in &mut records {
        record.tags = get_pomodoro_tags(conn, record.id.unwrap())?;
    }

    Ok(records)
}

pub fn get_pomodoro_tags(conn: &Connection, pomodoro_id: i64) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT t.name FROM tags t
         INNER JOIN pomodoro_tags pt ON pt.tag_id = t.id
         WHERE pt.pomodoro_id = ?"
    )?;

    let tags = stmt.query_map([pomodoro_id], |row| row.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| TimeTrackerError::Database(e.to_string()))?;

    Ok(tags)
}

// 统计查询
pub fn get_productivity_stats(
    conn: &Connection,
    start: DateTime<Local>,
    end: DateTime<Local>,
) -> Result<ProductivityStats> {
    let records = get_app_usage_by_date_range(conn, start, end)?;

    let mut total_time = Duration::from_secs(0);
    let mut productive_time = Duration::from_secs(0);
    let mut hourly_productivity = HashMap::new();
    let mut daily_productivity = HashMap::new();

    for record in records {
        total_time += record.duration;
        if record.is_productive {
            productive_time += record.duration;
        }

        // 按小时统计
        let hour = record.start_time.hour();
        let entry = hourly_productivity.entry(hour).or_insert((Duration::from_secs(0), Duration::from_secs(0)));
        entry.0 += record.duration;
        if record.is_productive {
            entry.1 += record.duration;
        }

        // 按日统计
        let day = record.start_time.weekday().num_days_from_monday();
        let entry = daily_productivity.entry(day).or_insert((Duration::from_secs(0), Duration::from_secs(0)));
        entry.0 += record.duration;
        if record.is_productive {
            entry.1 += record.duration;
        }
    }

    // 找出最高效的时段
    let most_productive_hour = hourly_productivity
        .iter()
        .max_by_key(|(_, (total, productive))| {
            if total.as_secs() == 0 {
                0
            } else {
                (productive.as_secs_f64() / total.as_secs_f64() * 1000.0) as u64
            }
        })
        .map(|(hour, _)| *hour);

    let most_productive_day = daily_productivity
        .iter()
        .max_by_key(|(_, (total, productive))| {
            if total.as_secs() == 0 {
                0
            } else {
                (productive.as_secs_f64() / total.as_secs_f64() * 1000.0) as u64
            }
        })
        .map(|(day, _)| *day);

    Ok(ProductivityStats {
        total_time,
        productive_time,
        productivity_ratio: if total_time.as_secs() == 0 {
            0.0
        } else {
            productive_time.as_secs_f64() / total_time.as_secs_f64()
        },
        most_productive_hour,
        most_productive_day,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log;
    
    // 使用fixture模式
    struct TestFixture {
        conn: Connection,
        _temp_dir: TempDir, // 保持TempDir活着
    }
    
    impl TestFixture {
        fn new() -> Self {
            let temp_dir = TempDir::new().unwrap();
            let conn = Connection::open(temp_dir.path().join("test.db")).unwrap();
            
            // 初始化测试数据库
            conn.execute_batch(include_str!("../../tests/fixtures/schema.sql")).unwrap();
            
            Self {
                conn,
                _temp_dir: temp_dir,
            }
        }
    }

    #[test]
    fn test_query_builder() {
        let fixture = TestFixture::new();
        // 使用fixture进行测试
    }
}