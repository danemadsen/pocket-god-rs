//! `PocketGodViewController`: methods in this part of the class.

use std::sync::OnceLock;

use pg_assets::data::links::{self, Link};
use pg_engine::display;
use pg_engine::display::text::{self, Text};
use pg_engine::hooks::EngineHooks;
use pg_engine::platform::{self, Locale, TextField};
use pg_engine::sound;
use pg_engine::touch;

use crate::logic::apocalypse_locust_closet_xenu::locust_logic;
use crate::logic::sky_powers::hurricane_logic;
use crate::managers::pocket_god_app_delegate;
use crate::persona::{persona, pygmy_persona};
use crate::prelude::*;

use super::*;

/// `PocketGodViewController` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct ControllerAppPlatform {
    /// `mView`: `@"BCView"`
    ///
    /// PORT: the view is the engine's `BcView` resource, so this stays `None`.
    pub view: Option<Entity>,

    /// `mbComicUrlOpened`: `B`
    pub comic_url_opened: bool,

    /// `mLaunchUrl`: `@"NSString"`
    pub launch_url: Option<String>,

    /// `mbHideLoadingScreen`: `B`
    pub hide_loading_screen: bool,

    /// `mbModalViewControllerActive`: `B`
    pub modal_view_controller_active: bool,

    /// `mTextField`: `@"UITextField"`
    ///
    /// PORT: the text field is the engine's `pg_engine::platform::TextField` resource, so this
    /// stays `None`.
    pub text_field: Option<Entity>,

    /// `mSaveEditText`: `@"NSString"`
    pub save_edit_text: Option<String>,

    /// `mKeyboardListener`: `@"<KeyboardListener>"`
    pub keyboard_listener: Option<Entity>,

    /// `mMuteSequence`: `@"BCSequence"`
    pub mute_sequence: Option<Entity>,

    /// `mDelegate`: `@"<PocketGodViewDelegate>"`
    pub delegate: Option<Entity>,

    /// `mbKeyboardOpen`: `B`
    pub keyboard_open: bool,

    /// `mMaxEditCharCount`: `i`
    pub max_edit_char_count: i32,

    /// `mEditingText`: `@"BCText"`
    pub editing_text: Option<Entity>,

    /// `mActiveCancelEditDisplayObject`: `@"BCDisplayObject"`
    pub active_cancel_edit_display_object: Option<Entity>,

    /// `mbChartboostAdVisible`: `B`
    pub chartboost_ad_visible: bool,
}

/// UIKit's `presentedViewController` of the controller: the sheet that
/// `presentViewController:animated:completion:` shows (only Game Center sheets, a
/// `GkTurnBasedMatchmakerViewController` entity). The port has no UIKit: a match service
/// (`docs/re/managers-platform.md` §18.3) shows the sheet while it is set.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct PresentedViewController {
    pub view_controller: Option<Entity>,
}

/// `UIInterfaceOrientationMaskLandscapeRight`.
const INTERFACE_ORIENTATION_MASK_LANDSCAPE_RIGHT: u64 = 8;

/// `UIInterfaceOrientationLandscapeRight`.
const INTERFACE_ORIENTATION_LANDSCAPE_RIGHT: i64 = 3;

/// The sound id `playBackgroundLoop:` re-aliases, and the island music outside the apocalypse.
const BACKGROUND_LOOP: &str = "BackgroundLoop";

/// The music during the apocalypse.
const APOCALYPSE_LOOP: &str = "ApocalypseLoop";

/// The default country of `openCountryURL:`.
const DEFAULT_COUNTRY_CODE: &str = "US";

