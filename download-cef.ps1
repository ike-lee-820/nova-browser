# CEF 自动下载脚本 (Windows)
#
# 用法:
#   .\download-cef.ps1                     # 下载当前平台
#   .\download-cef.ps1 -Platform windows64  # 下载指定平台
#   .\download-cef.ps1 -Platform linux64 -Version 120.1.10+g3ce3184+chromium-120.0.6099.129
#
# 如果不指定版本，自动从 Spotify CDN 获取最新列表

param(
    [string]$Platform = "",
    [string]$Version = "149.0.4+g2f1bfd8+chromium-149.0.7827.156"
)

$ErrorActionPreference = "Continue"
$ProjectRoot = $PSScriptRoot
$CefDir = "$ProjectRoot\cef"

# 自动检测当前平台
if (-not $Platform) {
    $os = if ($IsWindows) { "windows" } elseif ($IsLinux) { "linux" } elseif ($IsMacOS) { "macos" } else { "unknown" }
    $arch = if ([Environment]::Is64BitOperatingSystem) {
        if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "arm64" } else { "x64" }
    } else { "x86" }
    $Platform = switch ("$os-$arch") {
        "windows-x64" { "windows64" }
        "windows-x86" { "windows32" }
        "windows-arm64" { "windowsarm64" }
        "linux-x64" { "linux64" }
        "linux-arm64" { "linuxarm64" }
        "linux-arm" { "linuxarm" }
        "macos-x64" { "macosx64" }
        "macos-arm64" { "macosarm64" }
        default { $null }
    }
    if (-not $Platform) {
        Write-Host "无法检测当前平台，请手动指定 -Platform" -ForegroundColor Red
        exit 1
    }
    Write-Host "检测到平台: $Platform" -ForegroundColor Cyan
}

# 平台 → 目录映射
$platformDir = switch ($Platform) {
    "windows64" { "win64" }
    "windows32" { "win32" }
    "windowsarm64" { "win-arm64" }
    "linux64" { "linux64" }
    "linuxarm64" { "linux-arm64" }
    "linuxarm" { "linux-arm32" }
    "macosx64" { "macos64" }
    "macosarm64" { "macos-arm64" }
    default { $null }
}
if (-not $platformDir) {
    Write-Host "不支持的平台: $Platform" -ForegroundColor Red
    exit 1
}

$destDir = "$CefDir\$platformDir"

# 检查是否已存在
if ((Test-Path "$destDir\Release") -or (Test-Path "$destDir\Debug")) {
    Write-Host "[跳过] CEF 已存在于 $destDir" -ForegroundColor Yellow
    exit 0
}

# 构建下载 URL
# 所有平台统一使用 tar.bz2 格式
$ext = "tar.bz2"
# URL 编码版本号中的 + 号
$encodedVersion = $Version -replace '\+', '%2B'
$filename = "cef_binary_${encodedVersion}_${Platform}.${ext}"
$url = "https://cef-builds.spotifycdn.com/$filename"

Write-Host "下载 CEF $Platform..." -ForegroundColor Green
Write-Host "  URL: $url" -ForegroundColor Gray
Write-Host "  目标: $destDir" -ForegroundColor Gray

New-Item -ItemType Directory -Force -Path $destDir | Out-Null

$downloadPath = "$CefDir\$filename"

try {
    # 下载
    Write-Host "  下载中 (约 500MB-1.5GB，请耐心等待)..." -ForegroundColor Gray
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $url -OutFile $downloadPath -ErrorAction Stop

    # 解压 (tar.bz2)
    Write-Host "  解压中..." -ForegroundColor Gray
    if (Get-Command "7z" -ErrorAction SilentlyContinue) {
        & 7z x $downloadPath -o"$CefDir\_temp" -y | Out-Null
    } else {
        # Windows 10 1803+ 自带 tar
        tar -xf $downloadPath -C "$CefDir\_temp"
    }

    # 移动解压内容到平台目录
    # CEF 包解压后通常是一个 cef_binary_... 目录
    $extractedDir = Get-ChildItem -Path "$CefDir\_temp" -Directory | Select-Object -First 1
    if ($extractedDir) {
        Get-ChildItem -Path $extractedDir.FullName | ForEach-Object {
            Move-Item -Path $_.FullName -Destination "$destDir\" -Force
        }
    }

    # 清理
    Remove-Item -Recurse -Force "$CefDir\_temp" -ErrorAction SilentlyContinue
    Remove-Item -Force $downloadPath -ErrorAction SilentlyContinue

    Write-Host "  [OK] CEF $Platform 下载完成 → $destDir" -ForegroundColor Yellow
} catch {
    Write-Host "  [失败] 下载 CEF 失败: $_" -ForegroundColor Red
    Write-Host "  你可以手动下载: $url" -ForegroundColor Yellow
    Write-Host "  解压到: $destDir" -ForegroundColor Yellow
    Remove-Item -Force $downloadPath -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force "$CefDir\_temp" -ErrorAction SilentlyContinue
}