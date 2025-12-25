use serde::{Deserialize, Serialize};
use tauri::api::dialog::FileDialogBuilder;
use tauri::api::path::config_dir;
use std::fs;
use std::path::PathBuf;

use crate::download::{download_youtube, DownloadResult, check_dependencies, ensure_ytdlp, ensure_ffmpeg};

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
    config_dir()
        .map(|dir| dir.join("youtube-downloader").join("history.json"))
}

// Removed select_file - no longer needed for YouTube downloader

#[tauri::command]
pub async fn select_output_folder() -> Result<Option<String>, String> {
    use std::sync::mpsc;
    
    let (tx, rx) = mpsc::channel();
    let mut result: Option<String> = None;
    
    FileDialogBuilder::new()
        .pick_folder(move |folder_path| {
            if let Some(path) = folder_path {
                let path_str = path.to_string_lossy().to_string();
                let _ = tx.send(path_str);
            } else {
                let _ = tx.send(String::new());
            }
        });
    
    // Try to receive the result
    // The dialog callback should fire when user selects or cancels
    match rx.try_recv() {
        Ok(path) => {
            if !path.is_empty() {
                result = Some(path);
            }
        },
        Err(mpsc::TryRecvError::Empty) => {
            // Dialog might still be open, wait for it
            // In Tauri v1.5, pick_folder is blocking, so we need to wait
            match rx.recv() {
                Ok(path) => {
                    if !path.is_empty() {
                        result = Some(path);
                    }
                },
                Err(_) => {}
            }
        },
        Err(_) => {}
    }
    
    Ok(result)
}

#[tauri::command]
pub async fn download_from_youtube(
    url: String,
    output_folder: String,
    bitrate: u32,
    app_handle: tauri::AppHandle,
) -> Result<DownloadResult, String> {
    let result = download_youtube(&url, &output_folder, bitrate).await?;

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

    Ok(result)
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

/// Check if required dependencies (yt-dlp and ffmpeg) are installed
/// This should be called at app startup to ensure dependencies are available
#[tauri::command]
pub async fn check_required_dependencies() -> Result<crate::download::DependencyCheck, String> {
    Ok(check_dependencies().await)
}

/// Setup yt-dlp by downloading it if not available
/// This will automatically download yt-dlp to the app's data directory
#[tauri::command]
pub async fn setup_ytdlp() -> Result<String, String> {
    ensure_ytdlp().await
}

/// Setup FFmpeg by downloading it if not available
/// This will automatically download FFmpeg to the app's data directory
#[tauri::command]
pub async fn setup_ffmpeg() -> Result<String, String> {
    ensure_ffmpeg().await
}

