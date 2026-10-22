//! UIKit stand-ins drawn with egui (`docs/re/managers-platform.md` §13.3): the view
//! controller's hidden `UITextField` for name entry. The game drives it through
//! [`pg_engine::platform::TextField`]. While the field is first responder this plugin shows a
//! single-line field, asks the delegate before every edit, keeps the accepted text, and forwards
//! Return and the cancel gesture (Escape) the way UIKit and the original's cancel object do.

mod edit;

use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiPrimaryContextPass, PrimaryEguiContext, egui};
use pg_engine::platform::TextField;

pub use edit::{Utf16Edit, edit_between};

/// Distance of the field from the bottom of the window, in egui points.
const FIELD_BOTTOM_MARGIN: f32 = 24.0;
const FIELD_WIDTH: f32 = 240.0;

/// The text field. Needs `bevy_egui::EguiPlugin` with a primary context (the app's overlay).
pub struct PgUiPlugin;

impl Plugin for PgUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextFieldUi>();
        app.add_systems(EguiPrimaryContextPass, text_field_ui);
    }
}

/// Whether the field was open last frame (focus is taken when it opens).
#[derive(Resource, Default)]
struct TextFieldUi {
    open: bool,
}

/// What the player did to the field this frame.
struct FieldInput {
    edited: Option<String>,
    returned: bool,
    cancelled: bool,
}

fn text_field_ui(world: &mut World) {
    let field = world.resource::<TextField>();
    let open = field.first_responder;
    let text = field.text.clone();

    let was_open = std::mem::replace(&mut world.resource_mut::<TextFieldUi>().open, open);
    if !open {
        return;
    }
    let Some(context) = primary_context(world) else {
        return;
    };

    let input = show_field(&context, &text, !was_open);
    apply_input(world, &text, input);
}

fn primary_context(world: &mut World) -> Option<egui::Context> {
    let mut contexts = world.query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>();
    let mut context = contexts.single_mut(world).ok()?;
    Some(context.get_mut().clone())
}

fn show_field(context: &egui::Context, text: &str, opening: bool) -> FieldInput {
    let mut edited_text = text.to_owned();
    let mut returned = false;
    let mut cancelled = false;

    let offset = egui::vec2(0.0, -FIELD_BOTTOM_MARGIN);
    let area = egui::Area::new(egui::Id::new("pg-ui-text-field"));
    let anchored = area.anchor(egui::Align2::CENTER_BOTTOM, offset);
    anchored.show(context, |ui| {
        let editor = egui::TextEdit::singleline(&mut edited_text);
        let sized_editor = editor.desired_width(FIELD_WIDTH);
        let response = ui.add(sized_editor);
        // The iOS keyboard stays up until the game resigns the field.
        if opening || !response.has_focus() {
            response.request_focus();
        }
        returned = ui.input(|input| input.key_pressed(egui::Key::Enter));
        cancelled = ui.input(|input| input.key_pressed(egui::Key::Escape));
    });

    let edited = (edited_text != text).then_some(edited_text);
    FieldInput { edited, returned, cancelled }
}

fn apply_input(world: &mut World, old_text: &str, input: FieldInput) {
    let field = world.resource::<TextField>();
    let Some(delegate) = field.delegate else {
        return;
    };
    let should_change = field.should_change;
    let did_change = field.did_change;
    let should_return = field.should_return;
    let cancel = field.cancel;
    let return_needs_text = field.enables_return_key_automatically;

    if let Some(new_text) = input.edited {
        let edit = edit_between(old_text, &new_text);
        let accepted = match should_change {
            Some(should_change) => should_change(world, delegate, edit.location, edit.length, &edit.replacement),
            None => true,
        };
        if accepted {
            world.resource_mut::<TextField>().text = new_text;
            if let Some(did_change) = did_change {
                did_change(world, delegate);
            }
        }
    }

    if input.returned {
        let empty = world.resource::<TextField>().text.is_empty();
        let return_enabled = !(return_needs_text && empty);
        if let (true, Some(should_return)) = (return_enabled, should_return) {
            should_return(world, delegate);
        }
    }

    if let (true, Some(cancel)) = (input.cancelled, cancel) {
        cancel(world, delegate);
    }
}
