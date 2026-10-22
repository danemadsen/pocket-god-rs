plugins {
    id("com.android.application")
}

android {
    namespace = "com.boltcreative.pocketgod.port"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.boltcreative.pocketgod.port"
        // AAudio (Bevy's audio backend on Android) needs API 26.
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "1.48.2"
    }

    packaging {
        jniLibs {
            // Cargo already strips the Rust library (workspace Cargo.toml).
            keepDebugSymbols += "**/libpocket_god_android.so"
        }
    }
}

dependencies {
    // GameActivity's Java side must match the native glue in the android-activity crate
    // (0.6.x: GameActivity 4.4.0). The AAR declares no dependencies of its own, but
    // GameActivity extends AppCompatActivity.
    implementation("androidx.games:games-activity:4.4.0")
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.core:core:1.18.0")
}
