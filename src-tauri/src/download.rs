use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashSet;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use tauri::api::path::config_dir;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadResult {
    pub output_path: String,
    pub title: Option<String>,
    pub duration: Option<f64>,
    pub file_size: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistDownloadResult {
    pub output_folder: String,
    pub total_videos: usize,
    pub downloaded_videos: Vec<DownloadResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadProgress {
    pub overall_progress: f64,        // Overall progress percentage (0-100)
    pub current_song: Option<usize>,  // Current song number (for playlists)
    pub total_songs: Option<usize>,   // Total songs (for playlists)
    pub song_progress: f64,           // Current song download progress (0-100)
    pub status: String,               // Current status: "downloading", "converting", etc.
    pub current_title: Option<String>, // Current song title being processed
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyCheck {
    pub ytdlp_installed: bool,
    pub ffmpeg_installed: bool,
    pub ytdlp_command: String,
    pub ffmpeg_command: String,
    pub ytdlp_error: Option<String>,
    pub ffmpeg_error: Option<String>,
    pub installation_instructions: String,
}

/// Get the directory where yt-dlp binary should be stored
fn get_ytdlp_dir() -> Option<PathBuf> {
    config_dir()
        .map(|dir| dir.join("youtube-downloader").join("bin"))
}

/// Get the path to the local yt-dlp binary
fn get_local_ytdlp_path() -> Option<PathBuf> {
    get_ytdlp_dir().map(|dir| {
        if cfg!(target_os = "windows") {
            dir.join("yt-dlp.exe")
        } else {
            dir.join("yt-dlp")
        }
    })
}

/// Get the path to the local ffmpeg binary
fn get_local_ffmpeg_path() -> Option<PathBuf> {
    get_ytdlp_dir().map(|dir| {
        if cfg!(target_os = "windows") {
            dir.join("ffmpeg.exe")
        } else {
            dir.join("ffmpeg")
        }
    })
}

/// Check if a command exists and is executable
/// Returns (is_available, command_name, is_local)
async fn check_command(command_names: &[&str], local_path: Option<&Path>) -> (bool, String, bool) {
    // First check local version if available
    if let Some(local) = local_path {
        if local.exists() {
            let result = Command::new(local)
                .arg("--version")
                .output()
                .await;
            
            if result.is_ok() && result.as_ref().unwrap().status.success() {
                return (true, local.to_string_lossy().to_string(), true);
            }
        }
    }
    
    // Then check system PATH
    for cmd in command_names {
        let result = Command::new(cmd)
            .arg("--version")
            .output()
            .await;
        
        if result.is_ok() && result.as_ref().unwrap().status.success() {
            return (true, cmd.to_string(), false);
        }
    }
    (false, command_names[0].to_string(), false)
}

/// Get installation instructions for the current OS
fn get_installation_instructions() -> String {
    if cfg!(target_os = "windows") {
        "Windows Installation:\n\n\
        yt-dlp:\n\
        - Download from: https://github.com/yt-dlp/yt-dlp/releases/latest\n\
        - Or use: winget install yt-dlp\n\
        - Or use: pip install yt-dlp\n\n\
        FFmpeg:\n\
        - Download from: https://ffmpeg.org/download.html\n\
        - Or use: winget install ffmpeg\n\
        - Or use: choco install ffmpeg\n\n\
        IMPORTANT: After installation, RESTART the application for PATH changes to take effect!"
    } else if cfg!(target_os = "macos") {
        "macOS Installation:\n\n\
        yt-dlp:\n\
        - brew install yt-dlp\n\
        - Or: pip install yt-dlp\n\n\
        FFmpeg:\n\
        - brew install ffmpeg"
    } else {
        "Linux Installation:\n\n\
        yt-dlp:\n\
        - pip install yt-dlp\n\
        - Or: sudo apt install yt-dlp (Debian/Ubuntu)\n\
        - Or: sudo yum install yt-dlp (RHEL/CentOS)\n\n\
        FFmpeg:\n\
        - sudo apt install ffmpeg (Debian/Ubuntu)\n\
        - Or: sudo yum install ffmpeg (RHEL/CentOS)\n\
        - Or: sudo pacman -S ffmpeg (Arch Linux)"
    }
    .to_string()
}

/// Check if yt-dlp is installed and return the command to use
pub async fn check_ytdlp() -> (bool, String) {
    let local_path = get_local_ytdlp_path();
    let (installed, cmd, _) = if cfg!(target_os = "windows") {
        check_command(&["yt-dlp.exe", "yt-dlp"], local_path.as_deref()).await
    } else {
        check_command(&["yt-dlp"], local_path.as_deref()).await
    };
    (installed, cmd)
}

/// Check if ffmpeg is installed and return the command to use
pub async fn check_ffmpeg() -> (bool, String) {
    let local_path = get_local_ffmpeg_path();
    let (installed, cmd, _) = if cfg!(target_os = "windows") {
        check_command(&["ffmpeg.exe", "ffmpeg"], local_path.as_deref()).await
    } else {
        check_command(&["ffmpeg"], local_path.as_deref()).await
    };
    (installed, cmd)
}

/// Download yt-dlp binary from GitHub releases
pub async fn download_ytdlp() -> Result<String, String> {
    let bin_dir = get_ytdlp_dir().ok_or("Failed to get app data directory")?;
    let ytdlp_path = get_local_ytdlp_path().ok_or("Failed to get yt-dlp path")?;
    
    // Create bin directory if it doesn't exist
    fs::create_dir_all(&bin_dir)
        .map_err(|e| format!("Failed to create bin directory: {}", e))?;
    
    // Check if already downloaded
    if ytdlp_path.exists() {
        // Verify it works
        let result = Command::new(&ytdlp_path)
            .arg("--version")
            .output()
            .await;
        
        if result.is_ok() && result.as_ref().unwrap().status.success() {
            return Ok(ytdlp_path.to_string_lossy().to_string());
        }
        // If it doesn't work, delete and re-download
        fs::remove_file(&ytdlp_path).ok();
    }
    
    // Determine download URL based on platform
    let download_url = if cfg!(target_os = "windows") {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
        } else {
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
        }
    } else {
        // Linux
        if cfg!(target_arch = "x86_64") {
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux"
        } else if cfg!(target_arch = "aarch64") {
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux_aarch64"
        } else {
            return Err("Unsupported Linux architecture".to_string());
        }
    };
    
    // Download the binary
    let client = reqwest::Client::new();
    let response = client
        .get(download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download yt-dlp: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to download yt-dlp: HTTP {}", response.status()));
    }
    
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download: {}", e))?;
    
    // Write to file
    fs::write(&ytdlp_path, &bytes)
        .map_err(|e| format!("Failed to write yt-dlp binary: {}", e))?;
    
    // Make executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&ytdlp_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&ytdlp_path, perms)
            .map_err(|e| format!("Failed to set executable permissions: {}", e))?;
    }
    
    // Verify the download
    let result = Command::new(&ytdlp_path)
        .arg("--version")
        .output()
        .await;
    
    if result.is_err() || !result.as_ref().unwrap().status.success() {
        fs::remove_file(&ytdlp_path).ok();
        return Err("Downloaded yt-dlp binary is not working".to_string());
    }
    
    Ok(ytdlp_path.to_string_lossy().to_string())
}

/// Ensure yt-dlp is available, downloading it if necessary
pub async fn ensure_ytdlp() -> Result<String, String> {
    // First check if it's already available
    let (installed, cmd) = check_ytdlp().await;
    if installed {
        return Ok(cmd);
    }
    
    // If not available, try to download it
    download_ytdlp().await
}

/// Download FFmpeg binary from official sources
pub async fn download_ffmpeg() -> Result<String, String> {
    let bin_dir = get_ytdlp_dir().ok_or("Failed to get app data directory")?;
    let ffmpeg_path = get_local_ffmpeg_path().ok_or("Failed to get ffmpeg path")?;
    
    // Create bin directory if it doesn't exist
    fs::create_dir_all(&bin_dir)
        .map_err(|e| format!("Failed to create bin directory: {}", e))?;
    
    // Check if already downloaded
    if ffmpeg_path.exists() {
        // Verify it works
        let result = Command::new(&ffmpeg_path)
            .arg("-version")
            .output()
            .await;
        
        if result.is_ok() && result.as_ref().unwrap().status.success() {
            return Ok(ffmpeg_path.to_string_lossy().to_string());
        }
        // If it doesn't work, delete and re-download
        fs::remove_file(&ffmpeg_path).ok();
    }
    
    // Determine download URL based on platform
    // Using static builds from various sources
    let (download_url, is_zip) = if cfg!(target_os = "windows") {
        // Windows: Use gyan.dev static builds (most reliable)
        ("https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip", true)
    } else if cfg!(target_os = "macos") {
        // macOS: Use static build from evermeet.cx or GitHub
        if cfg!(target_arch = "aarch64") {
            ("https://evermeet.cx/ffmpeg/ffmpeg-7.0.zip", true)
        } else {
            ("https://evermeet.cx/ffmpeg/ffmpeg-7.0.zip", true)
        }
    } else {
        // Linux: Use static build from GitHub
        if cfg!(target_arch = "x86_64") {
            ("https://github.com/eugeneware/ffmpeg-static/releases/download/b6.0.1/ffmpeg-linux-x64", false)
        } else if cfg!(target_arch = "aarch64") {
            ("https://github.com/eugeneware/ffmpeg-static/releases/download/b6.0.1/ffmpeg-linux-arm64", false)
        } else {
            return Err("Unsupported Linux architecture".to_string());
        }
    };
    
    // Download the binary
    let client = reqwest::Client::new();
    let response = client
        .get(download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download FFmpeg: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to download FFmpeg: HTTP {}", response.status()));
    }
    
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download: {}", e))?;
    
    if is_zip {
        // Extract from ZIP (Windows and macOS)
        use std::io::Cursor;
        let cursor = Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;
        
        // Find ffmpeg executable in the archive
        let mut found = false;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| format!("Failed to read file in archive: {}", e))?;
            
            let file_path = file.name();
            let file_name = std::path::Path::new(file_path)
                .file_name()
                .and_then(|n| n.to_str());
            
            // Look for ffmpeg.exe (Windows) or ffmpeg (macOS/Linux)
            if file_name == Some("ffmpeg.exe") || file_name == Some("ffmpeg") {
                let mut outfile = fs::File::create(&ffmpeg_path)
                    .map_err(|e| format!("Failed to create ffmpeg file: {}", e))?;
                std::io::copy(&mut file, &mut outfile)
                    .map_err(|e| format!("Failed to extract ffmpeg: {}", e))?;
                found = true;
                break;
            }
        }
        
        if !found {
            return Err("Could not find ffmpeg executable in downloaded archive".to_string());
        }
    } else {
        // Direct binary (Linux)
        fs::write(&ffmpeg_path, &bytes)
            .map_err(|e| format!("Failed to write ffmpeg binary: {}", e))?;
    }
    
    // Make executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&ffmpeg_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&ffmpeg_path, perms)
            .map_err(|e| format!("Failed to set executable permissions: {}", e))?;
    }
    
    // Verify the download
    let result = Command::new(&ffmpeg_path)
        .arg("-version")
        .output()
        .await;
    
    if result.is_err() || !result.as_ref().unwrap().status.success() {
        fs::remove_file(&ffmpeg_path).ok();
        return Err("Downloaded FFmpeg binary is not working".to_string());
    }
    
    Ok(ffmpeg_path.to_string_lossy().to_string())
}

