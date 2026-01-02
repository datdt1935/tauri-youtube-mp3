use std::fs;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

fn set_binary_permissions(path: &Path) -> bool {
    #[cfg(unix)]
    {
        if path.exists() {
            if let Ok(metadata) = fs::metadata(path) {
                let file_size = metadata.len();
                if file_size == 0 {
                    return false;
                }
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                if let Err(e) = fs::set_permissions(path, perms) {
                    eprintln!(
                        "⚠️  Warning: Failed to set permissions on {}: {}",
                        path.display(),
                        e
                    );
                    return false;
                }
                return true;
            }
        }
    }
    false
}

fn check_binaries() {
    let current_platform = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };

    let current_arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x64"
    };

    let binary_ext = if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    };
    let required = vec!["yt-dlp", "ffmpeg"];

    let mut missing = Vec::new();
    let mut found = Vec::new();

    for binary in &required {
        let path = Path::new("binaries")
            .join(current_platform)
            .join(current_arch)
            .join(format!("{}{}", binary, binary_ext));

        if path.exists() {
            set_binary_permissions(&path);
            found.push(path.display().to_string());
        } else {
            missing.push(format!(
                "binaries/{}/{}/{}{}",
                current_platform, current_arch, binary, binary_ext
            ));
        }
    }

    #[cfg(unix)]
    {
        let all_binaries = vec![
            "binaries/macos/arm64/yt-dlp",
            "binaries/macos/arm64/ffmpeg",
            "binaries/macos/x64/yt-dlp",
            "binaries/macos/x64/ffmpeg",
        ];
        for bin_path in all_binaries {
            let path = Path::new(bin_path);
            if !set_binary_permissions(path) && path.exists() {
                if let Ok(metadata) = fs::metadata(path) {
                    if metadata.len() == 0 {
                        eprintln!("⚠️  Warning: {} is empty (placeholder file)", bin_path);
                    }
                }
            }
        }
    }

    if !found.is_empty() {
        println!(
            "✅ Found binaries for current platform ({}/{}):",
            current_platform, current_arch
        );
        for path in &found {
            println!("   - {}", path);
        }
    }

    if !missing.is_empty() {
        eprintln!("\n⚠️  WARNING: Some binaries are missing for current platform!");
        eprintln!("   Missing: {:?}", missing);
        eprintln!("\n   The app will build, but downloads may fail at runtime.");
        eprintln!("   To fix: Place the missing binaries in the repository.");
        eprintln!("\n   Download instructions:");
        eprintln!("     - yt-dlp: https://github.com/yt-dlp/yt-dlp/releases/latest");
        eprintln!("     - ffmpeg: https://ffmpeg.org/download.html");
        eprintln!("\n   After adding binaries, update tauri.conf.json:");
        eprintln!("     Add specific binary paths to bundle.resources, e.g.:");
        eprintln!("     \"resources\": [\"binaries/macos/arm64/yt-dlp\", \"binaries/macos/arm64/ffmpeg\"]\n");
    }
}

fn main() {
    check_binaries();

    #[cfg(unix)]
    {
        use std::process::Command;
        let resources = vec![
            "binaries/macos/arm64/yt-dlp",
            "binaries/macos/arm64/ffmpeg",
            "binaries/macos/x64/yt-dlp",
            "binaries/macos/x64/ffmpeg",
        ];

        for resource in resources {
            let path = Path::new(resource);
            if path.exists() {
                if let Ok(metadata) = fs::metadata(path) {
                    if metadata.len() > 0 {
                        let output = Command::new("xattr")
                            .args(&["-d", "com.apple.quarantine", resource])
                            .output();
                        if output.is_err() {
                            let output = Command::new("xattr").args(&["-c", resource]).output();
                            if output.is_err() {
                                eprintln!(
                                    "⚠️  Warning: Could not remove extended attributes from {}",
                                    resource
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    tauri_build::build()
}
