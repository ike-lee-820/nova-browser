# Windows x86_64 CEF 目录

请将 CEF Windows 64位 构建放入此目录。

## 目录结构

CEF Windows 构建是完整的源码分发包，包含头文件、源代码和预编译二进制。

```
win64/
├── CMakeLists.txt / BUILD.bazel / WORKSPACE
├── .bazelrc / .bazelversion / MODULE.bazel
├── cef_paths.gypi / cef_paths2.gypi
├── bazel/ / cmake/                  ← 构建脚本
├── include/                         ← C/C++ 头文件
├── libcef_dll/                      ← libcef_dll_wrapper 源码
├── Debug/                           ← Debug 二进制 ⚠️ 核心
│   ├── libcef.dll                   ← CEF 主库
│   ├── chrome_elf.dll               ← Chrome ELF 模块
│   ├── libEGL.dll / libGLESv2.dll
│   ├── vk_swiftshader.dll / vulkan-1.dll
│   ├── d3dcompiler_47.dll
│   ├── dxcompiler.dll / dxil.dll    ← DirectX Shader 编译器（仅 win64）
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

## 获取 CEF Windows 64位

从 https://cef-builds.spotifycdn.com/index.html 搜索：
- 平台: `windows64`
- 推荐版本: CEF 120+

下载 Standard Distribution 解压后，将整个目录内容复制到此。

## 注意事项

- 运行本程序时，需要将 `Debug/` 或 `Release/` 目录加入 PATH
- `libcef.lib` 是编译时链接用的导入库
- `bootstrap.exe` 和 `bootstrapc.exe` 是 CEF 子进程辅助程序，运行时必须存在