/// `-[PocketGodViewController init]` @ 0x100069ce8
pub fn init(world: &mut World, this: Entity) {
    insert_components(world, this);

    let mut singletons = world.get_resource_or_insert_with(Singletons::default);
    singletons.pocket_god_view_controller = Some(this);

    let app_start_time = cf_absolute_time_get_current(world);
    world.obj_mut::<ControllerBootAndFrame>(this).app_start_time = app_start_time;

    // `mTextField`: the hidden `UITextField` (frame 60, 214, 30, 30) with `self` as its delegate
    // and `textDidChange:` observing `UITextFieldTextDidChangeNotification`. It clears on begin
    // editing (`become_first_responder`); no autocapitalisation, no autocorrection, return key
    // Done.
    let mut field = world.get_resource_or_insert_with(TextField::default);
    field.delegate = Some(this);
    field.should_change = Some(text_field);
    field.did_change = Some(text_did_change);
    field.should_return = Some(text_field_should_return);
    field.did_end_editing = Some(text_field_did_end_editing);
    field.cancel = Some(on_hide_keyboard);

    world.obj_mut::<ControllerAppPlatform>(this).hide_loading_screen = false;
    {
        let mut locations = world.obj_mut::<ControllerIslandLocations>(this);
        locations.island_location_state = -1;
        locations.last_island_location_state = -1;
    }

    // `mView = [[BCView alloc] initWithFrame:[[UIScreen mainScreen] applicationFrame]]` is the
    // engine's `BcView` (the platform sizes it); its iOS < 8 rotation and `setView:` are UIKit.
    set_view_delegate(world);
}

/// `[mView setDelegate:self]`: the view's touch callbacks go to the controller.
fn set_view_delegate(world: &mut World) {
    let mut hooks = world.get_resource_or_insert_with(EngineHooks::default);
    hooks.touch_began = view_delegate_touch_began;
    hooks.touch_moved = view_delegate_touch_moved;
    hooks.touch_ended = view_delegate_touch_ended;
    hooks.touch_cancelled = view_delegate_touch_cancelled;
    hooks.pre_start_touch_began = view_delegate_pre_start_touch_began;
}

/// `BCViewDelegate -touchBegan:allTouches:`.
fn view_delegate_touch_began(world: &mut World, touch: Entity) {
    let Some(controller) = instance(world) else {
        return;
    };
    let all_touches = touch::all_touches(world);
    touch_began(world, controller, touch, &all_touches);
}

/// `BCViewDelegate -touchMoved:allTouches:`.
fn view_delegate_touch_moved(world: &mut World, touch: Entity) {
    let Some(controller) = instance(world) else {
        return;
    };
    let all_touches = touch::all_touches(world);
    touch_moved(world, controller, touch, &all_touches);
}

/// `BCViewDelegate -touchEnded:allTouches:`.
fn view_delegate_touch_ended(world: &mut World, touch: Entity) {
    let Some(controller) = instance(world) else {
        return;
    };
    let all_touches = touch::all_touches(world);
    touch_ended(world, controller, touch, &all_touches);
}

/// `BCViewDelegate -touchCancelled:allTouches:`.
fn view_delegate_touch_cancelled(world: &mut World, touch: Entity) {
    let Some(controller) = instance(world) else {
        return;
    };
    let all_touches = touch::all_touches(world);
    touch_cancelled(world, controller, touch, &all_touches);
}

/// `BCViewDelegate -preStartTouchBeganXPos:yPos:`.
fn view_delegate_pre_start_touch_began(world: &mut World, x_pos: f32, y_pos: f32) {
    let Some(controller) = instance(world) else {
        return;
    };
    pre_start_touch_began_x_pos(world, controller, x_pos, y_pos);
}

/// `-[PocketGodViewController presentModalViewController:]` @ 0x10006d2dc
pub fn present_modal_view_controller(world: &mut World, this: Entity, view_controller: Entity) {
    if world.obj::<ControllerAppPlatform>(this).modal_view_controller_active {
        return;
    }
    world.obj_mut::<ControllerAppPlatform>(this).modal_view_controller_active = true;

    cancel_all_touches(world, this);
    touch::clear_touch_array_for_modal_view_controller(world);

    let delegate = world.obj::<ControllerAppPlatform>(this).delegate;
    if let Some(delegate) = delegate {
        pocket_god_app_delegate::on_modal_view_will_appear(world, delegate);
    }

    // `[self presentViewController:viewController animated:NO completion:nil]`.
    let mut presented = world.get_resource_or_insert_with(PresentedViewController::default);
    presented.view_controller = Some(view_controller);
}

