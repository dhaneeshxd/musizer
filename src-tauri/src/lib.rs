mod db;
mod duplicates;
mod renamer;
mod scanner;

use db::{generate_session_id, get_latest_session_id, get_recent_records, get_records_by_session, init_db, delete_all_history, RenameRecord};
use duplicates::detect_duplicates;
use renamer::process_files;
use scanner::{scan_folder, AudioFile};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Manager, State};


pub struct AppState {
    pub db_path: Mutex<String>,
}


fn get_db_path(state: &State<AppState>) -> String {
    state.db_path.lock().unwrap().clone()
}

#[derive(Serialize)]
pub struct ProcessResult {
    pub records: Vec<RenameRecord>,
    pub errors: Vec<String>,
    pub session_id: String,
}

#[tauri::command]
fn cmd_scan_folder(folder: String) -> Result<Vec<AudioFile>, String> {
    let mut files = scan_folder(&folder);
    detect_duplicates(&mut files);
    Ok(files)
}

#[tauri::command]
fn cmd_process_files(
    files: Vec<AudioFile>,
    source_dir: String,
    state: State<AppState>,
) -> Result<ProcessResult, String> {
    let db_path = get_db_path(&state);
    let conn = init_db(&db_path).map_err(|e| e.to_string())?;
    let session_id = generate_session_id();

    let result = process_files(&files, &source_dir, &session_id, &conn);

    Ok(ProcessResult {
        records: result.records,
        errors: result.errors,
        session_id,
    })
}

#[tauri::command]
fn cmd_get_recent_history(
    limit: i64,
    state: State<AppState>,
) -> Result<Vec<RenameRecord>, String> {
    let db_path = get_db_path(&state);
    let conn = init_db(&db_path).map_err(|e| e.to_string())?;
    get_recent_records(&conn, limit).map_err(|e| e.to_string())
}

#[tauri::command]
fn cmd_delete_all_history(state: State<AppState>) -> Result<usize, String> {
    let db_path = get_db_path(&state);
    let conn = init_db(&db_path).map_err(|e| e.to_string())?;
    delete_all_history(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn cmd_get_latest_session(state: State<AppState>) -> Result<Option<String>, String> {
    let db_path = get_db_path(&state);
    let conn = init_db(&db_path).map_err(|e| e.to_string())?;
    get_latest_session_id(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn cmd_get_session_records(
    session_id: String,
    state: State<AppState>,
) -> Result<Vec<RenameRecord>, String> {
    let db_path = get_db_path(&state);
    let conn = init_db(&db_path).map_err(|e| e.to_string())?;
    get_records_by_session(&conn, &session_id).map_err(|e| e.to_string())
}

/// Copy files from the latest session to a destination directory
#[tauri::command]
fn cmd_copy_recent_to_dest(
    destination: String,
    session_id: String,
    state: State<AppState>,
) -> Result<CopyResult, String> {
    let db_path = get_db_path(&state);
    let conn = init_db(&db_path).map_err(|e| e.to_string())?;
    let records = get_records_by_session(&conn, &session_id)
        .map_err(|e| e.to_string())?;

    let dest_path = PathBuf::from(&destination);
    if !dest_path.exists() {
        fs::create_dir_all(&dest_path).map_err(|e| e.to_string())?;
    }

    let mut copied = 0u32;
    let mut skipped = 0u32;
    let mut errors = Vec::new();

    for record in &records {
        // Skip unwanted statuses first


        let src = PathBuf::from(&record.new_path);

        if !src.exists() {
            errors.push(format!("Source not found: {}", record.new_path));
            continue;
        }

        let file_name = match src.file_name() {
            Some(name) => name,
            None => {
                errors.push(format!("Invalid file name: {}", record.new_path));
                continue;
            }
        };

        let target = dest_path.join(file_name);

        // Skip if already exists in destination
        if target.exists() {
            skipped += 1;
            continue;
        }

        match fs::copy(&src, &target) {
            Ok(_) => copied += 1,
            Err(e) => errors.push(format!("Copy failed for {:?}: {}", file_name, e)),
        }
    }

    Ok(CopyResult { copied, skipped, errors })
}

/// Update a single file's metadata fields (for inline editing in UI)
#[tauri::command]
fn cmd_update_file_metadata(mut file: AudioFile, field: String, value: String) -> Result<AudioFile, String> {
    match field.as_str() {
        "title" => file.title = value,
        "artist" => file.artist = value,
        "album" => file.album = value,
        _ => return Err(format!("Unknown field: {}", field)),
    }
    Ok(file)
}

#[derive(Serialize)]
pub struct CopyResult {
    pub copied: u32,
    pub skipped: u32,
    pub errors: Vec<String>,
}

// ── Plugin setup ──────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Cannot resolve app data dir");
            std::fs::create_dir_all(&app_dir).ok();
            let db_path = app_dir.join("musizer.db").to_string_lossy().to_string();
            app.manage(AppState {
                db_path: Mutex::new(db_path),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd_scan_folder,
            cmd_process_files,
            cmd_get_recent_history,
            cmd_get_latest_session,
            cmd_get_session_records,
            cmd_copy_recent_to_dest,
            cmd_update_file_metadata,
            cmd_delete_all_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}       