/// Ensure FFmpeg is available, downloading it if necessary
pub async fn ensure_ffmpeg() -> Result<String, String> {
    // First check if it's already available
    let (installed, cmd) = check_ffmpeg().await;
    if installed {
        return Ok(cmd);
    }
    
    // If not available, try to download it
    download_ffmpeg().await
}

/// Check all dependencies and return a detailed status
pub async fn check_dependencies() -> DependencyCheck {
    let (ytdlp_installed, ytdlp_cmd) = check_ytdlp().await;
    let (ffmpeg_installed, ffmpeg_cmd) = check_ffmpeg().await;
    
    let mut ytdlp_error = None;
    let mut ffmpeg_error = None;
    
    if !ytdlp_installed {
        ytdlp_error = Some(format!(
            "yt-dlp is not installed or not found in PATH.\n\nVisit: https://github.com/yt-dlp/yt-dlp"
        ));
    }
    
    if !ffmpeg_installed {
        ffmpeg_error = Some(format!(
            "FFmpeg is not installed or not found in PATH.\n\nVisit: https://ffmpeg.org/download.html"
        ));
    }
    
    DependencyCheck {
        ytdlp_installed,
        ffmpeg_installed,
        ytdlp_command: ytdlp_cmd,
        ffmpeg_command: ffmpeg_cmd,
        ytdlp_error,
        ffmpeg_error,
        installation_instructions: get_installation_instructions(),
    }
}