/// `-[PocketGodViewController dismissModalViewController]` @ 0x10006d388
pub fn dismiss_modal_view_controller(world: &mut World, this: Entity) {
    if !world.obj::<ControllerAppPlatform>(this).modal_view_controller_active {
        return;
    }
    world.obj_mut::<ControllerAppPlatform>(this).modal_view_controller_active = false;

    // `[self dismissViewControllerAnimated:NO completion:nil]`.
    if let Some(mut presented) = world.get_resource_mut::<PresentedViewController>() {
        presented.view_controller = None;
    }

    let delegate = world.obj::<ControllerAppPlatform>(this).delegate;
    if let Some(delegate) = delegate {
        pocket_god_app_delegate::on_modal_view_will_disappear(world, delegate);
    }
}

/// `-[PocketGodViewController hideKeyboard]` @ 0x10006d3f8
pub fn hide_keyboard(world: &mut World, _this: Entity) {
    // `[mTextField endEditing:YES]` (runs `textFieldDidEndEditing:` if the field was editing),
    // then `removeFromSuperview`: `pg-ui` only shows the field while it is first responder.
    platform::resign_first_responder(world);
}

/// `-[PocketGodViewController showKeyboard]` @ 0x10006d444
pub fn show_keyboard(world: &mut World, _this: Entity) {
    // `[[self view] addSubview:mTextField]`, then `[mTextField becomeFirstResponder]`.
    platform::become_first_responder(world);
}

/// `-[PocketGodViewController textDidChange:]` @ 0x10006d49c
///
/// The `UITextFieldTextDidChangeNotification` argument is unused (and has no port object).
pub fn text_did_change(world: &mut World, this: Entity) {
    let editing_text = world.obj::<ControllerAppPlatform>(this).editing_text;
    let field_text = world.resource::<TextField>().text.clone();
    if let Some(editing_text) = editing_text {
        text::set_text(world, editing_text, Some(&field_text));
    }
}

/// `-[PocketGodViewController textField:shouldChangeCharactersInRange:replacementString:]` @ 0x10006d4f0
///
/// The text field is the `TextField` resource; the range is `location`, `length` in UTF-16
/// units, as in `NSString`.
pub fn text_field(world: &mut World, this: Entity, location: usize, length: usize, replacement_string: &str) -> bool {
    let field_text = world.resource::<TextField>().text.clone();
    let changed_text = string_by_replacing_characters_in_range(&field_text, location, length, replacement_string);
    let changed_length = changed_text.len() as u64;

    let max_edit_char_count = world.obj::<ControllerAppPlatform>(this).max_edit_char_count;
    let max_length = max_edit_char_count as i64 as u64;
    let accepted = changed_length <= max_length;
    if accepted {
        let editing_text = world.obj::<ControllerAppPlatform>(this).editing_text;
        let changed_string = String::from_utf16_lossy(&changed_text);
        if let Some(editing_text) = editing_text {
            text::set_text(world, editing_text, Some(&changed_string));
        }
    }
    accepted
}

/// `[text stringByReplacingCharactersInRange:range withString:replacement]`, as UTF-16 units.
///
/// PORT: a range beyond the text raises `NSRangeException` in the original; here it is
/// clamped to the text (the text field only reports ranges inside its text).
fn string_by_replacing_characters_in_range(text: &str, location: usize, length: usize, replacement: &str) -> Vec<u16> {
    let units: Vec<u16> = text.encode_utf16().collect();
    let start = location.min(units.len());
    let end = location.saturating_add(length).min(units.len());

    let mut replaced = Vec::with_capacity(units.len() + replacement.len());
    replaced.extend_from_slice(&units[..start]);
    replaced.extend(replacement.encode_utf16());
    replaced.extend_from_slice(&units[end..]);
    replaced
}

/// `-[PocketGodViewController textFieldShouldBeginEditing:]` @ 0x10006d5a4
pub fn text_field_should_begin_editing(_world: &mut World, _this: Entity) -> bool {
    true
}

/// `-[PocketGodViewController textFieldDidBeginEditing:]` @ 0x10006d5ac
pub fn text_field_did_begin_editing(_world: &mut World, _this: Entity) {}

/// `-[PocketGodViewController textFieldShouldEndEditing:]` @ 0x10006d5b0
pub fn text_field_should_end_editing(_world: &mut World, _this: Entity) -> bool {
    true
}

