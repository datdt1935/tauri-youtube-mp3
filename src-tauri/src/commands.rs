use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::api::path::config_dir;

use crate::deps;
use crate::download::{
    download_playlist_with_progress, download_youtube, is_playlist_url, DownloadResult,
    PlaylistDownloadResult,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadHistory {
    pub url: String,
    pub title: Option<String>,
    pub output_path: String,
    pub bitrate: u32,
    pub timestamp: String,
    pub duration: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryData {
    pub downloads: Vec<DownloadHistory>,
}

impl HistoryData {
    fn new() -> Self {
        Self {
            downloads: Vec::new(),
        }
    }

    fn load() -> Self {
        if let Some(history_path) = get_history_path() {
            if let Ok(content) = fs::read_to_string(&history_path) {
                if let Ok(data) = serde_json::from_str::<HistoryData>(&content) {
                    return data;
                }
            }
        }
        Self::new()
    }

    fn save(&self) -> Result<(), String> {
        if let Some(history_path) = get_history_path() {
            if let Some(parent) = history_path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let content = serde_json::to_string_pretty(self)
                .map_err(|e| format!("Failed to serialize history: {}", e))?;
            fs::write(&history_path, content).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn add(&mut self, download: DownloadHistory) -> Result<(), String> {
        self.downloads.push(download);
        // Keep only last 100 downloads
        if self.downloads.len() > 100 {
            self.downloads.remove(0);
        }
        self.save()
    }
}

fn get_history_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("youtube-downloader").join("history.json"))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppPreferences {
    pub output_folder: Option<String>,
    pub bitrate: Option<u32>,
    pub last_url: Option<String>,
}

impl AppPreferences {
    fn new() -> Self {
        Self {
            output_folder: None,
            bitrate: None,
            last_url: None,
        }
    }

    fn load() -> Self {
        if let Some(prefs_path) = get_preferences_path() {
            if let Ok(content) = fs::read_to_string(&prefs_path) {
                if let Ok(data) = serde_json::from_str::<AppPreferences>(&content) {
                    return data;
                }
            }
        }
        Self::new()
    }

    fn save(&self) -> Result<(), String> {
        if let Some(prefs_path) = get_preferences_path() {
            if let Some(parent) = prefs_path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let content = serde_json::to_string_pretty(self)
                .map_err(|e| format!("Failed to serialize preferences: {}", e))?;
            fs::write(&prefs_path, content).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

fn get_preferences_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("youtube-downloader").join("preferences.json"))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DownloadResponse {
    Single(DownloadResult),
    Playlist(PlaylistDownloadResult),
}

#[tauri::command]
pub async fn download_from_youtube(
    url: String,
    output_folder: String,
    bitrate: u32,
    app_handle: tauri::AppHandle,
) -> Result<DownloadResponse, String> {
    // Check if URL is a playlist
    if is_playlist_url(&url) {
        let result =
            download_playlist_with_progress(&url, &output_folder, bitrate, app_handle.clone())
                .await?;

        // Save each video to history
        let mut history = HistoryData::load();
        for video in &result.downloaded_videos {
            let download = DownloadHistory {
                url: url.clone(),
                title: video.title.clone(),
                output_path: video.output_path.clone(),
                bitrate,
                timestamp: chrono::Utc::now().to_rfc3339(),
                duration: video.duration,
            };
            history.add(download).ok();
        }

        // Send notification
        let app_name = app_handle.package_info().name.clone();
        tauri::api::notification::Notification::new(&app_name)
            .title("Playlist Download Complete")
            .body(&format!(
                "Successfully downloaded {} videos from playlist",
                result.downloaded_videos.len()
            ))
            .show()
            .ok();

        Ok(DownloadResponse::Playlist(result))
    } else {
        let result = download_youtube(&url, &output_folder, bitrate, &app_handle).await?;

        // Save to history
        let mut history = HistoryData::load();
        let download = DownloadHistory {
            url: url.clone(),
            title: result.title.clone(),
            output_path: result.output_path.clone(),
            bitrate,
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration: result.duration,
        };
        history.add(download).ok();

        // Send notification
        let app_name = app_handle.package_info().name.clone();
        tauri::api::notification::Notification::new(&app_name)
            .title("Download Complete")
            .body("Successfully downloaded and converted to MP3")
            .show()
            .ok();

        Ok(DownloadResponse::Single(result))
    }
}

#[tauri::command]
pub async fn get_download_history() -> Result<Vec<DownloadHistory>, String> {
    let history = HistoryData::load();
    Ok(history.downloads)
}

#[tauri::command]
pub async fn clear_history() -> Result<(), String> {
    let history = HistoryData::new();
    history.save()
}

#[tauri::command]
pub async fn check_deps(app_handle: tauri::AppHandle) -> Result<deps::DepsCheckResult, String> {
    Ok(deps::check_deps(&app_handle))
}

#[tauri::command]
pub async fn clear_extracted_binaries(app_handle: tauri::AppHandle) -> Result<(), String> {
    use std::fs;

    let app_data_dir = app_handle
        .path_resolver()
        .app_data_dir()
        .ok_or("Failed to get app data directory")?;

    let bin_dir = app_data_dir.join("bin");

    if bin_dir.exists() {
        let entries =
            fs::read_dir(&bin_dir).map_err(|e| format!("Failed to read bin directory: {}", e))?;

        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    fs::remove_file(&path)
                        .map_err(|e| format!("Failed to remove {}: {}", path.display(), e))?;
                    eprintln!("[deps] Removed extracted binary: {}", path.display());
                }
            }
        }
    }

    Ok(())
}

/// Save the output folder path to preferences
#[tauri::command]
pub async fn save_output_folder(output_folder: String) -> Result<(), String> {
    let mut prefs = AppPreferences::load();
    prefs.output_folder = Some(output_folder);
    prefs.save()
}

/// Get the saved output folder path from preferences
#[tauri::command]
pub async fn get_output_folder() -> Result<Option<String>, String> {
    let prefs = AppPreferences::load();
    Ok(prefs.output_folder)
}

/// Save all preferences (output folder, bitrate, and last URL)
#[tauri::command]
pub async fn save_preferences(
    output_folder: Option<String>,
    bitrate: Option<u32>,
    last_url: Option<String>,
) -> Result<(), String> {
    let mut prefs = AppPreferences::load();
    if let Some(folder) = output_folder {
        prefs.output_folder = Some(folder);
    }
    if let Some(br) = bitrate {
        prefs.bitrate = Some(br);
    }
    if let Some(url) = last_url {
        prefs.last_url = Some(url);
    }
    prefs.save()
}

/// Get all saved preferences
#[tauri::command]
pub async fn get_preferences() -> Result<AppPreferences, String> {
    Ok(AppPreferences::load())
}
