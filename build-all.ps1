# Nova Browser — 全平台构建脚本 (Windows x64)
#
# 用法: 在 PowerShell 中运行
#   .\build-all.ps1
# 或指定输出目录:
#   .\build-all.ps1 -OutputDir "L:\BBD"
#
# 前提条件:
#   1. Rust 工具链 (rustup)
#   2. Visual Studio Build Tools (含 C++ 工具链)
#   3. WSL2 (用于编译 Linux 目标)
#   4. Android NDK (用于编译 Android 目标)
#   5. cargo-ndk (用于 Android 构建)

param(
    [string]$OutputDir = "L:\BBD",
    [string]$CefVersion = "",   # 指定 CEF 版本以自动下载，留空则跳过
    [switch]$SkipLinux,
    [switch]$SkipAndroid,
    [switch]$SkipMacOS,         # macOS 默认跳过，因为无法在 Windows 上编译
    [switch]$SkipCefDownload    # 即使指定了 CefVersion 也跳过 CEF 下载
)

$ErrorActionPreference = "Continue"
$ProjectRoot = $PSScriptRoot

Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  Nova Browser — 全平台构建" -ForegroundColor Cyan
Write-Host "  输出目录: $OutputDir" -ForegroundColor Cyan
if ($CefVersion) {
    Write-Host "  CEF 版本: $CefVersion (自动下载)" -ForegroundColor Cyan
} else {
    Write-Host "  CEF: 跳过下载 (使用本地已有的 cef/)   " -ForegroundColor DarkGray
    Write-Host "  提示: 指定 -CefVersion 以自动下载 CEF" -ForegroundColor DarkGray
}
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# CEF 下载辅助函数
function Download-CefOnce {
    param([string]$Platform)
    if ($SkipCefDownload -or -not $CefVersion) { return }
    $destDir = "$ProjectRoot\cef\$((Get-CefDir $Platform))"
    if (Test-Path "$destDir\Release") { return }  # 已存在则跳过
    & "$ProjectRoot\download-cef.ps1" -Platform $Platform -Version $CefVersion
}
function Get-CefDir {
    param([string]$Platform)
    switch ($Platform) {
        "windows64" { "win64" }
        "windows32" { "win32" }
        "windowsarm64" { "win-arm64" }
        "linux64" { "linux64" }
        "linuxarm64" { "linux-arm64" }
        "linuxarm" { "linux-arm32" }
        default { $Platform }
    }
}

# 创建输出目录
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
New-Item -ItemType Directory -Force -Path "$OutputDir\windows-x64" | Out-Null
New-Item -ItemType Directory -Force -Path "$OutputDir\windows-x86" | Out-Null
New-Item -ItemType Directory -Force -Path "$OutputDir\windows-arm64" | Out-Null
New-Item -ItemType Directory -Force -Path "$OutputDir\linux-x64" | Out-Null
New-Item -ItemType Directory -Force -Path "$OutputDir\linux-arm64" | Out-Null
New-Item -ItemType Directory -Force -Path "$OutputDir\linux-arm32" | Out-Null
New-Item -ItemType Directory -Force -Path "$OutputDir\android" | Out-Null

# =============================================================================
# 1. Windows 构建
# =============================================================================
Write-Host "`n[1/6] 编译 Windows x86_64 (原生)..." -ForegroundColor Green
Write-Host "  目标: $OutputDir\windows-x64\nova-browser.exe"

Download-CefOnce -Platform "windows64"
cargo build --release -p nova-app
Move-Item "target\release\nova-browser.exe" "$OutputDir\windows-x64\" -Force
Write-Host "  [OK] Windows x86_64 完成" -ForegroundColor Yellow

# ---
Write-Host "`n[2/6] 编译 Windows x86 (32位)..." -ForegroundColor Green
Write-Host "  目标: $OutputDir\windows-x86\nova-browser.exe"

# 检查 target 是否已安装
$targets = rustup target list --installed
if ($targets -notmatch "i686-pc-windows-msvc") {
    Write-Host "  安装 i686-pc-windows-msvc target..." -ForegroundColor Gray
    rustup target add i686-pc-windows-msvc
}

Download-CefOnce -Platform "windows32"
cargo build --release -p nova-app --target i686-pc-windows-msvc
Move-Item "target\i686-pc-windows-msvc\release\nova-browser.exe" "$OutputDir\windows-x86\" -Force
Write-Host "  [OK] Windows x86 完成" -ForegroundColor Yellow

# ---
Write-Host "`n[3/6] 编译 Windows ARM64..." -ForegroundColor Green
Write-Host "  目标: $OutputDir\windows-arm64\nova-browser.exe"

