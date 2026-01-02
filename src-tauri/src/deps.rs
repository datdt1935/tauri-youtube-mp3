use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::AppHandle;

static EXTRACTION_LOCK: Mutex<()> = Mutex::new(());

fn get_platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    }
}

fn get_arch() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x64"
    }
}

fn get_binary_name(binary: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{}.exe", binary)
    } else {
        binary.to_string()
    }
}

fn get_bundled_binary_path(app_handle: &AppHandle, binary: &str) -> Result<PathBuf> {
    let platform = get_platform();
    let arch = get_arch();
    let binary_name = get_binary_name(binary);

    let resource_path = format!("binaries/{}/{}/{}", platform, arch, binary_name);

    if let Some(resource_dir) = app_handle.path_resolver().resource_dir() {
        eprintln!("[deps] Resource directory: {:?}", resource_dir);
    } else {
        eprintln!("[deps] WARNING: Resource directory not available");
    }

    eprintln!("[deps] Attempting to resolve resource: {}", resource_path);

    match app_handle.path_resolver().resolve_resource(&resource_path) {
        Some(path) => {
            eprintln!("[deps] Successfully resolved resource: {:?}", path);
            Ok(path)
        }
        None => {
            let error_msg = format!(
                "Resource '{}' is not bundled. Make sure binaries are placed in src-tauri/binaries/{}/{}/ and tauri.conf.json includes 'binaries/**' in bundle.resources",
                resource_path, platform, arch
            );
            eprintln!("[deps] ERROR: {}", error_msg);
            Err(anyhow::anyhow!(error_msg))
        }
    }
}

fn get_app_bin_dir(app_handle: &AppHandle) -> Result<PathBuf> {
    let app_data_dir = app_handle
        .path_resolver()
        .app_data_dir()
        .context("Failed to get app data directory")?;

    let bin_dir = app_data_dir.join("bin");
    Ok(bin_dir)
}

fn get_extracted_binary_path(app_handle: &AppHandle, binary: &str) -> Result<PathBuf> {
    let bin_dir = get_app_bin_dir(app_handle)?;
    let binary_name = get_binary_name(binary);
    Ok(bin_dir.join(&binary_name))
}

fn copy_binary_atomic(source: &Path, dest: &Path) -> Result<()> {
    let parent = dest
        .parent()
        .context("Destination has no parent directory")?;
    fs::create_dir_all(parent).context("Failed to create bin directory")?;

    let temp_dest = dest.with_extension(format!(
        "{}.tmp",
        dest.extension().and_then(|s| s.to_str()).unwrap_or("")
    ));

    fs::copy(source, &temp_dest).context("Failed to copy binary to temp location")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_dest)
            .context("Failed to get temp file metadata")?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_dest, perms).context("Failed to set executable permissions")?;
    }

    fs::rename(&temp_dest, dest).context("Failed to rename temp file to final destination")?;

    Ok(())
}

