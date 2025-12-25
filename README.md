# YouTube Downloader

A desktop application for downloading YouTube videos and converting them to MP3 format, built with Tauri, React, TypeScript, and SCSS.

## ⚠️ Legal Note

This application is intended for downloading content that users own or have legal rights to use. Downloading copyrighted content without permission may violate copyright laws & platform TOS. Users are responsible for ensuring they have the right to download and use the content.

## Features

- **URL Input**: Enter YouTube video URLs with validation
- **Download & Convert**: Download video and convert to MP3 format in one step using yt-dlp
- **Bitrate Options**: Choose from 128, 192, or 320 kbps
- **Output Folder Select**: Choose where to save downloaded files
- **Progress UI**: Real-time download and conversion progress indicator
- **Download History**: View and manage past downloads with metadata (title, bitrate, duration, timestamp)
- **Clear History**: Remove all download history entries
- **System Notifications**: Desktop notification when download completes
- **Automatic Dependency Management**: Automatically downloads yt-dlp and FFmpeg if not found in PATH
- **Dependency Checking**: Built-in commands to check and setup required dependencies

## Prerequisites

- **Node.js** (v16 or higher)
- **PNPM** package manager (v10.17.0+)
- **Rust** (latest stable version)
- **yt-dlp** - Required for YouTube downloading
  - **Windows**: `winget install yt-dlp` or download from [GitHub releases](https://github.com/yt-dlp/yt-dlp/releases/latest)
  - **macOS**: `brew install yt-dlp`
  - **Linux**: `sudo apt install yt-dlp` or `pip install yt-dlp`
  - **Note**: The app can automatically download yt-dlp if not found in PATH
- **FFmpeg** - Required for audio conversion
  - **Windows**: `winget install ffmpeg` or download from [FFmpeg website](https://ffmpeg.org/download.html)
  - **macOS**: `brew install ffmpeg`
  - **Linux**: `sudo apt install ffmpeg`
  - **Note**: The app can automatically download FFmpeg if not found in PATH

**See [INSTALLATION.md](INSTALLATION.md) for detailed installation instructions.**

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd tauri-download-youtube
```

2. Install dependencies:
```bash
pnpm install
```

3. Install Rust dependencies (this happens automatically on first build):
```bash
cd src-tauri
cargo build
cd ..
```

## Development

Run the development server:

```bash
pnpm tauri dev
```

This will:
- Start the Vite dev server for the frontend
- Build and run the Tauri application
- Enable hot-reload for both frontend and backend

## Building

Build the application for production:

```bash
pnpm tauri build
```

The built application will be in `src-tauri/target/release/bundle/`.

## Project Structure

```
.
├── src/                    # React frontend source
│   ├── components/         # React components
│   │   ├── UrlInput/      # YouTube URL input, bitrate selection, and download button
│   │   ├── Progress/      # Download progress indicator
│   │   ├── History/       # Download history display and management
│   │   └── FileImport/    # File import component (unused, legacy)
│   ├── styles/            # SCSS styles (BEM architecture)
│   │   ├── variables.scss # SCSS variables
│   │   ├── base.scss      # Base styles
│   │   └── main.scss      # Main stylesheet
│   ├── App.tsx            # Main App component
│   ├── App.scss           # App-specific styles
│   └── main.tsx           # Entry point
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs        # Tauri entry point and command registration
│   │   ├── commands.rs    # Tauri commands (download, history, dependencies)
│   │   ├── download.rs    # yt-dlp download logic and dependency management
│   │   └── conversion.rs  # File conversion utilities (legacy, not used for YouTube)
│   ├── Cargo.toml         # Rust dependencies
│   ├── tauri.conf.json    # Tauri configuration
│   └── icons/             # Application icons
├── dist/                   # Built frontend assets
├── INSTALLATION.md         # Detailed installation guide
├── SETUP.md                # Setup instructions
└── package.json            # Node.js dependencies
```

## Usage

1. **Enter YouTube URL**: Paste a YouTube video URL in the input field (supports youtube.com/watch and youtu.be formats)
2. **Choose output folder**: Click "Choose Output Folder" to select where the MP3 will be saved
3. **Select bitrate**: Choose your preferred bitrate (128, 192, or 320 kbps) using the button options
4. **Download**: Click "Start Download" to begin downloading and converting to MP3
5. **Monitor progress**: Watch the progress bar and status messages during download
6. **View history**: Check your download history at the bottom of the app, showing title, bitrate, duration, and timestamp
7. **Clear history**: Use the "Clear History" button to remove all history entries

## Supported Formats

### Input
- YouTube video URLs (youtube.com/watch, youtu.be, etc.)

### Output Format
- MP3 (128, 192, or 320 kbps)

## Troubleshooting

### yt-dlp not found
If you get an error about yt-dlp not being installed:
- The app will attempt to automatically download yt-dlp to the app's data directory
- Manual installation:
  - **Windows**: `winget install yt-dlp` or download from [GitHub releases](https://github.com/yt-dlp/yt-dlp/releases/latest)
  - **macOS**: `brew install yt-dlp`
  - **Linux**: `sudo apt install yt-dlp` or `pip install yt-dlp`
- Verify installation: `yt-dlp --version`
- Update yt-dlp: `brew upgrade yt-dlp` (macOS) or reinstall on other platforms

### FFmpeg not found
If you get an error about FFmpeg not being installed:
- The app will attempt to automatically download FFmpeg to the app's data directory
- Manual installation:
  - **Windows**: `winget install ffmpeg` or download from [FFmpeg website](https://ffmpeg.org/download.html)
  - **macOS**: `brew install ffmpeg`
  - **Linux**: `sudo apt install ffmpeg`
- Verify installation: `ffmpeg -version`

### Build errors
- Make sure Rust is installed: `rustc --version`
- Update Rust: `rustup update`
- Clean and rebuild: `cd src-tauri && cargo clean && cd .. && pnpm tauri build`

### Download failures
- Check your internet connection
- Verify the YouTube URL is valid and accessible
- Ensure the output folder has write permissions
- Check that yt-dlp and FFmpeg are properly installed (the app will try to auto-install them)

## Technical Details

### Tauri Commands

The application exposes the following Tauri commands:

- `download_from_youtube(url, output_folder, bitrate)` - Downloads and converts YouTube video to MP3
- `get_download_history()` - Retrieves download history
- `clear_history()` - Clears all download history
- `check_required_dependencies()` - Checks if yt-dlp and FFmpeg are installed
- `setup_ytdlp()` - Automatically downloads yt-dlp if not found
- `setup_ffmpeg()` - Automatically downloads FFmpeg if not found

### Download Process

1. Validates YouTube URL format
2. Checks/installs yt-dlp and FFmpeg dependencies
3. Retrieves video metadata (title, duration)
4. Downloads audio and converts to MP3 in one step using yt-dlp
5. Saves download to history
6. Sends system notification on completion

### History Storage

Download history is stored locally in JSON format at:
- **Windows**: `%APPDATA%\youtube-downloader\history.json`
- **macOS**: `~/Library/Application Support/youtube-downloader/history.json`
- **Linux**: `~/.config/youtube-downloader/history.json`

History is limited to the last 100 downloads.

## Future Features

- Batch downloading multiple URLs
- Playlist support
- Video format download (MP4)
- Quality selection (720p, 1080p, etc.)
- Dark/light theme toggle
- More output formats (WAV, FLAC, AAC)
- Export/import download history

## License

[Add your license here]

## Contributing

[Add contributing guidelines here]

