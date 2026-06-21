plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.compose)
}

android {
    namespace = "com.novabrowser"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.novabrowser"
        minSdk = 26
        targetSdk = 34
        versionCode = 1
        versionName = "0.1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        ndk {
            abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64", "x86")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
        debug {
            isMinifyEnabled = false
            applicationIdSuffix = ".debug"
            versionNameSuffix = "-debug"
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    buildFeatures {
        compose = true
    }

    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }
}

dependencies {
    // GeckoView browser engine — 优先使用本地 AAR，Maven 作为 fallback
    // 将你的 .aar 文件放到 android/app/libs/ 目录下
    implementation(fileTree("libs") { include("*.aar") })
    // 如果没有本地 AAR，取消下面这行的注释以从 Maven 下载：
    // implementation(libs.geckoview)

    // Compose
    implementation(platform(libs.compose.bom))
    implementation(libs.compose.ui)
    implementation(libs.compose.ui.graphics)
    implementation(libs.compose.ui.tooling.preview)
    implementation(libs.compose.material3)
    implementation(libs.compose.material.icons)
    debugImplementation(libs.compose.ui.tooling)

    // AndroidX
    implementation(libs.activity.compose)
    implementation(libs.core.ktx)
    implementation(libs.lifecycle.runtime)
    implementation(libs.lifecycle.viewmodel)
    implementation(libs.navigation.compose)

    // Coroutines
    implementation(libs.coroutines)
}

// Task to copy Rust .so files into jniLibs
tasks.register<Copy>("copyRustLibs") {
    from("../../target")
    include("aarch64-linux-android/release/libnova_android.so")
    include("armv7-linux-androideabi/release/libnova_android.so")
    include("x86_64-linux-android/release/libnova_android.so")
    include("i686-linux-android/release/libnova_android.so")

    into("src/main/jniLibs")
    rename { fileName ->
        when {
            fileName.contains("aarch64") -> "arm64-v8a/libnova_android.so"
            fileName.contains("armv7") -> "armeabi-v7a/libnova_android.so"
            fileName.contains("x86_64") -> "x86_64/libnova_android.so"
            fileName.contains("i686") -> "x86/libnova_android.so"
            else -> fileName
        }
    }
    dependsOn(":buildRustAndroid")
}

tasks.register<Exec>("buildRustAndroid") {
    workingDir = file("../../")
    commandLine(
        "cargo", "ndk",
        "-t", "arm64-v8a",
        "-t", "armeabi-v7a",
        "-t", "x86_64",
        "-t", "x86",
        "-o", "android/app/src/main/jniLibs",
        "build", "--release",
        "-p", "nova-android"
    )
}