/// `-[PocketGodViewController textFieldDidEndEditing:]` @ 0x10006d5b8
pub fn text_field_did_end_editing(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerAppPlatform>(this).keyboard_open = false;
    hide_active_cancel_edit_display_object(world, this);

    // `[mTextField endEditing:YES]` and `removeFromSuperview`: the field has already resigned.
    platform::resign_first_responder(world);

    let (editing_text, keyboard_listener) = {
        let mut state = world.obj_mut::<ControllerAppPlatform>(this);
        state.save_edit_text = None;
        let keyboard_listener = state.keyboard_listener.take();
        let editing_text = state.editing_text.take();
        (editing_text, keyboard_listener)
    };

    let entered_text = bc_text_string(world, editing_text);
    crate::send::on_keyboard_text_entered(world, keyboard_listener, entered_text.as_deref());
}

/// `[mActiveCancelEditDisplayObject setIsSelectable:NO]`, `hide`, then
/// `mActiveCancelEditDisplayObject = nil`.
fn hide_active_cancel_edit_display_object(world: &mut World, this: Entity) {
    let active_cancel_edit_display_object = world.obj::<ControllerAppPlatform>(this).active_cancel_edit_display_object;
    if let Some(active_cancel_edit_display_object) = active_cancel_edit_display_object {
        let mut object = display::object_mut(world, active_cancel_edit_display_object);
        object.is_selectable = false;
        object.hidden = true;
    }
    world.obj_mut::<ControllerAppPlatform>(this).active_cancel_edit_display_object = None;
}

/// `[text text]` of a `BCText` that may be nil.
fn bc_text_string(world: &World, bc_text: Option<Entity>) -> Option<String> {
    let bc_text = bc_text?;
    world.obj::<Text>(bc_text).text.clone()
}

/// `-[PocketGodViewController textFieldShouldClear:]` @ 0x10006d6b0
pub fn text_field_should_clear(_world: &mut World, _this: Entity) -> bool {
    true
}

/// `-[PocketGodViewController textFieldShouldReturn:]` @ 0x10006d6b8
pub fn text_field_should_return(world: &mut World, this: Entity) -> bool {
    let editing_text = world.obj::<ControllerAppPlatform>(this).editing_text;
    let editing_text_id = match editing_text {
        Some(editing_text) => display::id(world, editing_text),
        None => None,
    };

    // The stats pages mirror every edit into the `BCText`, so its text is never nil here.
    let entered_text = bc_text_string(world, editing_text);
    let entered_text = entered_text.unwrap_or_default();

    let (active_pygmy_persona, tribe_persona, god_persona) = {
        let stats = world.obj::<ControllerStatsAndAchievements>(this);
        (stats.active_pygmy_persona, stats.tribe_persona, stats.god_persona)
    };
    match editing_text_id.as_deref() {
        Some("PygmyName") => {
            if let Some(active_pygmy_persona) = active_pygmy_persona {
                pygmy_persona::set_name(world, active_pygmy_persona, &entered_text);
            }
        }
        Some("PygmyDescription") => {
            if let Some(active_pygmy_persona) = active_pygmy_persona {
                persona::set_description(world, active_pygmy_persona, &entered_text);
            }
        }
        Some("TribeName") => {
            if let Some(tribe_persona) = tribe_persona {
                persona::set_name(world, tribe_persona, &entered_text);
            }
        }
        Some("GodName") => {
            if let Some(god_persona) = god_persona {
                persona::set_name(world, god_persona, &entered_text);
            }
        }
        Some("GodDescription") => {
            if let Some(god_persona) = god_persona {
                persona::set_description(world, god_persona, &entered_text);
            }
        }
        _ => {}
    }

    // `[textField resignFirstResponder]` (runs `textFieldDidEndEditing:`).
    platform::resign_first_responder(world);
    true
}

