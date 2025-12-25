// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod download;

use commands::*;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            select_output_folder,
            download_from_youtube,
            get_download_history,
            clear_history,
            check_required_dependencies,
            setup_ytdlp,
            setup_ffmpeg,
            save_output_folder,
            get_output_folder,
            save_preferences,
            get_preferences
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

