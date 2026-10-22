#!/usr/bin/env bash
# Builds the Android APK: android/app/build/outputs/apk/{debug,release}/.
#
#   scripts/build-android.sh             debug APK for arm64-v8a (phones, Apple-silicon emulators)
#   scripts/build-android.sh --release   release APK, signed with the key android/keystore.properties
#                                        points at (see android/keystore.properties.example)
#   scripts/build-android.sh --install   also install and launch it on the adb device
#   ANDROID_ABIS="arm64-v8a x86_64" scripts/build-android.sh   also for x86_64 emulators
#
# Needs ANDROID_HOME (an SDK with platform 36 and an NDK under ndk/; cargo-ndk uses the
# newest one unless ANDROID_NDK_HOME is set), the matching rustup targets
# (aarch64-linux-android, x86_64-linux-android), cargo-ndk and JDK 17+.
set -euo pipefail

cd "$(dirname "$0")/.."

: "${ANDROID_HOME:?ANDROID_HOME must point at the Android SDK}"

ABIS="${ANDROID_ABIS:-arm64-v8a}"
# The app's minSdk: AAudio, Bevy's audio backend on Android, needs API 26.
API_LEVEL=26
JNI_LIBS=android/app/src/main/jniLibs
ADB="$ANDROID_HOME/platform-tools/adb"

release=false
install=false
for argument in "$@"; do
    case "$argument" in
        --release) release=true ;;
        --install) install=true ;;
        *) echo "unknown option $argument" >&2; exit 2 ;;
    esac
done

if $release; then
    GRADLE_TASK=assembleRelease
    APK=android/app/build/outputs/apk/release/app-release.apk
    if [[ ! -f android/keystore.properties ]]; then
        echo "android/keystore.properties is missing: the release APK would be unsigned" >&2
        exit 1
    fi
else
    GRADLE_TASK=assembleDebug
    APK=android/app/build/outputs/apk/debug/app-debug.apk
fi

target_args=()
for abi in $ABIS; do
    target_args+=(-t "$abi")
done

# Start from an empty jniLibs so ABIs that are no longer built don't end up in the APK.
rm -rf "$JNI_LIBS"
cargo ndk "${target_args[@]}" -P "$API_LEVEL" -o "$JNI_LIBS" build --locked --release -p pocket-god-android

(cd android && ./gradlew "$GRADLE_TASK")

apk_megabytes=$(( $(wc -c < "$APK") / 1000000 ))
echo "Built $APK ($apk_megabytes MB)"

if $install; then
    "$ADB" install -r "$APK"
    "$ADB" shell am start -n com.boltcreative.pocketgod.port/.MainActivity
fi