/// `-[PocketGodViewController onShowKeyboard:editingText:cancelEditDisplayObject:maxEditCharCount:preventEmpty:]` @ 0x10006d84c
pub fn on_show_keyboard(world: &mut World, this: Entity, listener: Entity, editing_text: Option<Entity>, cancel_edit_display_object: Option<Entity>, max_edit_char_count: i32, prevent_empty: bool) {
    if world.obj::<ControllerAppPlatform>(this).keyboard_open {
        return;
    }

    let save_edit_text = bc_text_string(world, editing_text);
    {
        let mut state = world.obj_mut::<ControllerAppPlatform>(this);
        state.keyboard_open = true;
        state.max_edit_char_count = max_edit_char_count;
        state.keyboard_listener = Some(listener);
        state.editing_text = editing_text;
        state.save_edit_text = save_edit_text;
        state.active_cancel_edit_display_object = cancel_edit_display_object;
    }

    if let Some(cancel_edit_display_object) = cancel_edit_display_object {
        display::object_mut(world, cancel_edit_display_object).is_selectable = true;
    }
    let active_cancel_edit_display_object = world.obj::<ControllerAppPlatform>(this).active_cancel_edit_display_object;
    if let Some(active_cancel_edit_display_object) = active_cancel_edit_display_object {
        display::object_mut(world, active_cancel_edit_display_object).hidden = false;
    }

    let mut field = world.get_resource_or_insert_with(TextField::default);
    field.enables_return_key_automatically = prevent_empty;

    perform_selector_on_main_thread(world, this, show_keyboard);
}

/// `-[PocketGodViewController onHideKeyboard]` @ 0x10006d994
pub fn on_hide_keyboard(world: &mut World, this: Entity) {
    if !world.obj::<ControllerAppPlatform>(this).keyboard_open {
        return;
    }

    let (editing_text, save_edit_text) = {
        let mut state = world.obj_mut::<ControllerAppPlatform>(this);
        state.keyboard_open = false;
        state.keyboard_listener = None;
        (state.editing_text, state.save_edit_text.clone())
    };
    if let Some(editing_text) = editing_text {
        text::set_text(world, editing_text, save_edit_text.as_deref());
    }
    {
        let mut state = world.obj_mut::<ControllerAppPlatform>(this);
        state.editing_text = None;
        state.save_edit_text = None;
    }
    hide_active_cancel_edit_display_object(world, this);

    perform_selector_on_main_thread(world, this, hide_keyboard);
}

/// `-[PocketGodViewController isKeyboardOpen]` @ 0x10006da88
pub fn is_keyboard_open(world: &mut World, this: Entity) -> bool {
    world.obj::<ControllerAppPlatform>(this).keyboard_open
}

/// `-[PocketGodViewController loadView]` @ 0x10006dad8
pub fn load_view(_world: &mut World, _this: Entity) {
    // `[self setView:mView]`: UIKit only.
}

/// `-[PocketGodViewController supportedInterfaceOrientations]` @ 0x10006daf4
pub fn supported_interface_orientations(_world: &mut World, _this: Entity) -> u64 {
    INTERFACE_ORIENTATION_MASK_LANDSCAPE_RIGHT
}

/// `-[PocketGodViewController preferredInterfaceOrientationForPresentation]` @ 0x10006dafc
pub fn preferred_interface_orientation_for_presentation(_world: &mut World, _this: Entity) -> i64 {
    INTERFACE_ORIENTATION_LANDSCAPE_RIGHT
}

/// `-[PocketGodViewController shouldAutorotate]` @ 0x10006db04
pub fn should_autorotate(_world: &mut World, _this: Entity) -> bool {
    false
}

/// `-[PocketGodViewController shouldAutorotateToInterfaceOrientation:]` @ 0x10006db0c
pub fn should_autorotate_to_interface_orientation(_world: &mut World, _this: Entity, interface_orientation: i64) -> bool {
    interface_orientation == INTERFACE_ORIENTATION_LANDSCAPE_RIGHT
}

/// `-[PocketGodViewController didReceiveMemoryWarning]` @ 0x10006db18
pub fn did_receive_memory_warning(_world: &mut World, _this: Entity) {
    // A log line, then `[super didReceiveMemoryWarning]` (UIKit).
}

/// `-[PocketGodViewController setDelegate:]` @ 0x10006e648
pub fn set_delegate(world: &mut World, this: Entity, delegate: Entity) {
    world.obj_mut::<ControllerAppPlatform>(this).delegate = Some(delegate);
}

/// `-[PocketGodViewController id]` @ 0x10006e6a4
pub fn id(_world: &World, _this: Entity) -> Option<String> {
    Some("PocketGodViewController".to_owned())
}

