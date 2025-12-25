# Installation Guide

## Automatic Installation (Recommended)

**Good news!** This application can automatically download and install `yt-dlp` and `FFmpeg` if they are not found in your system PATH. When you first run the app or attempt a download, it will:

1. Check if `yt-dlp` and `FFmpeg` are available in your system PATH
2. If not found, automatically download them to the app's data directory
3. Use the downloaded binaries for all operations

**Storage locations for auto-downloaded binaries:**
- **Windows**: `%APPDATA%\youtube-downloader\bin\`
- **macOS**: `~/Library/Application Support/youtube-downloader/bin/`
- **Linux**: `~/.config/youtube-downloader/bin/`

You don't need to manually install anything if you're okay with the app managing these dependencies automatically. However, if you prefer to install them system-wide (which allows other applications to use them), follow the manual installation instructions below.

## Manual Installation (Optional)

If you prefer to install the tools system-wide or want more control, you can manually install:

1. **yt-dlp** - For downloading YouTube videos
2. **FFmpeg** - For converting audio to MP3

Manual installation is recommended if:
- You want to use these tools with other applications
- You want to keep them updated via your system's package manager
- You prefer system-wide installations

## Windows Installation

### Option 1: Using winget (Recommended)
```bash
winget install yt-dlp --accept-source-agreements --accept-package-agreements
winget install ffmpeg --accept-source-agreements --accept-package-agreements
```

**Note:** After installation, **restart your terminal/command prompt and the Tauri application** for PATH changes to take effect.

### Option 2: Using pip (Python required)
```bash
pip install yt-dlp
```

For FFmpeg, download from: https://ffmpeg.org/download.html
- Extract the zip file
- Add the `bin` folder to your system PATH

### Option 3: Using Chocolatey
```bash
choco install yt-dlp
choco install ffmpeg
```

### Option 4: Manual Installation

**yt-dlp:**
1. Download from: https://github.com/yt-dlp/yt-dlp/releases/latest
2. Download `yt-dlp.exe`
3. Place it in a folder (e.g., `C:\tools\`)
4. Add that folder to your system PATH

**FFmpeg:**
1. Download from: https://ffmpeg.org/download.html
2. Extract the zip file
3. Add the `bin` folder to your system PATH

### Adding to PATH (Windows)

1. Right-click "This PC" → Properties
2. Click "Advanced system settings"
3. Click "Environment Variables"
4. Under "System variables", find "Path" and click "Edit"
5. Click "New" and add the folder containing `yt-dlp.exe` and `ffmpeg.exe`
6. Click "OK" on all dialogs
7. Restart your terminal/command prompt

### Verify Installation

Open a new Command Prompt or PowerShell and run:
```bash
yt-dlp --version
ffmpeg -version
```

Both commands should show version information if installed correctly.

## macOS Installation

```bash
brew install yt-dlp
brew install ffmpeg
```

## Linux Installation

**Debian/Ubuntu:**
```bash
sudo apt install yt-dlp ffmpeg
```

**Or using pip:**
```bash
pip install yt-dlp
sudo apt install ffmpeg
```

## Troubleshooting

### Automatic Installation Issues

If the app fails to automatically download dependencies:

1. **Check internet connection** - The app needs internet access to download binaries
2. **Check disk space** - Ensure you have enough space in the app's data directory
3. **Check permissions** - The app needs write permissions to its data directory
4. **Try manual installation** - If automatic installation fails, install manually using the instructions above

### "yt-dlp is not recognized" (Manual Installation)

- Make sure yt-dlp is in your PATH
- Restart your terminal/command prompt after adding to PATH
- On Windows, you may need to restart the application
- Verify installation: `yt-dlp --version`
- **Alternative**: Let the app auto-install it (no PATH setup needed)

### "FFmpeg is not recognized" (Manual Installation)

- Make sure FFmpeg is in your PATH
- Verify installation: `ffmpeg -version`
- Restart the application after installation
- **Alternative**: Let the app auto-install it (no PATH setup needed)

### Download Failures

- If downloads fail even with dependencies installed:
  - Check your internet connection
  - Verify the YouTube URL is valid
  - Ensure the output folder has write permissions
  - Try updating yt-dlp: `yt-dlp -U` (if manually installed)

### Still having issues?

- **For automatic installation**: Check the app's error messages for specific issues
- **For manual installation**: 
  - Check that both tools work in your terminal/command prompt
  - Make sure you're using the latest versions
  - Try restarting your computer after installation
- **General**: The app will attempt to use auto-downloaded binaries first, then fall back to PATH if available

