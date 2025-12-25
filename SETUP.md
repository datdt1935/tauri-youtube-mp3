# Quick Setup Guide

## Prerequisites Check

Before starting, ensure you have:

1. **Node.js** (v16+): `node --version`
2. **PNPM**: `pnpm --version` (if not installed: `npm install -g pnpm`)
3. **Rust**: `rustc --version` (if not installed: visit https://rustup.rs/)
4. **yt-dlp**: `yt-dlp --version` (macOS: `brew install yt-dlp`)
5. **FFmpeg**: `ffmpeg -version` (macOS: `brew install ffmpeg`)

## Installation Steps

1. **Install dependencies:**
   ```bash
   pnpm install
   ```

2. **Verify Rust setup:**
   ```bash
   cd src-tauri
   cargo --version
   cd ..
   ```

## Running the Application

**Development mode:**
```bash
pnpm tauri dev
```

This will:
- Install Rust dependencies (first time only, may take a few minutes)
- Start the Vite dev server
- Launch the Tauri app window

## Building for Production

```bash
pnpm tauri build
```

Output will be in: `src-tauri/target/release/bundle/`

## Project Structure Overview

- `src/` - React frontend (TypeScript + SCSS)
- `src-tauri/` - Rust backend (Tauri commands)
- `docs/` - Documentation

## Troubleshooting

### yt-dlp not found
- macOS: `brew install yt-dlp`
- Verify: `which yt-dlp` and `yt-dlp --version`
- Update: `brew upgrade yt-dlp`

### FFmpeg not found
- macOS: `brew install ffmpeg`
- Verify: `which ffmpeg` and `ffmpeg -version`

### Rust/Cargo errors
- Update Rust: `rustup update`
- Clean build: `cd src-tauri && cargo clean && cd ..`

### Port 1420 already in use
- Change port in `vite.config.ts` if needed
- Or kill the process using that port

## Next Steps

1. Add app icons to `src-tauri/icons/` (optional, Tauri will use defaults)
2. Customize colors in `src/styles/variables.scss`
3. Test with a YouTube video URL (e.g., `https://www.youtube.com/watch?v=dQw4w9WgXcQ`)