if ($targets -notmatch "aarch64-pc-windows-msvc") {
    Write-Host "  安装 aarch64-pc-windows-msvc target..." -ForegroundColor Gray
    rustup target add aarch64-pc-windows-msvc
}

Download-CefOnce -Platform "windowsarm64"
cargo build --release -p nova-app --target aarch64-pc-windows-msvc
Move-Item "target\aarch64-pc-windows-msvc\release\nova-browser.exe" "$OutputDir\windows-arm64\" -Force
Write-Host "  [OK] Windows ARM64 完成" -ForegroundColor Yellow

# =============================================================================
# 2. Linux 构建 (通过 WSL2)
# =============================================================================
if (-not $SkipLinux) {
    Write-Host "`n[4/6] 编译 Linux x86_64 (WSL2)..." -ForegroundColor Green
    Write-Host "  目标: $OutputDir\linux-x64\nova-browser"

    # 把 Windows 路径转成 WSL 路径
    $wslProjectRoot = ($ProjectRoot -replace '\\', '/') -replace '^([A-Z]):', '/mnt/$1'
    $wslProjectRoot = $wslProjectRoot.ToLower()

    $wslOutputDir = ($OutputDir -replace '\\', '/') -replace '^([A-Z]):', '/mnt/$1'
    $wslOutputDir = $wslOutputDir.ToLower()

    # 在 WSL 中编译 Linux x64
    wsl bash -c @"
        set -e
        echo "=== 安装 Rust 工具链 (如果尚未安装) ==="
        if ! command -v cargo &> /dev/null; then
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source \$HOME/.cargo/env
        fi

        echo "=== 安装系统依赖 ==="
        sudo apt update -qq
        sudo apt install -y -qq build-essential pkg-config cmake libssl-dev libgtk-3-dev libx11-dev libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libfontconfig1-dev libfreetype6-dev libglib2.0-dev 2>/dev/null

        cd "$wslProjectRoot"

        if [ -n "$CefVersion" ]; then
            echo "=== 下载 CEF ==="
            bash download-cef.sh linux64 "$CefVersion" || true
        fi

        echo "=== 编译 Linux x86_64 ==="
        cargo build --release -p nova-app
        mkdir -p "$wslOutputDir/linux-x64"
        mv target/release/nova-browser "$wslOutputDir/linux-x64/"
        echo "[OK] Linux x86_64 完成"
"@

    if ($LASTEXITCODE -ne 0) {
        Write-Host "  [警告] Linux x86_64 编译失败，请检查 WSL2 是否已安装" -ForegroundColor Red
    } else {
        Write-Host "  [OK] Linux x86_64 完成" -ForegroundColor Yellow
    }

    # --- Linux ARM64 (交叉编译)
    Write-Host "`n[5/6] 编译 Linux ARM64 (WSL2 交叉编译)..." -ForegroundColor Green
    Write-Host "  目标: $OutputDir\linux-arm64\nova-browser"

    wsl bash -c @"
        set -e
        source \$HOME/.cargo/env 2>/dev/null || true

        echo "=== 安装 ARM64 交叉编译器 ==="
        sudo apt install -y -qq gcc-aarch64-linux-gnu 2>/dev/null

        rustup target add aarch64-unknown-linux-gnu 2>/dev/null || true

        cd "$wslProjectRoot"

        if [ -n "$CefVersion" ]; then
            echo "=== 下载 CEF ARM64 ==="
            bash download-cef.sh linuxarm64 "$CefVersion" || true
        fi

        echo "=== 编译 Linux ARM64 ==="
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
        cargo build --release -p nova-app --target aarch64-unknown-linux-gnu
        mkdir -p "$wslOutputDir/linux-arm64"
        mv target/aarch64-unknown-linux-gnu/release/nova-browser "$wslOutputDir/linux-arm64/"
        echo "[OK] Linux ARM64 完成"
"@

    if ($LASTEXITCODE -ne 0) {
        Write-Host "  [警告] Linux ARM64 编译失败" -ForegroundColor Red
    } else {
        Write-Host "  [OK] Linux ARM64 完成" -ForegroundColor Yellow
    }

    # --- Linux ARM32 (交叉编译)
    Write-Host "`n[6/6] 编译 Linux ARM32 (WSL2 交叉编译)..." -ForegroundColor Green
    Write-Host "  目标: $OutputDir\linux-arm32\nova-browser"

    wsl bash -c @"
        set -e
        source \$HOME/.cargo/env 2>/dev/null || true

        echo "=== 安装 ARM32 交叉编译器 ==="
        sudo apt install -y -qq gcc-arm-linux-gnueabihf 2>/dev/null

        rustup target add armv7-unknown-linux-gnueabihf 2>/dev/null || true

        cd "$wslProjectRoot"

        if [ -n "$CefVersion" ]; then
            echo "=== 下载 CEF ARM32 ==="
            bash download-cef.sh linuxarm "$CefVersion" || true
        fi

        echo "=== 编译 Linux ARM32 ==="
        export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc
        cargo build --release -p nova-app --target armv7-unknown-linux-gnueabihf
        mkdir -p "$wslOutputDir/linux-arm32"
        mv target/armv7-unknown-linux-gnueabihf/release/nova-browser "$wslOutputDir/linux-arm32/"
        echo "[OK] Linux ARM32 完成"
"@

    if ($LASTEXITCODE -ne 0) {
        Write-Host "  [警告] Linux ARM32 编译失败" -ForegroundColor Red
    } else {
        Write-Host "  [OK] Linux ARM32 完成" -ForegroundColor Yellow
    }
}