/// `-[PocketGodViewController openCountryURL:]` @ 0x10006e7b8
///
/// `link` is the `BCStructXmlLinkNodeDef *`, as an index into `links.json` (nothing sends this
/// in 1.48.2).
pub fn open_country_url(world: &mut World, this: Entity, link: Option<usize>) {
    let country_code = match world.get_resource::<Locale>() {
        Some(locale) => locale.country_code.clone(),
        None => None,
    };

    let all_links = store_links();
    let link_def = match link {
        Some(link) => all_links.get(link),
        None => None,
    };
    let Some(link_def) = link_def else {
        return;
    };

    let mut default_url = None;
    let mut country_url = None;
    for country_link in &link_def.country_links {
        if country_link.country == DEFAULT_COUNTRY_CODE {
            default_url = Some(country_link.url.as_str());
        }
        if Some(country_link.country.as_str()) == country_code.as_deref() {
            country_url = Some(country_link.url.as_str());
        }
    }
    if country_url.is_some() {
        default_url = country_url;
    }
    let Some(url) = default_url else {
        return;
    };
    open_url(world, this, url);
}

/// The store links of `links.json` (`BCStructXmlLinkNodeDef`s), parsed once.
fn store_links() -> &'static [Link] {
    static LINKS: OnceLock<Vec<Link>> = OnceLock::new();

    let loaded_links = LINKS.get_or_init(load_store_links);
    loaded_links.as_slice()
}

fn load_store_links() -> Vec<Link> {
    match links::load() {
        Ok(links_file) => links_file.links,
        Err(error) => panic!("openCountryURL: {error}"),
    }
}

/// `-[PocketGodViewController openLaunchUrl]` @ 0x10006e93c
pub fn open_launch_url(world: &mut World, this: Entity) {
    // PORT: the original first stops the render loop (`[[BCView instance] stopRenderLoop]`)
    // because `openURL:` leaves the app. The port stays in the foreground and keeps running
    // (`docs/re/managers-platform.md` §13.3); the platform stops the loop when the browser
    // takes the focus or the app is backgrounded, and restarts it on return.
    let launch_url = world.obj::<ControllerAppPlatform>(this).launch_url.clone();
    if let Some(launch_url) = launch_url {
        platform::open_url(world, &launch_url);
    }
}

/// `-[PocketGodViewController openUrl:]` @ 0x10006e9d4
pub fn open_url(world: &mut World, this: Entity, url: &str) {
    world.obj_mut::<ControllerAppPlatform>(this).launch_url = Some(url.to_owned());

    let hide_loading_screen = world.obj::<ControllerAppPlatform>(this).hide_loading_screen;
    if hide_loading_screen {
        perform_selector_on_main_thread(world, this, open_launch_url);
    }
}

/// `-[PocketGodViewController handleTurnEventForActiveMatch:]` @ 0x100070a10
pub fn handle_turn_event_for_active_match(_world: &mut World, _this: Entity, _game_match: Entity) -> bool {
    false
}

/// `-[PocketGodViewController handleMatchEndedForActiveMatch:]` @ 0x100070a18
pub fn handle_match_ended_for_active_match(_world: &mut World, _this: Entity, _game_match: Entity) -> bool {
    false
}

/// `-[PocketGodViewController handleTurnEventNotificationForInactiveMatch:]` @ 0x100070a20
pub fn handle_turn_event_notification_for_inactive_match(_world: &mut World, _this: Entity, _game_match: Entity) -> bool {
    false
}

/// `-[PocketGodViewController handleMatchEndedNotificationForInactiveMatch:]` @ 0x100070a28
pub fn handle_match_ended_notification_for_inactive_match(_world: &mut World, _this: Entity, _game_match: Entity) -> bool {
    false
}

/// The loop `playBackgroundLoop:` and `stopBackgroundLoop` control: `ApocalypseLoop` during the
/// apocalypse, `BackgroundLoop` otherwise.
fn current_background_loop(world: &World, this: Entity) -> &'static str {
    let apocalypse_active = world.obj::<ControllerApocalypse>(this).apocalypse_active;
    if apocalypse_active { APOCALYPSE_LOOP } else { BACKGROUND_LOOP }
}

