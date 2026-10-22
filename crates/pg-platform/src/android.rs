//! JNI calls into the activity (`GameActivity`): the display rotation, the vibrator, URL
//! intents and the locale. Every call runs in its own local reference frame; Java exceptions
//! are described to logcat and cleared.

use bevy::android::ANDROID_APP;
use bevy::prelude::*;
use jni::errors::Result as JniResult;
use jni::objects::{JObject, JString, JValue};
use jni::{JNIEnv, JavaVM};

/// `VibrationEffect.DEFAULT_AMPLITUDE`.
const DEFAULT_AMPLITUDE: i32 = -1;

/// Local references one call may create.
const LOCAL_FRAME_CAPACITY: i32 = 16;

/// Runs `call` with the JNI environment of this thread and the activity.
fn with_activity<R, F>(call: F) -> Option<R>
where
    F: FnOnce(&mut JNIEnv, &JObject) -> JniResult<R>,
{
    let app = ANDROID_APP.get()?;

    // SAFETY: android-activity hands out the process's `JavaVM` and the activity's global
    // reference, both valid for the life of the app; `JObject` never deletes the reference.
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }.ok()?;
    let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
    let mut env = vm.attach_current_thread_permanently().ok()?;

    let result = env.with_local_frame(LOCAL_FRAME_CAPACITY, |env| call(env, &activity));
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_describe();
        let _ = env.exception_clear();
    }
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            warn!("JNI call failed: {error}");
            None
        }
    }
}

/// `getWindowManager().getDefaultDisplay().getRotation()`.
fn query_display_rotation(env: &mut JNIEnv, activity: &JObject) -> JniResult<i32> {
    let window_manager = env.call_method(activity, "getWindowManager", "()Landroid/view/WindowManager;", &[])?;
    let window_manager = window_manager.l()?;
    let display = env.call_method(&window_manager, "getDefaultDisplay", "()Landroid/view/Display;", &[])?;
    let display = display.l()?;
    let rotation = env.call_method(&display, "getRotation", "()I", &[])?;
    rotation.i()
}

/// The display rotation in quarter turns (`Surface.ROTATION_0` … `ROTATION_270`).
pub(crate) fn display_rotation() -> Option<i32> {
    with_activity(query_display_rotation)
}

/// `vibrator.vibrate(VibrationEffect.createOneShot(milliseconds, DEFAULT_AMPLITUDE))` (API 26).
fn vibrate_once(env: &mut JNIEnv, activity: &JObject, milliseconds: i64) -> JniResult<()> {
    let service_name = env.new_string("vibrator")?;
    let service_argument = JValue::Object(&service_name);
    let service = env.call_method(
        activity,
        "getSystemService",
        "(Ljava/lang/String;)Ljava/lang/Object;",
        &[service_argument],
    )?;
    let vibrator = service.l()?;
    if vibrator.is_null() {
        return Ok(());
    }
    let has_vibrator = env.call_method(&vibrator, "hasVibrator", "()Z", &[])?;
    if !has_vibrator.z()? {
        return Ok(());
    }

    let effect_arguments = [JValue::Long(milliseconds), JValue::Int(DEFAULT_AMPLITUDE)];
    let effect = env.call_static_method(
        "android/os/VibrationEffect",
        "createOneShot",
        "(JI)Landroid/os/VibrationEffect;",
        &effect_arguments,
    )?;
    let effect = effect.l()?;
    let effect_argument = JValue::Object(&effect);
    env.call_method(&vibrator, "vibrate", "(Landroid/os/VibrationEffect;)V", &[effect_argument])?;
    Ok(())
}

/// Vibrates once for `milliseconds`.
pub(crate) fn vibrate(milliseconds: i64) {
    with_activity(|env, activity| vibrate_once(env, activity, milliseconds));
}

/// `startActivity(new Intent(Intent.ACTION_VIEW, Uri.parse(url)))`.
fn start_view_intent(env: &mut JNIEnv, activity: &JObject, url: &str) -> JniResult<()> {
    let url_string = env.new_string(url)?;
    let url_argument = JValue::Object(&url_string);
    let uri = env.call_static_method("android/net/Uri", "parse", "(Ljava/lang/String;)Landroid/net/Uri;", &[url_argument])?;
    let uri = uri.l()?;

    let action = env.new_string("android.intent.action.VIEW")?;
    let intent_arguments = [JValue::Object(&action), JValue::Object(&uri)];
    let intent = env.new_object("android/content/Intent", "(Ljava/lang/String;Landroid/net/Uri;)V", &intent_arguments)?;

    let intent_argument = JValue::Object(&intent);
    env.call_method(activity, "startActivity", "(Landroid/content/Intent;)V", &[intent_argument])?;
    Ok(())
}

/// Opens `url` with an `ACTION_VIEW` intent (the browser, for web links).
pub(crate) fn open_url(url: &str) {
    with_activity(|env, activity| start_view_intent(env, activity, url));
}

/// `Locale.getDefault().getCountry()`.
fn query_country(env: &mut JNIEnv, _activity: &JObject) -> JniResult<String> {
    let locale = env.call_static_method("java/util/Locale", "getDefault", "()Ljava/util/Locale;", &[])?;
    let locale = locale.l()?;
    let country = env.call_method(&locale, "getCountry", "()Ljava/lang/String;", &[])?;
    let country_object = country.l()?;
    let country = JString::from(country_object);
    let text = env.get_string(&country)?;
    Ok(text.into())
}

/// The default locale's country, when the system properties do not tell.
pub(crate) fn country_code() -> Option<String> {
    let country = with_activity(query_country)?;
    if country.is_empty() { None } else { Some(country) }
}

/// The app's internal storage (`Context.getFilesDir()`).
pub(crate) fn internal_data_path() -> Option<std::path::PathBuf> {
    let app = ANDROID_APP.get()?;
    app.internal_data_path()
}
