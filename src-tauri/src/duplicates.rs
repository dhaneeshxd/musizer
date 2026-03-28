use std::collections::HashMap;
use crate::scanner::AudioFile;

/// Marks the highest-bitrate file as "primary", rest as "duplicate".
/// Files with no duplicates stay "unique".
pub fn detect_duplicates(files: &mut Vec<AudioFile>) {
    let mut key_map: HashMap<String, Vec<usize>> = HashMap::new();

    for (i, file) in files.iter().enumerate() {
        let album_clean = clean_album(&file.album).trim().to_lowercase();
        let title_clean = clean_title(&file.title, &album_clean).trim().to_lowercase();

        key_map.entry(title_clean).or_default().push(i);
    }

    for (_key, indices) in key_map.iter() {
        if indices.len() <= 1 {
            continue;
        }

        // Normalize artists for each file
        let artist_map: Vec<(usize, Vec<String>)> = indices
            .iter()
            .map(|&i| (i, normalize_artists(&files[i].artist)))
            .collect();

        // Find groups with overlapping artists
        let mut grouped: Vec<Vec<usize>> = Vec::new();

        for (idx, artists) in &artist_map {
            let mut found_group = false;

            for group in grouped.iter_mut() {
                if group.iter().any(|&g_idx| {
                    let g_artists = normalize_artists(&files[g_idx].artist);
                    artists.iter().any(|a| g_artists.contains(a))
                }) {
                    group.push(*idx);
                    found_group = true;
                    break;
                }
            }

            if !found_group {
                grouped.push(vec![*idx]);
            }
        }

        // process each artist-matching group
        for group in grouped {
            if group.len() <= 1 {
                continue;
            }

            let primary_idx = group
                .iter()
                .copied()
                .max_by_key(|&i| files[i].bitrate)
                .unwrap();

            for &idx in &group {
                if idx == primary_idx {
                    files[idx].status = "primary".to_string();
                } else {
                    files[idx].status = "duplicate".to_string();
                }
            }
        }
    }
}

fn normalize_artists(artist: &str) -> Vec<String> {
    artist
        .to_lowercase()
        .split(&[',', '&', ';'][..])
        .map(|s| s.trim().to_string())
        .collect()
}

fn clean_album(input: &str) -> &str {
    match input.find('(') {
        Some(idx) => &input[..idx],
        None => input,
    }
}

fn clean_title(input: &str, album: &str) -> String {
    let noise_tags = ["remix", "official", "hd", "lyrics", "video", "audio", "320kbps", "hq", "feat", "from", "original"];

    if let Some(start) = input.find('(') {
        if let Some(end) = input.find(')') {
            let inside = input[start + 1..end].to_lowercase();

            if noise_tags.iter().any(|tag| inside.contains(tag)) || album.to_lowercase().contains(&inside) {
                let cleaned = format!("{}{}", &input[..start], &input[end + 1..]);
                return cleaned.trim().to_string();
            }
        }
    }

    input.trim().to_string()
}