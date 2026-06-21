# Windows ARM64 CEF 目录

请将 CEF Windows ARM64 构建放入此目录。适用于 Surface Pro X、骁龙笔记本等 ARM64 Windows 设备。

## 目录结构

CEF Windows 构建是完整的源码分发包，包含头文件、源代码和预编译二进制。

```
win-arm64/
├── CMakeLists.txt / BUILD.bazel / WORKSPACE
├── .bazelrc / .bazelversion / MODULE.bazel
├── bazel/ / cmake/                  ← 构建脚本
├── include/                         ← C/C++ 头文件
├── libcef_dll/                      ← libcef_dll_wrapper 源码
├── Debug/                           ← Debug 二进制 ⚠️ 核心
│   ├── libcef.dll                   ← CEF 主库
│   ├── chrome_elf.dll               ← Chrome ELF 模块
│   ├── libEGL.dll / libGLESv2.dll
│   ├── vk_swiftshader.dll / vulkan-1.dll
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
│   └── locales/                     ← 多语言 .pak
└── tests/                           ← 测试代码（可忽略）
```

## 获取 CEF Windows ARM64

从 https://cef-builds.spotifycdn.com/index.html 搜索：
- 平台: `windowsarm64`
- 推荐版本: CEF 120+