/// `-[PocketGodViewController playBackgroundLoop:]` @ 0x100070ee8
///
/// A non-nil `background_loop` sound becomes the new `BackgroundLoop` (the island music); the
/// current loop then plays unless it already is.
pub fn play_background_loop(world: &mut World, this: Entity, background_loop: Option<&str>) {
    let current_loop = current_background_loop(world, this);
    if let Some(background_loop) = background_loop {
        let apocalypse_active = world.obj::<ControllerApocalypse>(this).apocalypse_active;
        if !apocalypse_active {
            sound::stop_sound(world, current_loop);
        }
        let background_loop_sound = sound::sound_by_id(world, background_loop);
        sound::set_sound_alias(world, BACKGROUND_LOOP, background_loop_sound);
    }

    if sound::is_sound_playing(world, current_loop) {
        return;
    }
    sound::play_sound(world, current_loop);
}

/// `-[PocketGodViewController playBackgroundLoop:::]` @ 0x10007104c
pub fn seq_play_background_loop(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let background_loop = call.param("soundId");
    play_background_loop(world, this, background_loop);
    ItemResult::Done
}

/// `-[PocketGodViewController stopBackgroundLoop:::]` @ 0x100071078
pub fn seq_stop_background_loop(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    stop_background_loop(world, this);
    ItemResult::Done
}

/// `-[PocketGodViewController pauseHurricaneSounds:::]` @ 0x10007109c
pub fn seq_pause_hurricane_sounds(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    pause_hurricane_sounds(world, this);
    ItemResult::Done
}

/// `-[PocketGodViewController resumeHurricaneSounds:::]` @ 0x1000710c0
pub fn seq_resume_hurricane_sounds(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    resume_hurricane_sounds(world, this);
    ItemResult::Done
}

/// `-[PocketGodViewController showChartboostAd]` @ 0x100088244
pub fn show_chartboost_ad(world: &mut World, this: Entity) -> bool {
    // PORT: there is no ad network (`docs/re/managers-platform.md` §18.1 rows 3–5).
    // `[Chartboost hasInterstitial:@"Home Screen"]` is always NO, so this is the original's
    // no-ad branch; its `cacheInterstitial:` does nothing.
    world.obj_mut::<ControllerAppPlatform>(this).chartboost_ad_visible = false;
    world.obj::<ControllerAppPlatform>(this).chartboost_ad_visible
}

/// `-[PocketGodViewController isChartboostAdVisible]` @ 0x100088354
pub fn is_chartboost_ad_visible(world: &mut World, this: Entity) -> bool {
    world.obj::<ControllerAppPlatform>(this).chartboost_ad_visible
}

/// `-[PocketGodViewController shouldRequestInterstitial:]` @ 0x100088364
pub fn should_request_interstitial(_world: &mut World, _this: Entity, _location: &str) -> bool {
    true
}

/// `-[PocketGodViewController shouldDisplayInterstitial:]` @ 0x100088390
pub fn should_display_interstitial(_world: &mut World, _this: Entity, _location: &str) -> bool {
    true
}

/// `-[PocketGodViewController didDisplayInterstitial:]` @ 0x1000883bc
pub fn did_display_interstitial(_world: &mut World, _this: Entity, _location: &str) {}

/// `-[PocketGodViewController didCacheInterstitial:]` @ 0x1000883e4
pub fn did_cache_interstitial(_world: &mut World, _this: Entity, _location: &str) {}

/// `-[PocketGodViewController didFailToLoadInterstitial:withError:]` @ 0x10008840c
pub fn did_fail_to_load_interstitial(world: &mut World, this: Entity, _location: &str, _error: u64) {
    world.obj_mut::<ControllerAppPlatform>(this).chartboost_ad_visible = false;
}

/// `-[PocketGodViewController didFailToRecordClick:withError:]` @ 0x10008844c
pub fn did_fail_to_record_click(_world: &mut World, _this: Entity, _location: &str, _error: u64) {}

/// `-[PocketGodViewController didDismissInterstitial:]` @ 0x100088474
pub fn did_dismiss_interstitial(world: &mut World, this: Entity, _location: &str) {
    // `[Chartboost cacheInterstitial:@"Home Screen"]` does nothing in the port.
    world.obj_mut::<ControllerAppPlatform>(this).chartboost_ad_visible = false;
}

