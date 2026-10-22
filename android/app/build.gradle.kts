import java.util.Properties

plugins {
    id("com.android.application")
}

/** Reads a properties file (any line endings). */
fun loadProperties(file: File): Properties {
    val properties = Properties()
    file.reader().use { reader -> properties.load(reader) }
    return properties
}

// Release signing: android/keystore.properties (never committed, see keystore.properties.example)
// points at a key bundle outside the repository. Without it, release builds are unsigned.
val keystorePointer = rootProject.file("keystore.properties")
val releaseKey: Properties? = if (keystorePointer.exists()) loadProperties(keystorePointer) else null

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

    signingConfigs {
        if (releaseKey != null) {
            val secrets = loadProperties(file(releaseKey.getProperty("keyProperties")))
            create("release") {
                storeFile = file(releaseKey.getProperty("storeFile"))
                storePassword = secrets.getProperty("storePassword")
                keyAlias = secrets.getProperty("releaseAlias")
                keyPassword = secrets.getProperty("releasePassword")
            }
        }
    }

    buildTypes {
        release {
            signingConfig = signingConfigs.findByName("release")
        }
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
