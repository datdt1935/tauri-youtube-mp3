use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversionResult {
    pub output_path: String,
    pub duration: Option<f64>,
    pub file_size: Option<u64>,
}

pub async fn convert_file(
    input_path: &str,
    output_folder: &str,
    bitrate: u32,
) -> Result<ConversionResult, String> {
    let input = Path::new(input_path);
    if !input.exists() {
        return Err(format!("Input file does not exist: {}", input_path));
    }

    // Generate output filename
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid input filename")?;
    let output_path = Path::new(output_folder).join(format!("{}.mp3", stem));

    // Check if ffmpeg is available
    let ffmpeg_check = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .await;

    if ffmpeg_check.is_err() {
        return Err("FFmpeg is not installed. Please install FFmpeg to use this application.".to_string());
    }

    // Build ffmpeg command
    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(input_path)
        .arg("-vn") // No video
        .arg("-acodec")
        .arg("libmp3lame")
        .arg("-ab")
        .arg(format!("{}k", bitrate))
        .arg("-ar")
        .arg("44100")
        .arg("-y") // Overwrite output file
        .arg(output_path.to_str().unwrap())
        .output()
        .await
        .map_err(|e| format!("FFmpeg execution failed: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Conversion failed: {}", error));
    }

    // Get file size
    let file_size = std::fs::metadata(&output_path)
        .ok()
        .map(|m| m.len());

    // Try to get duration (optional)
    let duration = get_duration(input_path).await.ok();

    Ok(ConversionResult {
        output_path: output_path.to_string_lossy().to_string(),
        duration,
        file_size,
    })
}

async fn get_duration(input_path: &str) -> Result<f64, String> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg(input_path)
        .output()
        .await
        .map_err(|e| format!("FFprobe execution failed: {}", e))?;

    if !output.status.success() {
        return Err("Failed to get duration".to_string());
    }

    let duration_str = String::from_utf8_lossy(&output.stdout);
    duration_str
        .trim()
        .parse::<f64>()
        .map_err(|_| "Failed to parse duration".to_string())
}

