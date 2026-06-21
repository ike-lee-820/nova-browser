fn main() {
    // Detect target platform for CEF binary selection
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    let cef_dir = match (target_os.as_str(), target_arch.as_str()) {
        ("windows", "x86_64") => "cef/win64",
        ("windows", "x86") => "cef/win32",
        ("windows", "aarch64") => "cef/win-arm64",
        ("linux", "x86_64") => "cef/linux64",
        ("linux", "aarch64") => "cef/linux-arm64",
        ("linux", "arm") => "cef/linux-arm32",
        ("macos", "aarch64") => "cef/macos-arm64",
        ("macos", "x86_64") => "cef/macos64",
        _ => {
            println!("cargo:warning=Unsupported platform: {}-{}", target_os, target_arch);
            return;
        }
    };

    let cef_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(cef_dir);

    // Determine which build subdirectory to use (prefer Release, fallback to Debug)
    let build_type = if cef_path.join("Release").exists() {
        "Release"
    } else if cef_path.join("Debug").exists() {
        "Debug"
    } else {
        ""
    };

    if cef_path.exists() && !build_type.is_empty() {
        let binary_dir = cef_path.join(build_type);
        println!("cargo:rustc-env=CEF_PATH={}", cef_path.display());
        println!("cargo:rustc-env=CEF_BINARY_DIR={}", binary_dir.display());
        println!("cargo:warning=Using CEF binaries from: {}", binary_dir.display());

        // Set library search paths
        if target_os == "macos" {
            // macOS: framework is inside Debug/ or Release/
            let framework_dir = binary_dir.join("Chromium Embedded Framework.framework");
            let libs_dir = framework_dir.join("Libraries");
            println!("cargo:rustc-link-search=framework={}", binary_dir.display());
            println!("cargo:rustc-link-search=native={}", libs_dir.display());
            println!("cargo:warning=CEF framework: {}", framework_dir.display());
        } else {
            // Linux/Windows: binaries directly in Debug/ or Release/
            println!("cargo:rustc-link-search=native={}", binary_dir.display());
        }
    } else if cef_path.exists() {
        // Legacy: no Debug/Release subdirs, assume flat layout
        println!("cargo:rustc-env=CEF_PATH={}", cef_path.display());
        println!("cargo:rustc-env=CEF_BINARY_DIR={}", cef_path.display());
        println!("cargo:warning=Using CEF binaries from: {}", cef_path.display());
        println!("cargo:rustc-link-search=native={}", cef_path.display());
        if target_os == "macos" {
            let framework_dir = cef_path.join("Chromium Embedded Framework.framework");
            if framework_dir.exists() {
                let libs_dir = framework_dir.join("Libraries");
                println!("cargo:rustc-link-search=framework={}", cef_path.display());
                println!("cargo:rustc-link-search=native={}", libs_dir.display());
            }
        }
    } else {
        println!("cargo:warning=CEF binaries not found at: {}", cef_path.display());
        println!("cargo:warning=Please place CEF Standard Distribution in the appropriate cef/ directory.");
        println!("cargo:warning=Expected directory: {}", cef_path.display());
        println!("cargo:warning=Download from: https://cef-builds.spotifycdn.com/index.html");
    }

    // Set V8 snapshot name based on platform
    let v8_snapshot = if target_os == "macos" {
        match target_arch.as_str() {
            "aarch64" => "v8_context_snapshot.arm64.bin",
            "x86_64" => "v8_context_snapshot.x86_64.bin",
            _ => "v8_context_snapshot.bin",
        }
    } else {
        "v8_context_snapshot.bin"
    };
    println!("cargo:rustc-env=CEF_V8_SNAPSHOT={}", v8_snapshot);

    // Set locale resource path format
    if target_os == "macos" {
        // macOS uses .lproj/locale.pak format
        println!("cargo:rustc-env=CEF_LOCALE_FORMAT=lproj");
    } else {
        // Linux/Windows use locales/xx.pak format
        println!("cargo:rustc-env=CEF_LOCALE_FORMAT=locales");
    }

    // Set resources directory path
    if target_os == "macos" {
        // macOS: resources are inside the framework
        println!("cargo:rustc-env=CEF_RESOURCES_DIR=Chromium Embedded Framework.framework/Resources");
    } else {
        // Linux/Windows: resources are in Resources/ relative to cef root
        println!("cargo:rustc-env=CEF_RESOURCES_DIR=Resources");
    }

    // Embed resources
    let resources_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("resources");
    if resources_dir.exists() {
        println!("cargo:rerun-if-changed=resources/");
    }

    // Set CEF-related environment variables
    if let Ok(cef_path_env) = std::env::var("CEF_PATH") {
        println!("cargo:rustc-env=CEF_PATH={}", cef_path_env);
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CEF_PATH");
}