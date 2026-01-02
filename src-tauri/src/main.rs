// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod deps;
mod download;

use commands::*;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            download_from_youtube,
            get_download_history,
            clear_history,
            check_deps,
            clear_extracted_binaries,
            save_output_folder,
            get_output_folder,
            save_preferences,
            get_preferences
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