pub async fn download_youtube(
    url: &str,
    output_folder: &str,
    bitrate: u32,
) -> Result<DownloadResult, String> {
    // Validate YouTube URL
    if !is_youtube_url(url) {
        return Err("Invalid YouTube URL. Please provide a valid YouTube video URL.".to_string());
    }

    // Ensure yt-dlp is available (download if necessary)
    let ytdlp_cmd = match ensure_ytdlp().await {
        Ok(cmd) => cmd,
        Err(e) => {
            return Err(format!(
                "Failed to setup yt-dlp: {}\n\nPlease install yt-dlp manually:\n{}\n\nOr visit: https://github.com/yt-dlp/yt-dlp",
                e,
                get_installation_instructions()
            ));
        }
    };

    // Ensure FFmpeg is available (download if necessary)
    let _ffmpeg_cmd = match ensure_ffmpeg().await {
        Ok(cmd) => cmd,
        Err(e) => {
            return Err(format!(
                "Failed to setup FFmpeg: {}\n\nPlease install FFmpeg manually:\n{}\n\nOr visit: https://ffmpeg.org/download.html",
                e,
                get_installation_instructions()
            ));
        }
    };

    // Get video info first to get the title
    // Use the ytdlp_cmd we determined during the check above
    let info_output = Command::new(&ytdlp_cmd)
        .arg("--dump-json")
        .arg("--no-playlist")
        .arg(url)
        .output()
        .await
        .map_err(|e| format!("Failed to get video info: {}", e))?;

    if !info_output.status.success() {
        let error = String::from_utf8_lossy(&info_output.stderr);
        return Err(format!("Failed to get video info: {}", error));
    }

    let video_info: serde_json::Value = serde_json::from_slice(&info_output.stdout)
        .map_err(|e| format!("Failed to parse video info: {}", e))?;

    let title = video_info["title"]
        .as_str()
        .map(|s| sanitize_filename(s));

    let duration = video_info["duration"]
        .as_f64();

    // Determine the expected output path
    let output_path = if let Some(ref t) = title {
        Path::new(output_folder).join(format!("{}.mp3", t))
    } else {
        // Fallback: use video ID or default name
        let video_id = video_info["id"]
            .as_str()
            .unwrap_or("video");
        Path::new(output_folder).join(format!("{}.mp3", video_id))
    };

    // Check if file already exists before downloading
    if output_path.exists() {
        // File already exists, skip download and return existing file info
        let file_size = std::fs::metadata(&output_path)
            .ok()
            .map(|m| m.len());
        
        return Ok(DownloadResult {
            output_path: output_path.to_string_lossy().to_string(),
            title,
            duration,
            file_size,
        });
    }

    // Generate output filename template for yt-dlp
    // yt-dlp will use the title and add .mp3 extension
    let output_template = format!("{}/%(title)s.%(ext)s", output_folder);

    // Download and convert to MP3 using yt-dlp
    // yt-dlp can extract audio and convert to MP3 in one step
    let download_output = Command::new(&ytdlp_cmd)
        .arg("-x") // Extract audio
        .arg("--audio-format")
        .arg("mp3")
        .arg("--audio-quality")
        .arg(format!("{}K", bitrate))
        .arg("-o")
        .arg(&output_template)
        .arg("--no-playlist")
        .arg(url)
        .output()
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    if !download_output.status.success() {
        let error = String::from_utf8_lossy(&download_output.stderr);
        return Err(format!("Download failed: {}", error));
    }

    // Get file size
    let file_size = std::fs::metadata(&output_path)
        .ok()
        .map(|m| m.len());

    Ok(DownloadResult {
        output_path: output_path.to_string_lossy().to_string(),
        title,
        duration,
        file_size,
    })
}

