use std::fs;
use std::path::{Path, PathBuf};
use crate::scanner::AudioFile;
use crate::db::{RenameRecord, insert_record};
use rusqlite::Connection;
use chrono::Utc;

/// Sanitize a string for use in filenames
fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Build the target filename for a primary file
pub fn build_target_name(file: &AudioFile) -> String {
    format!(
        "{} - {} - {} BIT.{}",
        sanitize(&file.title),
        sanitize(&file.album),
        file.bitrate,
        file.extension
    )
}

/// Find a non-conflicting path by appending (1), (2), etc.
fn resolve_conflict(target: &Path) -> PathBuf {
    if !target.exists() {
        return target.to_path_buf();
    }
    let stem = target
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = target
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let parent = target.parent().unwrap_or(Path::new("."));

    let mut counter = 1u32;
    loop {
        let candidate = parent.join(format!("{} ({}).{}", stem, counter, ext));
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}

pub struct RenameResult {
    pub records: Vec<RenameRecord>,
    pub errors: Vec<String>,
}

pub fn process_files(
    files: &[AudioFile],
    source_dir: &str,
    session_id: &str,
    conn: &Connection,
) -> RenameResult {
    let mut records = Vec::new();
    let mut errors = Vec::new();

    let duplicates_dir = PathBuf::from(source_dir).join("duplicates");

    for file in files {
        let original_path = PathBuf::from(&file.path);
        let parent = original_path.parent().unwrap_or(Path::new(source_dir));

        match file.status.as_str() {
            "primary" | "unique" => {
                let target_name = build_target_name(file);
                let current_name = original_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                // Skip if already correctly named
                if current_name == target_name {
                    let record = RenameRecord {
                        id: None,
                        original_path: file.path.clone(),
                        new_path: file.path.clone(),
                        bitrate: file.bitrate,
                        status: "Skipped".to_string(),
                        timestamp: Utc::now().to_rfc3339(),
                        session_id: session_id.to_string(),
                    };
                    let _ = insert_record(conn, &record);
                    records.push(record);
                    continue;
                }

                let raw_target = parent.join(&target_name);
                let target = resolve_conflict(&raw_target);

                match fs::rename(&original_path, &target) {
                    Ok(_) => {
                        let record = RenameRecord {
                            id: None,
                            original_path: file.path.clone(),
                            new_path: target.to_string_lossy().to_string(),
                            bitrate: file.bitrate,
                            status: "Renamed".to_string(),
                            timestamp: Utc::now().to_rfc3339(),
                            session_id: session_id.to_string(),
                        };
                        let _ = insert_record(conn, &record);
                        records.push(record);
                    }
                    Err(e) => {
                        errors.push(format!("Rename failed for {}: {}", file.path, e));
                    }
                }
            }

            "duplicate" => {
                if !duplicates_dir.exists() {
                    if let Err(e) = fs::create_dir_all(&duplicates_dir) {
                        errors.push(format!("Cannot create duplicates dir: {}", e));
                        continue;
                    }
                }

                let file_name = original_path
                    .file_name()
                    .unwrap_or_default();
                let raw_target = duplicates_dir.join(file_name);
                let target = resolve_conflict(&raw_target);

                match fs::rename(&original_path, &target) {
                    Ok(_) => {
                        let record = RenameRecord {
                            id: None,
                            original_path: file.path.clone(),
                            new_path: target.to_string_lossy().to_string(),
                            bitrate: file.bitrate,
                            status: "Moved".to_string(),
                            timestamp: Utc::now().to_rfc3339(),
                            session_id: session_id.to_string(),
                        };
                        let _ = insert_record(conn, &record);
                        records.push(record);
                    }
                    Err(e) => {
                        errors.push(format!("Move failed for {}: {}", file.path, e));
                    }
                }
            }

            _ => {}
        }
    }

    RenameResult { records, errors }
}