# macOS Intel (x86_64) CEF 目录

请将 CEF macOS x86_64 构建放入此目录。

## 目录结构

> **重要**：macOS 的 CEF 框架位于 `Debug/` 和 `Release/` 子目录内，与 Linux/Windows 的扁平结构不同。

```
macos64/
├── CMakeLists.txt / BUILD.bazel / WORKSPACE
├── .bazelrc / .bazelversion / MODULE.bazel
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
│           ├── gpu_shader_cache.bin
│           ├── icudtl.dat
│           ├── Info.plist
│           ├── resources.pak
│           ├── v8_context_snapshot.x86_64.bin  ← x86_64 专用
│           ├── en.lproj/locale.pak
│           ├── zh_CN.lproj/locale.pak
│           ├── zh_TW.lproj/locale.pak
│           ├── ja.lproj/locale.pak
│           └── ... (其他语言 .lproj 目录)
├── Release/
│   └── Chromium Embedded Framework.framework/   ← ⚠️ 核心
│       └── (同上)
└── tests/                           ← 测试代码（可忽略）
```

## 获取 CEF macOS x86_64

从 https://cef-builds.spotifycdn.com/index.html 搜索：
- 平台: `macosx64`
- 推荐版本: CEF 120+

下载 Standard Distribution 解压后，将整个目录内容复制到此。

## 注意事项

- Framework 在 `Debug/` 和 `Release/` 子目录内，不是根目录
- 编译时需链接 `Debug/Chromium Embedded Framework.framework/Chromium Embedded Framework`
- 运行时需设置 `DYLD_FRAMEWORK_PATH` 或嵌入到 .app bundle
- V8 快照命名为 `v8_context_snapshot.x86_64.bin`（非 `v8_context_snapshot.bin`）
- 多语言资源使用 `.lproj/locale.pak` 格式（非 `locales/` 目录）