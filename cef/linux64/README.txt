# Linux x86_64 CEF 目录

请将 CEF Linux x86_64 构建放入此目录。

## 目录结构

CEF Linux 构建是完整的源码分发包，包含头文件、源代码和预编译二进制。

```
linux64/
├── CMakeLists.txt / BUILD.bazel / WORKSPACE
├── .bazelrc / .bazelversion / MODULE.bazel
├── cef_paths.gypi / cef_paths2.gypi
├── CREDITS.html / Doxyfile / LICENSE.txt / README.md / README.txt
├── bazel/ / cmake/                  ← 构建脚本
├── include/                         ← C/C++ 头文件（含 base/ capi/ internal/ views/ wrapper/）
├── libcef_dll/                      ← libcef_dll_wrapper 源码
├── Debug/                           ← Debug 二进制 ⚠️ 核心
│   ├── libcef.so                    ← CEF 主库
│   ├── chrome-sandbox               ← 沙箱（需 SUID）
│   ├── libEGL.so / libGLESv2.so
│   ├── libvk_swiftshader.so / libvulkan.so.1
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

## 获取 CEF Linux x86_64

从 https://cef-builds.spotifycdn.com/index.html 搜索：
- 平台: `linux64`
- 推荐版本: CEF 120+

下载 Standard Distribution 解压后，将整个目录内容复制到此。

## 注意事项

运行时需要设置库路径（指向 Debug/ 或 Release/）：
```bash
export LD_LIBRARY_PATH=$(pwd)/cef/linux64/Release:$LD_LIBRARY_PATH
```

`chrome-sandbox` 需要设置 SUID 权限：
```bash
sudo chown root:root cef/linux64/Release/chrome-sandbox
sudo chmod 4755 cef/linux64/Release/chrome-sandbox
```