# =============================================================================
# 3. Android 构建
# =============================================================================
if (-not $SkipAndroid) {
    Write-Host "`n[Android] 编译 Android 原生库..." -ForegroundColor Green
    Write-Host "  目标: $OutputDir\android\jniLibs\"

    # 检查 cargo-ndk
    $ndkInstalled = cargo install --list | Select-String "cargo-ndk"
    if (-not $ndkInstalled) {
        Write-Host "  安装 cargo-ndk..." -ForegroundColor Gray
        cargo install cargo-ndk
    }

    # 安装 Android targets
    $targets = rustup target list --installed
    @("aarch64-linux-android", "armv7-linux-androideabi", "x86_64-linux-android", "i686-linux-android") | ForEach-Object {
        if ($targets -notmatch $_) {
            Write-Host "  安装 $_ target..." -ForegroundColor Gray
            rustup target add $_
        }
    }

    $androidOut = "$OutputDir\android\jniLibs"
    New-Item -ItemType Directory -Force -Path $androidOut | Out-Null

    cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -t x86 -o $androidOut build --release -p nova-android

    if ($LASTEXITCODE -ne 0) {
        Write-Host "  [警告] Android 编译失败，请检查 ANDROID_NDK_HOME 环境变量" -ForegroundColor Red
    } else {
        Write-Host "  [OK] Android 完成" -ForegroundColor Yellow
    }
}

# =============================================================================
# 4. macOS 构建
# =============================================================================
if (-not $SkipMacOS) {
    Write-Host "`n[macOS] macOS 无法在 Windows 上编译" -ForegroundColor Red
    Write-Host "  请使用 GitHub Actions 编译 macOS 目标:" -ForegroundColor Yellow
    Write-Host "  git push 到 GitHub 仓库，CI 会自动编译 macOS 并上传产物" -ForegroundColor Yellow
    Write-Host "  或手动触发: GitHub → Actions → Build All Platforms → Run workflow" -ForegroundColor Yellow
}

# =============================================================================
# 5. 汇总
# =============================================================================
Write-Host "`n============================================" -ForegroundColor Cyan
Write-Host "  构建完成！产物目录: $OutputDir" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

Get-ChildItem -Path $OutputDir -Recurse -File | ForEach-Object {
    $size = "{0,10:N0} KB" -f ($_.Length / 1KB)
    Write-Host "  $size  $($_.FullName.Replace($OutputDir, ''))"
}

Write-Host ""
Write-Host "产物结构:" -ForegroundColor Cyan
Write-Host "  $OutputDir\" -ForegroundColor White
Write-Host "  ├── windows-x64\    nova-browser.exe" -ForegroundColor White
Write-Host "  ├── windows-x86\    nova-browser.exe" -ForegroundColor White
Write-Host "  ├── windows-arm64\  nova-browser.exe" -ForegroundColor White
Write-Host "  ├── linux-x64\      nova-browser" -ForegroundColor White
Write-Host "  ├── linux-arm64\    nova-browser" -ForegroundColor White
Write-Host "  ├── linux-arm32\    nova-browser" -ForegroundColor White
Write-Host "  ├── android\        jniLibs\ (arm64-v8a, armeabi-v7a, x86_64, x86)" -ForegroundColor White
Write-Host "  ├── macos-x64\      (需要 GitHub Actions)" -ForegroundColor DarkGray
Write-Host "  └── macos-arm64\    (需要 GitHub Actions)" -ForegroundColor DarkGray