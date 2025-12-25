use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use tokio::process::Command;
use tauri::api::path::config_dir;

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadResult {
    pub output_path: String,
    pub title: Option<String>,
    pub duration: Option<f64>,
    pub file_size: Option<u64>,
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

    // Determine the actual output path
    // yt-dlp will have created a file with the title as the name
    let output_path = if let Some(ref t) = title {
        Path::new(output_folder).join(format!("{}.mp3", sanitize_filename(t)))
    } else {
        // Fallback: try to find the most recent .mp3 file in the output folder
        let mut fallback_path = Path::new(output_folder).join("video.mp3");
        if let Ok(entries) = std::fs::read_dir(output_folder) {
            let mut mp3_files: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().extension().and_then(|s| s.to_str()) == Some("mp3")
                })
                .collect();
            mp3_files.sort_by_key(|e| {
                e.metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            });
            if let Some(latest) = mp3_files.last() {
                fallback_path = latest.path();
            }
        }
        fallback_path
    };

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
    url_lower.starts_with("http://youtu.be/")
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

