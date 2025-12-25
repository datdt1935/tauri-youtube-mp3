# Icons Directory

This directory contains the application icons required by Tauri.

## Required Files

- `32x32.png` - 32x32 pixel PNG icon
- `128x128.png` - 128x128 pixel PNG icon  
- `128x128@2x.png` - 256x256 pixel PNG icon (for Retina displays)
- `icon.icns` - macOS icon file
- `icon.ico` - Windows icon file

## Generating Icons

You can generate proper icons using:
- Online tools: https://www.icoconverter.com/
- ImageMagick: `convert input.png -resize 32x32 icons/32x32.png`
- Or use Tauri's icon generation: `tauri icon path/to/icon.png`

For now, placeholder icons have been created to allow the build to proceed.
You should replace these with proper application icons before releasing.

