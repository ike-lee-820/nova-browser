#!/usr/bin/env bash
# CEF 自动下载脚本 (Linux / macOS / CI)
#
# 用法:
#   ./download-cef.sh                           # 下载当前平台
#   ./download-cef.sh windows64                 # 下载指定平台
#   ./download-cef.sh linux64 120.1.10+g3ce3184  # 指定平台和版本
#
# 如果不指定版本，使用默认版本

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CEF_DIR="$SCRIPT_DIR/cef"

# 默认 CEF 版本
DEFAULT_VERSION="149.0.4+g2f1bfd8+chromium-149.0.7827.156"

# 自动检测当前平台
detect_platform() {
    local os=""
    local arch=""
    case "$(uname -s)" in
        Linux*)  os="linux" ;;
        Darwin*) os="macos" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *)       os="unknown" ;;
    esac
    case "$(uname -m)" in
        x86_64|amd64) arch="x64" ;;
        aarch64|arm64) arch="arm64" ;;
        armv7l|armhf) arch="arm" ;;
        i686) arch="x86" ;;
        *) arch="unknown" ;;
    esac

    case "$os-$arch" in
        windows-x64) echo "windows64" ;;
        windows-x86) echo "windows32" ;;
        windows-arm64) echo "windowsarm64" ;;
        linux-x64)    echo "linux64" ;;
        linux-arm64)  echo "linuxarm64" ;;
        linux-arm)    echo "linuxarm" ;;
        macos-x64)    echo "macosx64" ;;
        macos-arm64)  echo "macosarm64" ;;
        *)            echo "" ;;
    esac
}

# 平台 → 目录名映射
platform_to_dir() {
    case "$1" in
        windows64)    echo "win64" ;;
        windows32)    echo "win32" ;;
        windowsarm64) echo "win-arm64" ;;
        linux64)      echo "linux64" ;;
        linuxarm64)   echo "linux-arm64" ;;
        linuxarm)     echo "linux-arm32" ;;
        macosx64)     echo "macos64" ;;
        macosarm64)   echo "macos-arm64" ;;
        *)            echo "" ;;
    esac
}

PLATFORM="${1:-$(detect_platform)}"
VERSION="${2:-$DEFAULT_VERSION}"

if [ -z "$PLATFORM" ]; then
    echo "[错误] 无法检测当前平台，请手动指定: $0 <平台代码>"
    echo "  可用平台: windows64, windows32, windowsarm64, linux64, linuxarm64, linuxarm, macosx64, macosarm64"
    exit 1
fi

PLATFORM_DIR=$(platform_to_dir "$PLATFORM")
if [ -z "$PLATFORM_DIR" ]; then
    echo "[错误] 不支持的平台: $PLATFORM"
    exit 1
fi

DEST_DIR="$CEF_DIR/$PLATFORM_DIR"

# 检查是否已存在
if [ -d "$DEST_DIR/Release" ] || [ -d "$DEST_DIR/Debug" ]; then
    echo "[跳过] CEF 已存在于 $DEST_DIR"
    exit 0
fi

# 构建下载 URL
# 所有平台统一使用 tar.bz2 格式
EXT="tar.bz2"
# URL 编码版本号中的 + 号
ENCODED_VERSION="${VERSION//+/%2B}"
FILENAME="cef_binary_${ENCODED_VERSION}_${PLATFORM}.${EXT}"
URL="https://cef-builds.spotifycdn.com/${FILENAME}"

echo "下载 CEF ${PLATFORM}..."
echo "  URL: $URL"
echo "  目标: $DEST_DIR"

mkdir -p "$DEST_DIR" "$CEF_DIR/_temp"

DOWNLOAD_PATH="$CEF_DIR/$FILENAME"

# 下载 (带重试)
echo "  下载中 (约 500MB-1.5GB，请耐心等待)..."
for i in 1 2 3; do
    if curl -L --fail --retry 3 -o "$DOWNLOAD_PATH" "$URL"; then
        break
    fi
    echo "  下载失败，重试 $i/3..."
    sleep 5
done

if [ ! -f "$DOWNLOAD_PATH" ]; then
    echo "[失败] 下载 CEF 失败"
    echo "  你可以手动下载: $URL"
    echo "  解压到: $DEST_DIR"
    exit 1
fi

# 解压 (tar.bz2)
echo "  解压中..."
tar -xjf "$DOWNLOAD_PATH" -C "$CEF_DIR/_temp"

# 移动内容到平台目录
EXTRACTED_DIR=$(find "$CEF_DIR/_temp" -maxdepth 1 -type d | head -2 | tail -1)
if [ -n "$EXTRACTED_DIR" ] && [ -d "$EXTRACTED_DIR" ]; then
    shopt -s dotglob
    mv "$EXTRACTED_DIR"/* "$DEST_DIR/" 2>/dev/null || true
    shopt -u dotglob
fi

# 清理
rm -rf "$CEF_DIR/_temp"
rm -f "$DOWNLOAD_PATH"

echo "  [OK] CEF $PLATFORM 下载完成 → $DEST_DIR"