/// Download all videos from a YouTube playlist
pub async fn download_playlist(
    url: &str,
    output_folder: &str,
    bitrate: u32,
) -> Result<PlaylistDownloadResult, String> {
    // Validate YouTube URL
    if !is_youtube_url(url) {
        return Err("Invalid YouTube URL. Please provide a valid YouTube playlist URL.".to_string());
    }

    if !is_playlist_url(url) {
        return Err("URL does not appear to be a playlist URL.".to_string());
    }

    // Ensure yt-dlp is available (download if necessary)
    let ytdlp_cmd = match ensure_ytdlp().await {
        Ok(cmd) => cmd,
        Err(e) => {
            return Err(format!(
                "Failed to setup yt-dlp: {}\n\nPlease install yt-dlp manually:\n{}\n\nOr visit: https://github.com/yt-dlp/yt-dlp",
                e,
                get_installation_instructions()
            ));
        }
    };

    // Ensure FFmpeg is available (download if necessary)
    let _ffmpeg_cmd = match ensure_ffmpeg().await {
        Ok(cmd) => cmd,
        Err(e) => {
            return Err(format!(
                "Failed to setup FFmpeg: {}\n\nPlease install FFmpeg manually:\n{}\n\nOr visit: https://ffmpeg.org/download.html",
                e,
                get_installation_instructions()
            ));
        }
    };

    // Get playlist info to determine the number of videos
    let info_output = Command::new(&ytdlp_cmd)
        .arg("--dump-json")
        .arg("--flat-playlist")
        .arg(url)
        .output()
        .await
        .map_err(|e| format!("Failed to get playlist info: {}", e))?;

    if !info_output.status.success() {
        let error = String::from_utf8_lossy(&info_output.stderr);
        return Err(format!("Failed to get playlist info: {}", error));
    }

    // Parse playlist entries (one JSON object per line)
    let output_str = String::from_utf8_lossy(&info_output.stdout);
    let entries: Vec<serde_json::Value> = output_str
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    let total_videos = entries.len();
    
    if total_videos == 0 {
        return Err("Playlist appears to be empty or could not be accessed.".to_string());
    }

    // Capture existing files before download to identify newly downloaded files
    let existing_files: HashSet<String> = if let Ok(entries) = std::fs::read_dir(output_folder) {
        entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("mp3"))
            .filter_map(|e| e.path().to_string_lossy().to_string().into())
            .collect()
    } else {
        HashSet::new()
    };

    // Generate output filename template for yt-dlp
    // All videos will be downloaded to the same folder
    let output_template = format!("{}/%(title)s.%(ext)s", output_folder);

    // Download all videos in the playlist
    // Use --yes-playlist to explicitly allow playlist downloads
    // Use --no-overwrites to skip files that already exist
    let download_output = Command::new(&ytdlp_cmd)
        .arg("-x") // Extract audio
        .arg("--audio-format")
        .arg("mp3")
        .arg("--audio-quality")
        .arg(format!("{}K", bitrate))
        .arg("-o")
        .arg(&output_template)
        .arg("--yes-playlist") // Explicitly allow playlist downloads
        .arg("--no-overwrites") // Skip files that already exist
        .arg(url)
        .output()
        .await
        .map_err(|e| format!("Playlist download failed: {}", e))?;

    if !download_output.status.success() {
        let error = String::from_utf8_lossy(&download_output.stderr);
        return Err(format!("Playlist download failed: {}", error));
    }

    // Collect only newly downloaded files from the output folder
    let mut downloaded_videos = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(output_folder) {
        let mp3_files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().and_then(|s| s.to_str()) == Some("mp3")
            })
            .collect();

        // For each file, check if it's new (wasn't there before download)
        for entry in mp3_files {
            let path = entry.path();
            let path_str = path.to_string_lossy().to_string();
            
            // Only include files that weren't there before the download
            if !existing_files.contains(&path_str) {
                if let Ok(metadata) = std::fs::metadata(&path) {
                    let file_size = Some(metadata.len());
                    let file_name = path.file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string());
                    
                    downloaded_videos.push(DownloadResult {
                        output_path: path_str,
                        title: file_name,
                        duration: None, // We don't parse duration for playlist items
                        file_size,
                    });
                }
            }
        }
    }

    Ok(PlaylistDownloadResult {
        output_folder: output_folder.to_string(),
        total_videos,
        downloaded_videos,
    })
}

