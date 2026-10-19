#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_DIR="$ROOT_DIR/src-tauri/resources/ffmpeg"
mkdir -p "$TARGET_DIR"

RELEASE_BASE="https://github.com/julyx10/lap-binaries/releases/download/ffmpeg-8.1"

# Support overriding OS and ARCH for cross-compilation or CI
OS_TYPE="${1:-$(uname -s)}"
ARCH_TYPE="${2:-$(uname -m)}"

case "$OS_TYPE" in
  Darwin|macos|macos-*)
    case "$ARCH_TYPE" in
      x86_64|x64)
        FFMPEG_FILE="ffmpeg-x86_64-apple-darwin"
        FFPROBE_FILE="ffprobe-x86_64-apple-darwin"
        ;;
      arm64|aarch64)
        FFMPEG_FILE="ffmpeg-aarch64-apple-darwin"
        FFPROBE_FILE="ffprobe-aarch64-apple-darwin"
        ;;
      *)
        echo "Unsupported macOS architecture: $ARCH_TYPE"
        exit 1
        ;;
    esac
    ;;
  Linux|linux|linux-*|ubuntu-*)
    case "$ARCH_TYPE" in
      x86_64|x64)
        FFMPEG_FILE="ffmpeg-x86_64-unknown-linux-gnu"
        FFPROBE_FILE="ffprobe-x86_64-unknown-linux-gnu"
        ;;
      aarch64|arm64)
        FFMPEG_FILE="ffmpeg-aarch64-unknown-linux-gnu"
        FFPROBE_FILE="ffprobe-aarch64-unknown-linux-gnu"
        ;;
      *)
        echo "Unsupported Linux architecture: $ARCH_TYPE"
        exit 1
        ;;
    esac
    ;;
  MINGW*|MSYS*|CYGWIN*|Windows_NT|windows|windows-*)
    FFMPEG_FILE="ffmpeg-x86_64-pc-windows-msvc.exe"
    FFPROBE_FILE="ffprobe-x86_64-pc-windows-msvc.exe"
    ;;
  *)
    echo "Unsupported platform: $OS_TYPE"
    exit 1
    ;;
esac

DOWNLOADS=(
  "$FFMPEG_FILE"
  "$FFPROBE_FILE"
)

echo "Downloading FFmpeg sidecar files into $TARGET_DIR..."
for FILENAME in "${DOWNLOADS[@]}"; do
  URL="$RELEASE_BASE/$FILENAME"
  DEST="$TARGET_DIR/$FILENAME"
  if [ -f "$DEST" ]; then
    echo "✓ $FILENAME already exists, skipping."
  else
    echo "⬇ Downloading $FILENAME..."
    curl -L -o "$DEST" "$URL"
    chmod +x "$DEST" || true
    echo "✓ $FILENAME downloaded."
  fi
 done

echo "All FFmpeg sidecar files downloaded into $TARGET_DIR."