/// `-[PocketGodViewController didCloseInterstitial:]` @ 0x1000884d8
pub fn did_close_interstitial(world: &mut World, this: Entity, _location: &str) {
    // `[Chartboost cacheInterstitial:@"Home Screen"]` does nothing in the port.
    world.obj_mut::<ControllerAppPlatform>(this).chartboost_ad_visible = false;
}

/// `-[PocketGodViewController didClickInterstitial:]` @ 0x10008853c
pub fn did_click_interstitial(world: &mut World, this: Entity, _location: &str) {
    // `[Chartboost cacheInterstitial:@"Home Screen"]` does nothing in the port.
    world.obj_mut::<ControllerAppPlatform>(this).chartboost_ad_visible = false;
}

/// `+[PocketGodViewController instance]` @ 0x100069cdc
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().pocket_god_view_controller
}

/// `-[PocketGodViewController (Game) pauseHurricaneSounds]` @ 0x100168df4
pub fn pause_hurricane_sounds(world: &mut World, this: Entity) {
    if is_hurricane_wind_active(world, this) {
        sound::stop_sound(world, "Wind");
    }
}

/// `-[PocketGodViewController (Game) resumeHurricaneSounds]` @ 0x100168e70
pub fn resume_hurricane_sounds(world: &mut World, this: Entity) {
    if is_hurricane_wind_active(world, this) {
        sound::play_sound(world, "Wind");
    }
}

/// `mHurricaneLogic && [mHurricaneLogic windActive]`.
fn is_hurricane_wind_active(world: &mut World, this: Entity) -> bool {
    let hurricane = world.obj::<ControllerGodPowers>(this).hurricane_logic;
    match hurricane {
        Some(hurricane) => hurricane_logic::wind_active(world, hurricane),
        None => false,
    }
}

/// `-[PocketGodViewController (Game) stopBackgroundLoop]` @ 0x100168eec
pub fn stop_background_loop(world: &mut World, this: Entity) {
    let current_loop = current_background_loop(world, this);
    sound::stop_sound(world, current_loop);
}

/// `-[PocketGodViewController (Game) pauseSleepingSounds]` @ 0x100168f64
pub fn pause_sleeping_sounds(world: &mut World, this: Entity) {
    let sleeping_count = world.obj::<ControllerPygmies>(this).sleeping_count;
    if 0 < sleeping_count {
        sound::stop_sound(world, "Snore");
    }
}

/// `-[PocketGodViewController (Game) resumeSleepingSounds]` @ 0x100168fd0
pub fn resume_sleeping_sounds(world: &mut World, this: Entity) {
    let sleeping_count = world.obj::<ControllerPygmies>(this).sleeping_count;
    if 0 < sleeping_count {
        sound::play_sound(world, "Snore");
    }
}

/// `-[PocketGodViewController (Game) pauseSleepingSounds:::]` @ 0x10016903c
pub fn seq_pause_sleeping_sounds(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    pause_sleeping_sounds(world, this);
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) resumeSleepingSounds:::]` @ 0x100169060
pub fn seq_resume_sleeping_sounds(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    resume_sleeping_sounds(world, this);
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) pauseLocustSounds]` @ 0x100169084
pub fn pause_locust_sounds(world: &mut World, this: Entity) {
    if is_locust_logic_active(world, this) {
        sound::stop_sound(world, "InsectBuzzLoop");
    }
}

/// `-[PocketGodViewController (Game) resumeLocustSounds]` @ 0x1001690fc
pub fn resume_locust_sounds(world: &mut World, this: Entity) {
    if is_locust_logic_active(world, this) {
        sound::play_sound(world, "InsectBuzzLoop");
    }
}

/// `[mLocustLogic isLocustActive]` (NO while there is no locust logic).
fn is_locust_logic_active(world: &mut World, this: Entity) -> bool {
    let locust = world.obj::<ControllerGodPowers>(this).locust_logic;
    match locust {
        Some(locust) => locust_logic::is_locust_active(world, locust),
        None => false,
    }
}

/// `-[PocketGodViewController (Game) pauseLocustSounds:::]` @ 0x100169174
pub fn seq_pause_locust_sounds(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    pause_locust_sounds(world, this);
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) resumeLocustSounds:::]` @ 0x100169198
pub fn seq_resume_locust_sounds(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    resume_locust_sounds(world, this);
    ItemResult::Done
}