/// Helper function to parse yt-dlp progress lines and emit progress events
fn process_progress_line(
    line: &str,
    current_song: &mut usize,
    song_progress: &mut f64,
    status: &mut String,
    current_title: &mut Option<String>,
    completed_songs: &mut usize,
    total_videos: usize,
    app_handle: &AppHandle,
) {
    // Pattern 1: [download] XX.X% of YYY at ZZZ ETA MM:SS
    // Pattern 2: [download] 100% of YYY
    // Pattern 3: [download]   XX.X% of ... (with spaces)
    if line.contains("[download]") {
        // Try to extract percentage - look for number followed by %
        // Use regex-like approach: find % and work backwards
        if let Some(percent_pos) = line.find('%') {
            // Find the start of the number
            let mut num_start = percent_pos;
            let mut found_digit = false;
            
            // Work backwards to find the start of the number
            while num_start > 0 {
                let ch = line.chars().nth(num_start - 1).unwrap_or(' ');
                if ch.is_ascii_digit() || ch == '.' {
                    found_digit = true;
                    num_start -= 1;
                } else if found_digit {
                    // We've found the start of the number
                    break;
                } else {
                    num_start -= 1;
                }
            }
            
            if found_digit && num_start < percent_pos {
                let percent_str = &line[num_start..percent_pos].trim();
                if let Ok(percent) = percent_str.parse::<f64>() {
                    let new_progress = percent.min(100.0).max(0.0);
                    
                    // Only update if progress actually changed (avoid spam)
                    if (new_progress - *song_progress).abs() > 0.1 || *song_progress == 0.0 {
                        *song_progress = new_progress;
                        *status = "Downloading...".to_string();
                        
                        let overall_progress = if total_videos > 0 {
                            ((*completed_songs as f64 + *song_progress / 100.0) / total_videos as f64) * 100.0
                        } else {
                            *song_progress
                        };

                        let progress = DownloadProgress {
                            overall_progress,
                            current_song: Some(*current_song + 1),
                            total_songs: Some(total_videos),
                            song_progress: *song_progress,
                            status: status.clone(),
                            current_title: current_title.clone(),
                        };
                        app_handle.emit_all("download-progress", progress).ok();
                    }
                }
            }
            
            // Check if download is complete (100%)
            if line.contains("100%") || (*song_progress >= 99.9 && line.contains("[download]")) {
                *song_progress = 100.0;
            }
        }
    }
    // Pattern: [ExtractAudio] Destination: ...
    else if line.contains("[ExtractAudio]") {
        *status = "Extracting audio...".to_string();
        *song_progress = 90.0;
        
        let overall_progress = if total_videos > 0 {
            ((*completed_songs as f64 + 0.90) / total_videos as f64) * 100.0
        } else {
            90.0
        };

        let progress = DownloadProgress {
            overall_progress,
            current_song: Some(*current_song + 1),
            total_songs: Some(total_videos),
            song_progress: 90.0,
            status: status.clone(),
            current_title: current_title.clone(),
        };
        app_handle.emit_all("download-progress", progress).ok();
    }
    // Pattern: [Merger] Merging formats into ...
    else if line.contains("[Merger]") || line.contains("Merging formats") {
        *status = "Converting to MP3...".to_string();
        *song_progress = 95.0;
        
        let overall_progress = if total_videos > 0 {
            ((*completed_songs as f64 + 0.95) / total_videos as f64) * 100.0
        } else {
            95.0
        };

        let progress = DownloadProgress {
            overall_progress,
            current_song: Some(*current_song + 1),
            total_songs: Some(total_videos),
            song_progress: 95.0,
            status: status.clone(),
            current_title: current_title.clone(),
        };
        app_handle.emit_all("download-progress", progress).ok();
    }
    // Pattern: [youtube] Extracting URL: ... or [youtube] Video ID: ...
    // This indicates a new video is starting
    else if line.contains("[youtube]") {
        if line.contains("Extracting URL") || line.contains("Video ID") || line.contains("Downloading") {
            // Check if we're actually starting a new song (not just processing the same one)
            // If previous song was complete, mark it as done
            if *song_progress >= 99.0 && *current_song > 0 {
                *completed_songs = (*completed_songs + 1).min(total_videos);
            }
            
            // Only increment if we're actually on a new song
            // We detect this by checking if progress was near completion or if it's the first song
            if *song_progress >= 99.0 || *current_song == 0 {
                *current_song += 1;
            }
            
            // Reset progress for new song
            if *song_progress >= 99.0 {
                *song_progress = 0.0;
                *status = "Preparing download...".to_string();
                *current_title = None; // Reset title for new song
                
                let overall_progress = if total_videos > 0 {
                    (*completed_songs as f64 / total_videos as f64) * 100.0
                } else {
                    0.0
                };

                let progress = DownloadProgress {
                    overall_progress,
                    current_song: Some(*current_song),
                    total_songs: Some(total_videos),
                    song_progress: 0.0,
                    status: status.clone(),
                    current_title: None,
                };
                app_handle.emit_all("download-progress", progress).ok();
            }
        }
    }
    // Pattern: [download] Destination: ... (indicates file completed)
    // Pattern: [download] 100% of ... (download complete)
    else if line.contains("[download]") && (line.contains("Destination:") || (line.contains("100%") && *song_progress < 50.0)) {
        // Song download completed
        if *song_progress < 100.0 {
            *song_progress = 100.0;
            *completed_songs += 1;
            *status = "Download complete".to_string();
            
            let overall_progress = if total_videos > 0 {
                (*completed_songs as f64 / total_videos as f64) * 100.0
            } else {
                100.0
            };

            let progress = DownloadProgress {
                overall_progress,
                current_song: Some(*current_song + 1),
                total_songs: Some(total_videos),
                song_progress: 100.0,
                status: status.clone(),
                current_title: current_title.clone(),
            };
            app_handle.emit_all("download-progress", progress).ok();
        }
    }
    // Pattern: Downloading playlist: ... or [playlist] ...
    else if (line.contains("Downloading playlist") || line.contains("[playlist]")) && line.contains("Downloading item") {
        // Extract item number if available
        if let Some(item_start) = line.find("item") {
            let rest = &line[item_start + 4..];
            if let Some(num_end) = rest.find(' ') {
                if let Ok(item_num) = rest[..num_end].trim().parse::<usize>() {
                    *current_song = item_num.saturating_sub(1);
                    *song_progress = 0.0;
                    *status = "Starting download...".to_string();
                    
                    let overall_progress = if total_videos > 0 {
                        (*completed_songs as f64 / total_videos as f64) * 100.0
                    } else {
                        0.0
                    };

                    let progress = DownloadProgress {
                        overall_progress,
                        current_song: Some(*current_song + 1),
                        total_songs: Some(total_videos),
                        song_progress: 0.0,
                        status: status.clone(),
                        current_title: current_title.clone(),
                    };
                    app_handle.emit_all("download-progress", progress).ok();
                }
            }
        }
    }
    // Try to extract title from various patterns
    // Pattern: [youtube] TITLE or title: TITLE
    if current_title.is_none() || current_title.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
        // Look for title patterns in the line
        if let Some(title_marker) = line.to_lowercase().find("title") {
            let after_title = &line[title_marker + 5..];
            if let Some(colon) = after_title.find(':') {
                let potential_title = after_title[colon + 1..].trim();
                if !potential_title.is_empty() && potential_title.len() < 200 {
                    *current_title = Some(potential_title.to_string());
                    
                    // Emit progress with new title
                    let overall_progress = if total_videos > 0 {
                        ((*completed_songs as f64 + *song_progress / 100.0) / total_videos as f64) * 100.0
                    } else {
                        *song_progress
                    };

                    let progress = DownloadProgress {
                        overall_progress,
                        current_song: Some(*current_song + 1),
                        total_songs: Some(total_videos),
                        song_progress: *song_progress,
                        status: status.clone(),
                        current_title: current_title.clone(),
                    };
                    app_handle.emit_all("download-progress", progress).ok();
                }
            }
        }
        // Also try to extract from [youtube] lines that might contain the title
        else if line.contains("[youtube]") && line.len() > 20 {
            // Sometimes yt-dlp outputs: [youtube] VideoTitle
            let parts: Vec<&str> = line.splitn(2, ']').collect();
            if parts.len() == 2 {
                let potential_title = parts[1].trim();
                if !potential_title.is_empty() && potential_title.len() < 200 && !potential_title.contains("http") {
                    *current_title = Some(potential_title.to_string());
                }
            }
        }
    }
}

