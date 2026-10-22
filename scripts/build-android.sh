#!/usr/bin/env bash
# Builds the Android debug APK: android/app/build/outputs/apk/debug/app-debug.apk.
#
#   scripts/build-android.sh             build for arm64-v8a (phones, Apple-silicon emulators)
#   scripts/build-android.sh --install   build, then install and launch it on the adb device
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
APK=android/app/build/outputs/apk/debug/app-debug.apk
ADB="$ANDROID_HOME/platform-tools/adb"

target_args=()
for abi in $ABIS; do
    target_args+=(-t "$abi")
done

# Start from an empty jniLibs so ABIs that are no longer built don't end up in the APK.
rm -rf "$JNI_LIBS"
cargo ndk "${target_args[@]}" -P "$API_LEVEL" -o "$JNI_LIBS" build --release -p pocket-god-android

(cd android && ./gradlew assembleDebug)

apk_megabytes=$(( $(wc -c < "$APK") / 1000000 ))
echo "Built $APK ($apk_megabytes MB)"

if [[ "${1:-}" == "--install" ]]; then
    "$ADB" install -r "$APK"
    "$ADB" shell am start -n com.boltcreative.pocketgod.port/.MainActivity
fi
