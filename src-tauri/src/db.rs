use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenameRecord {
    pub id: Option<i64>,
    pub original_path: String,
    pub new_path: String,
    pub bitrate: u8,
    pub status: String,
    pub timestamp: String,
    pub session_id: String,
}

pub fn init_db(db_path: &str) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS rename_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            original_path TEXT NOT NULL,
            new_path TEXT NOT NULL,
            bitrate INTEGER NOT NULL,
            status TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            session_id TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_session ON rename_history(session_id);
        CREATE INDEX IF NOT EXISTS idx_timestamp ON rename_history(timestamp);
    ")?;
    Ok(conn)
}

pub fn insert_record(conn: &Connection, record: &RenameRecord) -> Result<i64> {
    conn.execute(
        "INSERT INTO rename_history (original_path, new_path, bitrate, status, timestamp, session_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            record.original_path,
            record.new_path,
            record.bitrate,
            record.status,
            record.timestamp,
            record.session_id
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_recent_records(conn: &Connection, limit: i64) -> Result<Vec<RenameRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, original_path, new_path, bitrate, status, timestamp, session_id
         FROM rename_history
         ORDER BY timestamp DESC
         LIMIT ?1"
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        Ok(RenameRecord {
            id: Some(row.get(0)?),
            original_path: row.get(1)?,
            new_path: row.get(2)?,
            bitrate: row.get::<_, i64>(3)? as u8,
            status: row.get(4)?,
            timestamp: row.get(5)?,
            session_id: row.get(6)?,
        })
    })?;
    rows.collect()
}

pub fn get_records_by_session(conn: &Connection, session_id: &str) -> Result<Vec<RenameRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, original_path, new_path, bitrate, status, timestamp, session_id
         FROM rename_history
         WHERE session_id = ?1
         ORDER BY timestamp DESC"
    )?;
    let rows = stmt.query_map(params![session_id], |row| {
        Ok(RenameRecord {
            id: Some(row.get(0)?),
            original_path: row.get(1)?,
            new_path: row.get(2)?,
            bitrate: row.get::<_, i64>(3)? as u8,
            status: row.get(4)?,
            timestamp: row.get(5)?,
            session_id: row.get(6)?,
        })
    })?;
    rows.collect()
}

pub fn get_latest_session_id(conn: &Connection) -> Result<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT session_id FROM rename_history ORDER BY timestamp DESC LIMIT 1"
    )?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn delete_all_history(conn: &Connection) -> Result<usize> {
    conn.execute("DELETE FROM rename_history", [])
}


pub fn generate_session_id() -> String {
    format!("session_{}", Utc::now().timestamp_millis())
}