/// Download all videos from a YouTube playlist with progress tracking
pub async fn download_playlist_with_progress(
    url: &str,
    output_folder: &str,
    bitrate: u32,
    app_handle: AppHandle,
) -> Result<PlaylistDownloadResult, String> {
    // Validate YouTube URL
    if !is_youtube_url(url) {
        return Err("Invalid YouTube URL. Please provide a valid YouTube playlist URL.".to_string());
    }

    if !is_playlist_url(url) {
        return Err("URL does not appear to be a playlist URL.".to_string());
    }

    // Ensure yt-dlp is available (download if necessary)
    let ytdlp_cmd = match ensure_ytdlp().await {
        Ok(cmd) => cmd,
        Err(e) => {
            return Err(format!(
                "Failed to setup yt-dlp: {}\n\nPlease install yt-dlp manually:\n{}\n\nOr visit: https://github.com/yt-dlp/yt-dlp",
                e,
                get_installation_instructions()
            ));
        }
    };

    // Ensure FFmpeg is available (download if necessary)
    let _ffmpeg_cmd = match ensure_ffmpeg().await {
        Ok(cmd) => cmd,
        Err(e) => {
            return Err(format!(
                "Failed to setup FFmpeg: {}\n\nPlease install FFmpeg manually:\n{}\n\nOr visit: https://ffmpeg.org/download.html",
                e,
                get_installation_instructions()
            ));
        }
    };

    // Get playlist info to determine the number of videos
    let info_output = Command::new(&ytdlp_cmd)
        .arg("--dump-json")
        .arg("--flat-playlist")
        .arg(url)
        .output()
        .await
        .map_err(|e| format!("Failed to get playlist info: {}", e))?;

    if !info_output.status.success() {
        let error = String::from_utf8_lossy(&info_output.stderr);
        return Err(format!("Failed to get playlist info: {}", error));
    }

    // Parse playlist entries (one JSON object per line) and extract video URLs
    let output_str = String::from_utf8_lossy(&info_output.stdout);
    let entries: Vec<serde_json::Value> = output_str
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    let total_videos = entries.len();
    
    if total_videos == 0 {
        return Err("Playlist appears to be empty or could not be accessed.".to_string());
    }

    // Extract video IDs and URLs from entries
    let mut video_urls = Vec::new();
    for entry in &entries {
        if let Some(id) = entry.get("id").and_then(|v| v.as_str()) {
            // Construct video URL from ID
            let video_url = format!("https://www.youtube.com/watch?v={}", id);
            video_urls.push(video_url);
        } else if let Some(url) = entry.get("url").and_then(|v| v.as_str()) {
            video_urls.push(url.to_string());
        }
    }

    // Capture existing files before download
    let existing_files: HashSet<String> = if let Ok(entries) = std::fs::read_dir(output_folder) {
        entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("mp3"))
            .filter_map(|e| e.path().to_string_lossy().to_string().into())
            .collect()
    } else {
        HashSet::new()
    };

    let mut downloaded_videos = Vec::new();

    // Download each video one by one with progress tracking
    for (index, video_url) in video_urls.iter().enumerate() {
        let current_song_num = index + 1;
        
        // Emit progress: starting new song
        let start_progress = DownloadProgress {
            overall_progress: (index as f64 / total_videos as f64) * 100.0,
            current_song: Some(current_song_num),
            total_songs: Some(total_videos),
            song_progress: 0.0,
            status: "Preparing download...".to_string(),
            current_title: None,
        };
        app_handle.emit_all("download-progress", start_progress).ok();

        // Get video info to extract title
        let info_output = Command::new(&ytdlp_cmd)
            .arg("--dump-json")
            .arg("--no-playlist")
            .arg(video_url)
            .output()
            .await;

        let mut current_title: Option<String> = None;
        if let Ok(info) = info_output {
            if info.status.success() {
                if let Ok(video_info) = serde_json::from_slice::<serde_json::Value>(&info.stdout) {
                    if let Some(title) = video_info.get("title").and_then(|v| v.as_str()) {
                        current_title = Some(sanitize_filename(title));
                        
                        // Emit progress with title
                        let title_progress = DownloadProgress {
                            overall_progress: (index as f64 / total_videos as f64) * 100.0,
                            current_song: Some(current_song_num),
                            total_songs: Some(total_videos),
                            song_progress: 0.0,
                            status: "Starting download...".to_string(),
                            current_title: current_title.clone(),
                        };
                        app_handle.emit_all("download-progress", title_progress).ok();
                    }
                }
            }
        }

        // Check if file already exists
        let expected_path = if let Some(ref title) = current_title {
            Path::new(output_folder).join(format!("{}.mp3", title))
        } else {
            // Fallback: use video ID
            if let Some(id) = video_url.split("v=").nth(1).and_then(|s| s.split('&').next()) {
                Path::new(output_folder).join(format!("{}.mp3", id))
            } else {
                Path::new(output_folder).join(format!("video_{}.mp3", current_song_num))
            }
        };

        if expected_path.exists() {
            // File already exists, skip
            let file_size = std::fs::metadata(&expected_path)
                .ok()
                .map(|m| m.len());
            
            downloaded_videos.push(DownloadResult {
                output_path: expected_path.to_string_lossy().to_string(),
                title: current_title.clone(),
                duration: None,
                file_size,
            });

            // Emit progress: song skipped (already exists)
            let skip_progress = DownloadProgress {
                overall_progress: ((index + 1) as f64 / total_videos as f64) * 100.0,
                current_song: Some(current_song_num),
                total_songs: Some(total_videos),
                song_progress: 100.0,
                status: "Already exists, skipping...".to_string(),
                current_title: current_title.clone(),
            };
            app_handle.emit_all("download-progress", skip_progress).ok();
            continue;
        }

        // Download this video with progress tracking
        let output_template = format!("{}/%(title)s.%(ext)s", output_folder);
        
        let mut child = Command::new(&ytdlp_cmd)
            .arg("-x")
            .arg("--audio-format")
            .arg("mp3")
            .arg("--audio-quality")
            .arg(format!("{}K", bitrate))
            .arg("-o")
            .arg(&output_template)
            .arg("--no-playlist")
            .arg("--newline")
            .arg(video_url)
            .stderr(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start download for video {}: {}", current_song_num, e))?;

        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        let mut song_progress = 0.0;

        // Parse progress for this single video
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let line = line.trim();
                    if !line.is_empty() {
                        // Parse download progress: [download] XX.X%
                        if line.contains("[download]") {
                            if let Some(percent_pos) = line.find('%') {
                                let mut num_start = percent_pos;
                                let mut found_digit = false;
                                
                                while num_start > 0 {
                                    let ch = line.chars().nth(num_start - 1).unwrap_or(' ');
                                    if ch.is_ascii_digit() || ch == '.' {
                                        found_digit = true;
                                        num_start -= 1;
                                    } else if found_digit {
                                        break;
                                    } else {
                                        num_start -= 1;
                                    }
                                }
                                
                                if found_digit && num_start < percent_pos {
                                    let percent_str = &line[num_start..percent_pos].trim();
                                    if let Ok(percent) = percent_str.parse::<f64>() {
                                        let new_progress = percent.min(100.0).max(0.0);
                                        
                                        // Only update if progress changed significantly
                                        if (new_progress - song_progress).abs() > 0.5 || song_progress == 0.0 {
                                            song_progress = new_progress;
                                            
                                            let overall_progress = if total_videos > 0 {
                                                ((index as f64 + song_progress / 100.0) / total_videos as f64) * 100.0
                                            } else {
                                                song_progress
                                            };

                                            let progress = DownloadProgress {
                                                overall_progress,
                                                current_song: Some(current_song_num),
                                                total_songs: Some(total_videos),
                                                song_progress,
                                                status: if song_progress >= 95.0 {
                                                    "Converting to MP3...".to_string()
                                                } else {
                                                    "Downloading...".to_string()
                                                },
                                                current_title: current_title.clone(),
                                            };
                                            app_handle.emit_all("download-progress", progress).ok();
                                        }
                                    }
                                }
                            }
                        }
                        // Check for conversion status
                        else if line.contains("[ExtractAudio]") || line.contains("[Merger]") {
                            song_progress = 95.0;
                            let overall_progress = if total_videos > 0 {
                                ((index as f64 + 0.95) / total_videos as f64) * 100.0
                            } else {
                                95.0
                            };

                            let progress = DownloadProgress {
                                overall_progress,
                                current_song: Some(current_song_num),
                                total_songs: Some(total_videos),
                                song_progress: 95.0,
                                status: "Converting to MP3...".to_string(),
                                current_title: current_title.clone(),
                            };
                            app_handle.emit_all("download-progress", progress).ok();
                        }
                    }
                }
                Err(_) => break,
            }
        }

        // Wait for process to complete
        let status_result = child.wait().await.map_err(|e| format!("Failed to wait for process: {}", e))?;

        if !status_result.success() {
            eprintln!("Warning: Download failed for video {}: {}", current_song_num, video_url);
            continue; // Skip this video and continue with next
        }

        // Emit 100% progress for this song
        let complete_progress = DownloadProgress {
            overall_progress: ((index + 1) as f64 / total_videos as f64) * 100.0,
            current_song: Some(current_song_num),
            total_songs: Some(total_videos),
            song_progress: 100.0,
            status: "Completed".to_string(),
            current_title: current_title.clone(),
        };
        app_handle.emit_all("download-progress", complete_progress).ok();

        // Find the downloaded file
        if let Ok(entries) = std::fs::read_dir(output_folder) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("mp3") {
                    let path_str = path.to_string_lossy().to_string();
                    if !existing_files.contains(&path_str) {
                        if let Ok(metadata) = std::fs::metadata(&path) {
                            let file_size = Some(metadata.len());
                            let file_name = path.file_stem()
                                .and_then(|s| s.to_str())
                                .map(|s| s.to_string());
                            
                            downloaded_videos.push(DownloadResult {
                                output_path: path_str,
                                title: file_name.or(current_title.clone()),
                                duration: None,
                                file_size,
                            });
                            break; // Found the file for this video
                        }
                    }
                }
            }
        }
    }

    // Emit final 100% progress
    let final_progress = DownloadProgress {
        overall_progress: 100.0,
        current_song: Some(total_videos),
        total_songs: Some(total_videos),
        song_progress: 100.0,
        status: "Complete!".to_string(),
        current_title: None,
    };
    app_handle.emit_all("download-progress", final_progress).ok();

    Ok(PlaylistDownloadResult {
        output_folder: output_folder.to_string(),
        total_videos,
        downloaded_videos,
    })
}

