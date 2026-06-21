# Nova Browser — 编译指南

## 前提条件

### 1. 安装 Rust 工具链

需要 Rust **1.75.0 或更高版本**（推荐 1.80+）：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

验证安装：

```bash
rustc --version
cargo --version
```

### 2. 安装系统依赖

**Linux (Debian/Ubuntu)**：
```bash
sudo apt update
sudo apt install -y build-essential pkg-config cmake libssl-dev \
  libgtk-3-dev libx11-dev libxcb1-dev libxcb-render0-dev \
  libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev \
  libfontconfig1-dev libfreetype6-dev libglib2.0-dev
```

**Linux (Fedora/RHEL)**：
```bash
sudo dnf install -y gcc-c++ cmake openssl-devel pkgconfig \
  gtk3-devel libX11-devel libxcb-devel fontconfig-devel \
  freetype-devel glib2-devel
```

**macOS**：
```bash
xcode-select --install
brew install cmake pkg-config
```

**Windows**：
- 安装 [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)（含 C++ 工具链）
- 或安装 [MSYS2](https://www.msys2.org/) 并运行 `pacman -S mingw-w64-x86_64-toolchain cmake`

### 3. 获取 CEF 二进制文件

> **CEF 不纳入 Git 仓库**（单个平台 500MB-2GB，8 个平台远超 GitHub 限制），
> 改为构建时自动下载或手动放置。

#### 方式 A：自动下载（推荐）

运行项目的下载脚本，自动从 CEF 官方 CDN 下载当前平台的 CEF：

```bash
# Windows (PowerShell)
.\download-cef.ps1

# Linux / macOS
./download-cef.sh
```

指定平台和版本：

```bash
# Windows 下载 Linux 版 CEF（交叉编译时用）
.\download-cef.ps1 -Platform linux64 -Version "149.0.4+g2f1bfd8+chromium-149.0.7827.156"

# bash
./download-cef.sh linux64 "149.0.4+g2f1bfd8+chromium-149.0.7827.156"
```

下载后 CEF 会自动解压到 `cef/<平台目录>/` 下面。

#### 方式 B：一键构建时自动下载

```powershell
# 构建时指定 CEF 版本，自动下载所有平台的 CEF
.\build-all.ps1 -OutputDir "L:\BBD" -CefVersion "149.0.4+g2f1bfd8+chromium-149.0.7827.156"
```

#### 方式 C：手动下载放置

从 [CEF 官方下载页](https://cef-builds.spotifycdn.com/index.html) 下载 Standard Distribution，解压到对应平台目录。

#### 目录总览

```
nova-browser/
└── cef/
    ├── win64/          # Windows x86_64
    ├── win32/          # Windows x86 (32位)
    ├── win-arm64/      # Windows ARM64
    ├── linux64/        # Linux x86_64
    ├── linux-arm64/    # Linux ARM64 (树莓派5等)
    ├── linux-arm32/    # Linux ARM32 (树莓派3/4)
    ├── macos-arm64/    # macOS Apple Silicon (M1/M2/M3)
    └── macos64/        # macOS Intel (x86_64)
```

#### Linux (linux64 / linux-arm64 / linux-arm32) 文件结构

CEF Linux 构建是完整的源码分发包，包含头文件、CMake 模块和预编译二进制。**二进制文件位于 `Debug/` 和 `Release/` 子目录中，资源文件位于 `Resources/`。**

```
cef/linux64/
├── CMakeLists.txt / BUILD.bazel     ← 构建文件
├── bazel/ / cmake/                  ← 构建脚本
├── include/                         ← C/C++ 头文件
├── libcef_dll/                      ← libcef_dll_wrapper 源码
├── Debug/                           ← Debug 二进制 ⚠️ 核心
│   ├── libcef.so                    ← CEF 主库
│   ├── chrome-sandbox               ← 沙箱（需 SUID）
│   ├── libEGL.so
│   ├── libGLESv2.so
│   ├── libvk_swiftshader.so
│   ├── libvulkan.so.1
│   ├── v8_context_snapshot.bin
│   └── vk_swiftshader_icd.json
├── Release/                         ← Release 二进制 ⚠️ 核心
│   └── (同上)
├── Resources/                       ← 资源文件
│   ├── chrome_100_percent.pak
│   ├── chrome_200_percent.pak
│   ├── icudtl.dat
│   ├── resources.pak
│   └── locales/                     ← 多语言 .pak
└── tests/                           ← 测试代码（可忽略）
```

#### Windows (win64 / win32 / win-arm64) 文件结构

CEF Windows 构建同样是完整的源码分发包，**二进制文件在 `Debug/` 和 `Release/` 中，资源文件在 `Resources/`。**

```
cef/win64/
├── CMakeLists.txt / BUILD.bazel / WORKSPACE
├── bazel/ / cmake/                  ← 构建脚本
├── include/                         ← C/C++ 头文件
├── libcef_dll/                      ← libcef_dll_wrapper 源码
├── Debug/                           ← Debug 二进制 ⚠️ 核心
│   ├── libcef.dll                   ← CEF 主库
│   ├── chrome_elf.dll               ← Chrome ELF 模块
│   ├── libEGL.dll / libGLESv2.dll
│   ├── vk_swiftshader.dll / vulkan-1.dll
│   ├── d3dcompiler_47.dll
│   ├── dxcompiler.dll / dxil.dll    ← 仅 win64 有此文件
│   ├── libcef.lib                   ← 导入库（编译用）
│   ├── bootstrap.exe / bootstrapc.exe  ← CEF 子进程
│   ├── v8_context_snapshot.bin
│   └── vk_swiftshader_icd.json
├── Release/                         ← Release 二进制 ⚠️ 核心
│   └── (同上)
├── Resources/                       ← 资源文件
│   ├── chrome_100_percent.pak
│   ├── chrome_200_percent.pak
│   ├── icudtl.dat
│   ├── resources.pak
│   └── locales/                     ← 多语言 .pak（含 _FEMININE/_MASCULINE/_NEUTER）
└── tests/                           ← 测试代码（可忽略）
```

> **注意**：win32 版本**没有** `dxcompiler.dll` 和 `dxil.dll`（仅 win64 有）。

#### macOS (macos-arm64 / macos64) 文件结构

> **重要**：macOS 的 CEF framework 位于 `Debug/` 和 `Release/` 子目录内，不是在根目录！V8 快照和语言资源格式也与 Linux/Windows 不同。

```
cef/macos-arm64/
├── CMakeLists.txt / BUILD.bazel / WORKSPACE
├── bazel/ / cmake/                  ← 构建脚本
├── include/                         ← C/C++ 头文件
├── libcef_dll/                      ← libcef_dll_wrapper 源码
├── Debug/
│   └── Chromium Embedded Framework.framework/   ← ⚠️ 核心
│       ├── Chromium Embedded Framework          ← 主二进制
│       ├── Libraries/
│       │   ├── libcef_sandbox.dylib
│       │   ├── libEGL.dylib
│       │   ├── libGLESv2.dylib
│       │   ├── libvk_swiftshader.dylib
│       │   └── vk_swiftshader_icd.json
│       └── Resources/
│           ├── chrome_100_percent.pak
│           ├── chrome_200_percent.pak
│           ├── icudtl.dat
│           ├── Info.plist
│           ├── resources.pak
│           ├── v8_context_snapshot.arm64.bin   ← ARM64 专用
│           ├── en.lproj/locale.pak             ← 语言格式
│           ├── zh_CN.lproj/locale.pak
│           └── ... (其他语言 .lproj 目录)
├── Release/
│   └── Chromium Embedded Framework.framework/   ← ⚠️ 核心
│       └── (同上)
└── tests/                           ← 测试代码（可忽略）
```

> **macOS 关键差异**：
> - Framework 在 `Debug/`/`Release/` 子目录内，不是根目录
> - V8 快照命名带架构后缀：`v8_context_snapshot.arm64.bin` / `v8_context_snapshot.x86_64.bin`
> - 多语言资源使用 `.lproj/locale.pak` 格式（而非 `locales/` 目录）

---

## 编译步骤

### 第一步：进入项目目录

```bash
cd nova-browser
```

### 第二步：编译所有 crate（推荐）

```bash
cargo build --release
```

生成的可执行文件位于：
- **Linux/macOS**：`target/release/nova-browser`
- **Windows**：`target/release/nova-browser.exe`

### 第三步（可选）：逐个编译调试

如果遇到问题，可以逐个 crate 编译定位错误：

```bash
# 1. 核心库
cargo build -p nova-core

# 2. 功能库（书签、历史、扩展、密码、广告拦截、阅读模式、设置等）
cargo build -p nova-features

# 3. UI 库（标签栏、地址栏、侧边栏、设置界面等）
cargo build -p nova-ui

# 4. 主程序
cargo build -p nova-app
```

### 第四步：运行

```bash
cargo run --release
```

或者直接运行编译产物：

```bash
./target/release/nova-browser
```

---

## Windows x64 — 一键编译全平台

如果你只有一台 Windows x64 电脑，可以用项目自带的 `build-all.ps1` 脚本一键编译所有可编译的目标：

### 前提条件

1. **Rust** — 已安装 rustup
2. **Visual Studio Build Tools** — 含 C++ 工具链、x86 工具链、ARM64 工具链
3. **WSL2** — 用于编译 Linux 目标（在 PowerShell 中运行 `wsl --install`）
4. **Android NDK** — 用于编译 Android 目标（可选，设置 `ANDROID_NDK_HOME` 环境变量）
5. **cargo-ndk** — `cargo install cargo-ndk`

### 一键构建

```powershell
# 输出到 L:\BBD
.\build-all.ps1 -OutputDir "L:\BBD"

# 跳过 Android 编译
.\build-all.ps1 -OutputDir "L:\BBD" -SkipAndroid

# 跳过 Linux 编译（如果没有 WSL2）
.\build-all.ps1 -OutputDir "L:\BBD" -SkipLinux
```

### 产物结构

```
L:\BBD\
├── windows-x64\    nova-browser.exe     ← 你的 Windows 原生
├── windows-x86\    nova-browser.exe     ← 32位 Windows
├── windows-arm64\  nova-browser.exe     ← ARM64 Windows
├── linux-x64\      nova-browser         ← WSL2 编译
├── linux-arm64\    nova-browser         ← WSL2 交叉编译
├── linux-arm32\    nova-browser         ← WSL2 交叉编译
├── android\        jniLibs\             ← cargo-ndk 编译
│   ├── arm64-v8a\  libnova_android.so
│   ├── armeabi-v7a\ libnova_android.so
│   ├── x86_64\     libnova_android.so
│   └── x86\        libnova_android.so
├── macos-x64\      (需要 GitHub Actions)
└── macos-arm64\    (需要 GitHub Actions)
```

### 各目标可行性

| 目标 | 能否在 Windows 编译 | 方式 |
|------|-------------------|------|
| Windows x86_64 | 原生 | 你的 Windows 电脑 |
| Windows x86 (32位) | 安装 x86 工具链即可 | VS Installer → 勾选 x86 工具链 |
| Windows ARM64 | 安装 ARM64 工具链即可 | VS Installer → 勾选 ARM64 工具链 |
| Linux x86_64 | WSL2 | `wsl --install` 后脚本自动处理 |
| Linux ARM64 | WSL2 + 交叉编译器 | 脚本自动安装 `gcc-aarch64-linux-gnu` |
| Linux ARM32 | WSL2 + 交叉编译器 | 脚本自动安装 `gcc-arm-linux-gnueabihf` |
| Android (4架构) | cargo-ndk + Android NDK | 设置 `ANDROID_NDK_HOME` |
| macOS x86_64 | 无法编译 | 需要 Mac 或 GitHub Actions |
| macOS ARM64 | 无法编译 | 需要 Mac 或 GitHub Actions |

### 手动编译各目标

如果不使用脚本，手动编译：

```powershell
# Windows x86_64
cargo build --release -p nova-app
copy target\release\nova-browser.exe L:\BBD\windows-x64\

# Windows x86 (32位)
rustup target add i686-pc-windows-msvc
cargo build --release -p nova-app --target i686-pc-windows-msvc
copy target\i686-pc-windows-msvc\release\nova-browser.exe L:\BBD\windows-x86\

# Windows ARM64
rustup target add aarch64-pc-windows-msvc
cargo build --release -p nova-app --target aarch64-pc-windows-msvc
copy target\aarch64-pc-windows-msvc\release\nova-browser.exe L:\BBD\windows-arm64\

# Linux (在 WSL2 中)
wsl
cd /mnt/你的项目路径
cargo build --release -p nova-app
cp target/release/nova-browser /mnt/l/BBD/linux-x64/

# Android
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -t x86 -o L:\BBD\android\jniLibs build --release -p nova-android
```

---

## 项目结构概览

```
nova-browser/
├── crates/
│   ├── nova-core/       # 核心库：配置、窗口管理、CEF 桥接
│   ├── nova-features/   # 功能库：书签、历史、扩展、密码、广告拦截、阅读模式、设置
│   ├── nova-ui/         # UI 库：标签栏、地址栏、侧边栏、设置界面、主题
│   ├── nova-app/        # 主程序入口（桌面端）
│   └── nova-android/    # Android JNI 原生库（GeckoView 桥接）
├── android/             # Android 项目（Gradle + Kotlin + Jetpack Compose）
│   └── app/
│       └── src/main/
│           ├── java/com/novabrowser/
│           │   ├── MainActivity.kt
│           │   ├── NovaApplication.kt
│           │   ├── browser/GeckoViewEngine.kt
│           │   ├── ui/MainScreen.kt
│           │   └── bridge/RustBridge.kt
│           └── jniLibs/    # Rust 编译的 .so 文件（cargo-ndk 输出）
├── resources/
│   ├── locales/         # 多语言翻译（16种语言）
│   └── themes/          # 主题配置（亮色/暗色）
├── cef/                 # CEF 二进制文件（桌面端，需自行放置）
├── .github/workflows/   # GitHub Actions CI 配置
├── build.rs             # 构建脚本（平台检测）
├── Cargo.toml           # 工作区配置
└── BUILD.md             # 本文件
```

---

## Android 编译

### 前提条件

1. **Android Studio**（推荐 Hedgehog 2023.1+）
2. **Android SDK**（API 34+）
3. **Android NDK**（26+）
4. **Rust 交叉编译工具链**：
   ```bash
   rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
   ```
5. **cargo-ndk**：
   ```bash
   cargo install cargo-ndk
   ```

### 编译 Rust 原生库

```bash
# 编译所有 Android 架构的 .so 文件
cargo ndk \
  -t arm64-v8a \
  -t armeabi-v7a \
  -t x86_64 \
  -t x86 \
  -o android/app/src/main/jniLibs \
  build --release \
  -p nova-android
```

输出会放到 `android/app/src/main/jniLibs/` 下：
```
jniLibs/
├── arm64-v8a/libnova_android.so
├── armeabi-v7a/libnova_android.so
├── x86_64/libnova_android.so
└── x86/libnova_android.so
```

### 编译 Android APK

用 Android Studio 打开 `android/` 目录，或使用命令行：

```bash
cd android
./gradlew assembleDebug    # Debug APK
./gradlew assembleRelease  # Release APK（需签名配置）
```

### GeckoView 引擎

Android 版本使用 **GeckoView**（Firefox 引擎）替代 CEF。GeckoView AAR 会自动从 Mozilla Maven 仓库下载，无需手动配置。

> 使用的 GeckoView 版本：`152.0.20260617213557`

---

## 多语言支持

项目内置 16 种语言翻译：

| 代码 | 语言 | 代码 | 语言 |
|------|------|------|------|
| zh-CN | 简体中文 | zh-TW | 繁體中文 |
| en | English | ja | 日本語 |
| ko | 한국어 | fr | Français |
| de | Deutsch | es | Español |
| pt | Português | ru | Русский |
| ar | العربية | hi | हिन्दी |
| it | Italiano | nl | Nederlands |
| tr | Türkçe | vi | Tiếng Việt |
| th | ไทย | | |

语言文件位于 `resources/locales/`，可通过设置界面切换语言，程序会自动检测系统语言。

---

## 功能特性一览

- **多标签页浏览**：支持标签页分组、拖动排序、关闭恢复
- **地址栏搜索**：支持搜索建议、历史记录、搜索引擎关键词触发
- **侧边栏**：书签、历史、阅读列表、下载管理、AI 助手
- **隐私模式**：无痕窗口，不保存浏览记录
- **广告拦截**：内置过滤规则，支持自定义规则
- **密码管理**：AES-256-GCM 加密存储，自动填充
- **阅读模式**：去广告、调整字体/背景色
- **扩展支持**：支持 CRX 格式扩展安装
- **多语言**：16 种界面语言
- **主题切换**：亮色/暗色/跟随系统
- **开发者工具**：元素检查、控制台、网络监控

---

## 故障排除

### 编译错误：`edition 2024 is not supported`

如果系统 Rust 版本低于 1.85，某些依赖可能要求 edition 2024。本项目已将所有依赖锁定到兼容 Rust 1.75 的版本。如果仍有问题，请升级 Rust：

```bash
rustup update stable
```

### 找不到 `libcef.so` / `libcef.dll`

确保已将 CEF 二进制文件放到正确的 `cef/<平台>/` 目录下，并在运行时设置 `LD_LIBRARY_PATH`（Linux）：

```bash
export LD_LIBRARY_PATH=$(pwd)/cef/linux64:$LD_LIBRARY_PATH
./target/release/nova-browser
```

### macOS 安全提示

如果 macOS 提示无法验证开发者，请前往 **系统偏好设置 → 安全性与隐私 → 通用** 中允许运行。

### 内存不足

编译 Release 版本需要较多内存。如果内存不足，可以：
- 使用 `cargo build`（不带 `--release`）进行调试编译
- 减少并行编译线程：`CARGO_BUILD_JOBS=2 cargo build --release`