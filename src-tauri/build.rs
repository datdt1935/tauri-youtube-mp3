use std::path::Path;

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

    let binary_ext = if cfg!(target_os = "windows") { ".exe" } else { "" };
    let required = vec!["yt-dlp", "ffmpeg"];

    let mut missing = Vec::new();
    let mut found = Vec::new();

    for binary in &required {
        let path = Path::new("binaries")
            .join(current_platform)
            .join(current_arch)
            .join(format!("{}{}", binary, binary_ext));
        
        if path.exists() {
            found.push(path.display().to_string());
        } else {
            missing.push(format!("binaries/{}/{}/{}{}", current_platform, current_arch, binary, binary_ext));
        }
    }

    if !found.is_empty() {
        println!("✅ Found binaries for current platform ({}/{}):", current_platform, current_arch);
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
    tauri_build::build()
}