/// Validate if the URL is a valid YouTube URL
/// Supports various YouTube URL formats across different platforms
fn is_youtube_url(url: &str) -> bool {
    let url_lower = url.to_lowercase();
    url_lower.contains("youtube.com/watch") || 
    url_lower.contains("youtu.be/") ||
    url_lower.contains("youtube.com/embed/") ||
    url_lower.contains("youtube.com/v/") ||
    url_lower.contains("youtube.com/shorts/") ||
    url_lower.contains("m.youtube.com/watch") ||
    url_lower.contains("www.youtube.com/watch") ||
    url_lower.starts_with("https://youtube.com/") ||
    url_lower.starts_with("http://youtube.com/") ||
    url_lower.starts_with("https://youtu.be/") ||
    url_lower.starts_with("http://youtu.be/") ||
    url_lower.contains("youtube.com/playlist")
}

/// Check if the URL is a YouTube playlist URL
pub fn is_playlist_url(url: &str) -> bool {
    let url_lower = url.to_lowercase();
    // Check for playlist parameter in URL
    url_lower.contains("list=") && (
        url_lower.contains("youtube.com/watch") ||
        url_lower.contains("youtube.com/playlist")
    )
}

/// Sanitize filename to be safe for all operating systems
/// Removes or replaces characters that are invalid on Windows, macOS, and Linux
fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            // Invalid characters on Windows: < > : " / \ | ? *
            // Invalid characters on macOS/Linux: / and null
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' | '\0' => '_',
            // Control characters
            c if c.is_control() => '_',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .trim_end_matches('.') // Windows doesn't allow trailing dots
        .trim_end_matches(' ')  // Windows doesn't allow trailing spaces
        .to_string()
}

