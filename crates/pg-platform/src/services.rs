//! OS services the game asks for (`pg_engine::platform`, `docs/re/managers-platform.md` §18):
//! opening URLs, vibrating, the locale's country code and the clock seed of the random
//! generator.

use bevy::prelude::*;
use pg_engine::platform::{Locale, PlatformRequests};
use pg_engine::random::Random;

/// How long `kSystemSoundID_Vibrate` buzzes (ms).
pub const VIBRATION_MS: u32 = 400;

/// The URL schemes handed to the system: web pages. App Store and other links of the
/// original's data have no target outside iOS.
const OPENABLE_SCHEMES: [&str; 2] = ["http://", "https://"];

/// `window.open` features on the web: a new tab without access to the game's.
#[cfg(target_arch = "wasm32")]
const WEB_WINDOW_FEATURES: &str = "noopener,noreferrer";

/// Whether the port opens `url`.
pub fn is_openable_url(url: &str) -> bool {
    let lowercase = url.trim().to_ascii_lowercase();
    OPENABLE_SCHEMES.iter().any(|scheme| lowercase.starts_with(scheme))
}

/// `[UIApplication openURL:]` with the platform's browser.
pub fn open_url(url: &str) {
    if !is_openable_url(url) {
        info!("not opening {url}: no handler outside iOS");
        return;
    }

    #[cfg(target_os = "android")]
    crate::android::open_url(url);

    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        // The opened page gets no handle on the game's tab: some of the original's link
        // domains may have changed hands. With `noopener` the browser returns no window even
        // when it opened one, so only an exception counts as a failure.
        let result = window.open_with_url_and_target_and_features(url, "_blank", WEB_WINDOW_FEATURES);
        if let Err(error) = result {
            warn!("the browser did not open {url}: {error:?}");
        }
    }

    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    if let Err(error) = webbrowser::open(url) {
        warn!("could not open {url}: {error}");
    }
}

/// `AudioServicesPlaySystemSound(kSystemSoundID_Vibrate)`: the Android vibrator,
/// `navigator.vibrate` on the web, nothing on desktop.
pub fn vibrate() {
    #[cfg(target_os = "android")]
    crate::android::vibrate(i64::from(VIBRATION_MS));

    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        let navigator = window.navigator();
        navigator.vibrate_with_duration(VIBRATION_MS);
    }
}

/// The region of a locale identifier (`en-US`, `en_US.UTF-8`, `zh-Hans-CN`, `de_DE@euro`) as
/// `NSLocaleCountryCode` gives it: two upper-case letters or three digits.
pub fn country_code_from_locale(locale: &str) -> Option<String> {
    let without_encoding = locale.split(['.', '@']).next()?;
    let mut subtags = without_encoding.split(['-', '_']);
    subtags.next()?;

    for subtag in subtags {
        let letters = subtag.len() == 2 && subtag.chars().all(|character| character.is_ascii_alphabetic());
        let digits = subtag.len() == 3 && subtag.chars().all(|character| character.is_ascii_digit());
        if letters || digits {
            return Some(subtag.to_ascii_uppercase());
        }
    }
    None
}

/// The OS locale identifier.
fn os_locale() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        window.navigator().language()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        sys_locale::get_locale()
    }
}

/// `[[NSLocale currentLocale] objectForKey:NSLocaleCountryCode]`.
fn os_country_code() -> Option<String> {
    let locale = os_locale();
    let from_locale = locale.as_deref().and_then(country_code_from_locale);

    #[cfg(target_os = "android")]
    if from_locale.is_none() {
        return crate::android::country_code();
    }

    from_locale
}

/// Milliseconds since the Unix epoch: `srandom(time(0))`'s clock.
/// Seconds since the Unix epoch from the system clock (web-safe).
pub fn unix_time_seconds() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() / 1000.0
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let now = std::time::SystemTime::now();
        let since_epoch = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
        since_epoch.as_secs_f64()
    }
}

/// `[[NSTimeZone systemTimeZone] secondsFromGMTForDate:]`: the local time zone's offset from
/// UTC at `unix_seconds`, daylight saving included (the OS time-zone database; the browser's
/// on the web).
pub fn seconds_from_gmt(unix_seconds: i64) -> f64 {
    let Some(instant) = chrono::DateTime::from_timestamp(unix_seconds, 0) else {
        return 0.0;
    };
    let local = instant.with_timezone(&chrono::Local);
    let offset = local.offset().local_minus_utc();
    f64::from(offset)
}

fn clock_seed() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as u64
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let now = std::time::SystemTime::now();
        let since_epoch = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
        since_epoch.as_millis() as u64
    }
}

/// Seeds the random generator from the clock and reads the locale (`PreStartup`).
pub fn start_services(mut random: ResMut<Random>, mut locale: ResMut<Locale>) {
    let seed = clock_seed();
    *random = Random::with_seed(seed);

    locale.country_code = os_country_code();
    info!("locale country code: {:?}", locale.country_code);
}

/// Opens the requested URLs and vibrates (`PlatformSystems::Output`).
pub fn handle_platform_requests(mut requests: ResMut<PlatformRequests>) {
    let wants_vibration = requests.vibrate;
    let has_urls = !requests.open_urls.is_empty();
    if !wants_vibration && !has_urls {
        return;
    }

    let urls = std::mem::take(&mut requests.open_urls);
    for url in urls {
        open_url(&url);
    }
    if wants_vibration {
        requests.vibrate = false;
        vibrate();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn country_codes_come_from_the_region_subtag() {
        assert_eq!(country_code_from_locale("en-US").as_deref(), Some("US"));
        assert_eq!(country_code_from_locale("en_GB.UTF-8").as_deref(), Some("GB"));
        assert_eq!(country_code_from_locale("zh-Hans-CN").as_deref(), Some("CN"));
        assert_eq!(country_code_from_locale("de_de@euro").as_deref(), Some("DE"));
        assert_eq!(country_code_from_locale("es-419").as_deref(), Some("419"));
        assert_eq!(country_code_from_locale("fr"), None);
        assert_eq!(country_code_from_locale("C"), None);
        assert_eq!(country_code_from_locale(""), None);
    }

    #[test]
    fn only_web_links_are_opened() {
        assert!(is_openable_url("http://www.boltcreative.com"));
        assert!(is_openable_url(" HTTPS://example.com/x"));
        assert!(!is_openable_url("itms-apps://itunes.apple.com/app/id123"));
        assert!(!is_openable_url("file:///etc/passwd"));
        assert!(!is_openable_url("javascript:alert(1)"));
    }

    #[test]
    fn requests_are_drained() {
        let mut world = World::new();
        let mut requests = PlatformRequests::default();
        requests.open_urls.push("itms-apps://itunes.apple.com".to_owned());
        requests.vibrate = true;
        world.insert_resource(requests);
        world.run_system_cached(handle_platform_requests).expect("runs");

        let requests = world.resource::<PlatformRequests>();
        assert!(requests.open_urls.is_empty());
        assert!(!requests.vibrate);
    }
}
