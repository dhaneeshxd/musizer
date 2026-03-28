use std::path::Path;
use lofty::file::AudioFile as LoftyAudioFile;
use lofty::prelude::*;
use lofty::probe::Probe;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "aac", "ogg", "m4a", "wav", "opus", "wma"];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioFile {
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub bitrate: u8,
    pub extension: String,
    pub file_stem: String,
    /// "primary" | "duplicate" | "unique"
    pub status: String,
}

pub fn scan_folder(folder: &str) -> Vec<AudioFile> {
    let mut results = Vec::new();

    for entry in WalkDir::new(folder)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e.to_lowercase(),
            None => continue,
        };

        if !AUDIO_EXTENSIONS.contains(&ext.as_str()) {
            continue;
        }

        // Skip files inside /duplicates subfolder (already processed)
        if path.components().any(|c| c.as_os_str() == "duplicates") {
            continue;
        }

        if let Some(audio) = extract_metadata(path, &ext) {
            results.push(audio);
        }
    }

    results
}

fn extract_metadata(path: &Path, ext: &str) -> Option<AudioFile> {
    let path_str = path.to_string_lossy().to_string();
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let tagged_file = match Probe::open(path).ok()?.read() {
        Ok(f) => f,
        Err(_) => return None,
    };

    let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());

    let (title, artist, album) = if let Some(tag) = tag {
        let title = tag
            .title()
            .map(|t| t.to_string())
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| file_stem.clone());

        let artist = tag
            .artist()
            .map(|a| a.to_string())
            .filter(|a| !a.trim().is_empty())
            .unwrap_or_else(|| "Unknown".to_string());

        let album = tag
            .album()
            .map(|a| a.to_string())
            .filter(|a| !a.trim().is_empty())
            .unwrap_or_else(|| "Unknown".to_string());

        (title, artist, album)
    } else {
        (file_stem.clone(), "Unknown".to_string(), "Unknown".to_string())
    };

    // Extract bitrate from audio properties
    let bitrate = tagged_file
        .properties()
        .bit_depth()
        .unwrap_or(0);

    Some(AudioFile {
        path: path_str,
        title,
        artist,
        album,
        bitrate,
        extension: ext.to_string(),
        file_stem,
        status: "unique".to_string(),
    })
}