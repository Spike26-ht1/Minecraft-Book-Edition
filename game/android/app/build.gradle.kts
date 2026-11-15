plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "dev.bookedition.game"
    compileSdk = 35

    defaultConfig {
        applicationId = "dev.bookedition.game"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0-dev.1"

        ndk {
            abiFilters += "arm64-v8a"
        }
    }

    sourceSets.getByName("main").jniLibs.srcDirs("src/main/jniLibs")

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }
}