fn extract_binary(app_handle: &AppHandle, binary: &str) -> Result<PathBuf> {
    let _lock = EXTRACTION_LOCK.lock().unwrap();

    let extracted_path = get_extracted_binary_path(app_handle, binary)?;
    eprintln!("[deps] Extracted binary path: {:?}", extracted_path);

    if extracted_path.exists() {
        eprintln!("[deps] Extracted binary already exists, verifying...");

        let file_size = fs::metadata(&extracted_path)
            .ok()
            .map(|m| m.len())
            .unwrap_or(0);

        if file_size == 0 {
            eprintln!("[deps] Extracted binary is empty (0 bytes), removing placeholder...");
            fs::remove_file(&extracted_path).ok();
        } else {
            let result = std::process::Command::new(&extracted_path)
                .arg(if binary == "ffmpeg" {
                    "-version"
                } else {
                    "--version"
                })
                .output();

            if result.is_ok() && result.as_ref().unwrap().status.success() {
                eprintln!("[deps] Extracted binary is valid, using existing copy");
                return Ok(extracted_path);
            }

            eprintln!("[deps] Extracted binary is invalid, removing and re-extracting...");
            fs::remove_file(&extracted_path).ok();
        }
    }

    eprintln!("[deps] Extracting bundled binary: {}", binary);
    let bundled_path = get_bundled_binary_path(app_handle, binary)?;

    if !bundled_path.exists() {
        anyhow::bail!(
            "Bundled binary does not exist at resolved path: {}",
            bundled_path.display()
        );
    }

    let file_size = fs::metadata(&bundled_path)
        .context("Failed to get bundled binary metadata")?
        .len();

    if file_size == 0 {
        anyhow::bail!(
            "Bundled binary at {} is empty (0 bytes). This is likely a placeholder file.\n\nPlease replace it with an actual {} binary:\n- yt-dlp: https://github.com/yt-dlp/yt-dlp/releases/latest\n- ffmpeg: https://ffmpeg.org/download.html",
            bundled_path.display(),
            binary
        );
    }

    eprintln!("[deps] Bundled binary size: {} bytes", file_size);
    eprintln!(
        "[deps] Copying from {:?} to {:?}",
        bundled_path, extracted_path
    );
    copy_binary_atomic(&bundled_path, &extracted_path)
        .context(format!("Failed to extract binary: {}", binary))?;

    eprintln!("[deps] Successfully extracted binary: {}", binary);
    Ok(extracted_path)
}

pub fn get_bundled_binary(app_handle: &AppHandle, binary: &str) -> Result<PathBuf> {
    extract_binary(app_handle, binary)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DepsCheckResult {
    pub ytdlp_path: Option<String>,
    pub ffmpeg_path: Option<String>,
    pub ytdlp_version: Option<String>,
    pub ffmpeg_version: Option<String>,
    pub ytdlp_error: Option<String>,
    pub ffmpeg_error: Option<String>,
}

pub fn check_deps(app_handle: &AppHandle) -> DepsCheckResult {
    eprintln!("[deps] Checking dependencies...");
    let mut result = DepsCheckResult {
        ytdlp_path: None,
        ffmpeg_path: None,
        ytdlp_version: None,
        ffmpeg_version: None,
        ytdlp_error: None,
        ffmpeg_error: None,
    };

    eprintln!("[deps] Checking yt-dlp...");
    match get_bundled_binary(app_handle, "yt-dlp") {
        Ok(path) => {
            result.ytdlp_path = Some(path.to_string_lossy().to_string());

            match std::process::Command::new(&path).arg("--version").output() {
                Ok(output) => {
                    if output.status.success() {
                        result.ytdlp_version = String::from_utf8_lossy(&output.stdout)
                            .trim()
                            .to_string()
                            .into();
                    } else {
                        result.ytdlp_error = Some("yt-dlp version check failed".to_string());
                    }
                }
                Err(e) => {
                    result.ytdlp_error = Some(format!("Failed to run yt-dlp: {}", e));
                }
            }
        }
        Err(e) => {
            eprintln!("[deps] ERROR: Failed to extract yt-dlp: {}", e);
            result.ytdlp_error = Some(format!("Failed to extract yt-dlp: {}", e));
        }
    }

    eprintln!("[deps] Checking ffmpeg...");
    match get_bundled_binary(app_handle, "ffmpeg") {
        Ok(path) => {
            result.ffmpeg_path = Some(path.to_string_lossy().to_string());

            match std::process::Command::new(&path).arg("-version").output() {
                Ok(output) => {
                    if output.status.success() {
                        let version_output = String::from_utf8_lossy(&output.stdout);
                        if let Some(first_line) = version_output.lines().next() {
                            result.ffmpeg_version = Some(first_line.to_string());
                        }
                    } else {
                        result.ffmpeg_error = Some("ffmpeg version check failed".to_string());
                    }
                }
                Err(e) => {
                    result.ffmpeg_error = Some(format!("Failed to run ffmpeg: {}", e));
                }
            }
        }
        Err(e) => {
            eprintln!("[deps] ERROR: Failed to extract ffmpeg: {}", e);
            result.ffmpeg_error = Some(format!("Failed to extract ffmpeg: {}", e));
        }
    }

    eprintln!("[deps] Dependency check complete");
    result
}
