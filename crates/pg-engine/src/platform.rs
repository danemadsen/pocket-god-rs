//! Platform services the game requests (`docs/re/managers-platform.md` §13.3, §18): URL
//! opening, vibration, the launch splash, the locale, and the `UITextField` the controller uses
//! for text entry. `pg-platform` and `pg-ui` fulfil them; the game only reads and writes these
//! resources.

use bevy_ecs::prelude::*;

/// Requests drained by the platform layer every frame.
#[derive(Resource, Default, Debug)]
pub struct PlatformRequests {
    /// `[UIApplication openURL:]`.
    pub open_urls: Vec<String>,

    /// `AudioServicesPlaySystemSound(kSystemSoundID_Vibrate)`.
    pub vibrate: bool,

    /// Set once by `hideLoadingScreen`; the platform hides the launch image.
    pub loading_screen_hidden: bool,
}

/// `[[NSLocale currentLocale] objectForKey:NSLocaleCountryCode]`, filled by the platform.
#[derive(Resource, Default, Debug, Clone)]
pub struct Locale {
    pub country_code: Option<String>,
}

/// `UITextFieldDelegate -textField:shouldChangeCharactersInRange:replacementString:`
/// (delegate, location, length, replacement) in UTF-16 units like UIKit.
pub type ShouldChangeFn = fn(&mut World, Entity, usize, usize, &str) -> bool;

/// The controller's hidden `UITextField` (`PocketGodViewController::init`). The game sets the
/// delegate callbacks; `pg-ui` shows an egui text field while `first_responder` is set and
/// forwards edits and Return through the callbacks, exactly like UIKit would.
#[derive(Resource, Default)]
pub struct TextField {
    /// The keyboard is up (`becomeFirstResponder`).
    pub first_responder: bool,
    pub text: String,

    /// `enablesReturnKeyAutomatically`: Return disabled while the field is empty.
    pub enables_return_key_automatically: bool,
    pub delegate: Option<Entity>,

    /// `textField:shouldChangeCharactersInRange:replacementString:`.
    pub should_change: Option<ShouldChangeFn>,

    /// `UITextFieldTextDidChangeNotification` observer (`textDidChange:`).
    pub did_change: Option<fn(&mut World, Entity)>,

    /// `textFieldShouldReturn:`.
    pub should_return: Option<fn(&mut World, Entity) -> bool>,

    /// `textFieldDidEndEditing:` (run by [`resign_first_responder`]).
    pub did_end_editing: Option<fn(&mut World, Entity)>,

    /// The port's cancel gesture (Escape, Android back): what tapping the cancel display object
    /// does in the original, i.e. the controller's `onHideKeyboard`.
    pub cancel: Option<fn(&mut World, Entity)>,
}

/// `[UIApplication openURL:]`.
pub fn open_url(world: &mut World, url: &str) {
    world.resource_mut::<PlatformRequests>().open_urls.push(url.to_owned());
}

/// Vibrate once.
pub fn vibrate(world: &mut World) {
    world.resource_mut::<PlatformRequests>().vibrate = true;
}

/// `[textField becomeFirstResponder]`: the keyboard opens and `pg-ui` shows the field, which
/// starts empty (`clearsOnBeginEditing = YES`).
pub fn become_first_responder(world: &mut World) {
    let mut field = world.resource_mut::<TextField>();
    field.first_responder = true;
    field.text.clear();
}

/// `[textField resignFirstResponder]`: the keyboard closes and, as in UIKit, the delegate's
/// `textFieldDidEndEditing:` runs before this returns. Nothing happens if it was not open.
pub fn resign_first_responder(world: &mut World) {
    let mut field = world.resource_mut::<TextField>();
    let was_editing = field.first_responder;
    field.first_responder = false;
    let delegate = field.delegate;
    let did_end_editing = field.did_end_editing;
    if !was_editing {
        return;
    }
    if let (Some(delegate), Some(did_end_editing)) = (delegate, did_end_editing) {
        did_end_editing(world, delegate);
    }
}
