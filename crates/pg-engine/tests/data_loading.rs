//! `load_game_data` loads every embedded definition into a `World` exactly as `loadGame`
//! fills `BCLibrary` (`docs/re/textures.md` §5, `docs/re/sequences.md` §2,
//! `docs/re/struct-xml.md` §6, `docs/re/timelines.md` §2.9).

use bevy_ecs::prelude::*;
use pg_engine::animation::{Timelines, state_flags};
use pg_engine::display::DisplayObject;
use pg_engine::display::group;
use pg_engine::display::model::Model;
use pg_engine::display::text::{Justification, Text};
use pg_engine::library::Library;
use pg_engine::library::loader::load_game_data;
use pg_engine::sound::Sounds;
use pg_engine::texture::{TextureRequest, Textures};
use pg_engine::view::BcView;

fn loaded_world(wide_screen: bool) -> World {
    let mut world = World::new();
    let view = BcView {
        wide_screen,
        ..Default::default()
    };
    world.insert_resource(view);
    load_game_data(&mut world);
    world
}

fn display_group(world: &World, id: &str) -> Entity {
    let library = world.resource::<Library>();
    let Some(entity) = library.display_group(id) else {
        panic!("display group {id} is not registered");
    };
    entity
}

fn child_ids(world: &World, parent: Entity) -> Vec<String> {
    let mut ids = Vec::new();
    for child in group::children(world, parent) {
        let object = world.get::<DisplayObject>(*child).expect("children are display objects");
        let id = object.id.clone().unwrap_or_default();
        ids.push(id);
    }
    ids
}

#[test]
fn every_registry_has_the_documented_size() {
    let world = loaded_world(false);
    let library = world.resource::<Library>();
    let textures = world.resource::<Textures>();
    let timelines = world.resource::<Timelines>();
    let sounds = world.resource::<Sounds>();

    assert_eq!(textures.atlases.len(), 26);
    assert_eq!(library.atlas_ids.len(), 26);
    assert_eq!(textures.defs.len(), 8337);
    assert_eq!(library.texture_def_ids.len(), 8337);
    assert_eq!(library.sequence_defs.len(), 2548);
    assert_eq!(sounds.defs.len(), 359);
    assert_eq!(library.sounds.len(), 359);
    assert_eq!(library.font_defs.len(), 4);
    assert_eq!(library.gradient_defs.len(), 15);

    // 266 top-level groups plus the empty group TheRunsDemonDisplayGroup.
    assert_eq!(library.display_groups.len(), 267);

    // 148 markers; StatPageBackground, HelpBackdrop, UnderwaterBG and Menu repeat.
    assert_eq!(library.display_markers.len(), 78);

    assert_eq!(timelines.animations.len(), 2354);
    assert_eq!(library.animation_defs.len(), 2354);
    assert_eq!(timelines.frames.len(), 56_175);
    assert_eq!(timelines.state_lists.len(), 304_286);
    assert_eq!(timelines.states.len(), 81_294);
    assert_eq!(library.dynamic_texts.len(), 176);

    assert!(library.display_group("PocketGod").is_some());
    assert!(library.display_group("IslandDisplayGroup").is_some());
    assert!(library.display_group("TheRunsDemonDisplayGroup").is_some());
    assert!(library.display_marker("IslandDisplayMarker").is_some());
}

#[test]
fn start_up_uploads_the_init_images_then_the_main_atlases() {
    let narrow = loaded_world(false);
    let wide = loaded_world(true);

    let mut images = Vec::new();
    for request in &wide.resource::<Textures>().requests {
        let TextureRequest::Load { image, .. } = request else {
            panic!("only whole images are loaded at start-up");
        };
        images.push(image.as_str());
    }
    assert_eq!(images.len(), 19);
    assert_eq!(images[..2], ["EpisodeAlertTextureAtlas0", "VampireStayingAlive0"]);
    assert_eq!(images[16..], ["TextureAtlas0", "TextureAtlas1", "WidescreenOverlayAtlas0"]);
    assert_eq!(narrow.resource::<Textures>().requests.len(), 18);
}

#[test]
fn display_groups_hold_their_children_in_order() {
    let world = loaded_world(false);

    let pocket_god = display_group(&world, "PocketGod");
    let children = child_ids(&world, pocket_god);
    assert_eq!(children.len(), 13);
    assert_eq!(children[..3], ["IslandRootDisplayMarker", "UnderwaterDisplayMarker", "MenuDisplayMarker"]);

    let island = display_group(&world, "IslandDisplayGroup");
    let children = child_ids(&world, island);
    assert_eq!(children.len(), 50);
    assert_eq!(children[..4], ["Gradient", "Stars", "StarDisplayMarker", "Sun"]);
    for child in group::children(&world, island) {
        let object = world.get::<DisplayObject>(*child).expect("display object");
        assert_eq!(object.display_group, Some(island));
    }

    let stars = display_group(&world, "StarsDisplaygroup");
    let first_star = group::children(&world, stars)[0];
    let object = world.get::<DisplayObject>(first_star).expect("display object");
    assert_eq!((object.pos.x, object.pos.y, object.pos.z), (291.5, 64.6, -277.12));
    assert_eq!((object.scale.x, object.scale.y), (0.65, 0.65));
    assert_eq!(object.red, 0.41);

    let model = world.get::<Model>(first_star).expect("AchievementStar is a model");
    let library = world.resource::<Library>();
    assert_eq!(model.texture_def, library.texture_def_id("AchievementStar"));
}

#[test]
fn empty_groups_are_leaves_of_the_current_group() {
    let world = loaded_world(false);
    let the_runs = display_group(&world, "TheRunsDisplayGroup");
    let demon = display_group(&world, "TheRunsDemonDisplayGroup");

    assert!(group::children(&world, the_runs).contains(&demon));
    let object = world.get::<DisplayObject>(demon).expect("display object");
    assert_eq!(object.display_group, Some(the_runs));
}

#[test]
fn markers_link_to_groups_after_each_call() {
    let world = loaded_world(false);
    let library = world.resource::<Library>();

    // A call-4 marker linking a group defined in call 3.
    let island_marker = library.display_marker("IslandDisplayMarker").expect("marker exists");
    let rock_island = display_group(&world, "RockIslandDisplayGroup");
    assert_eq!(group::children(&world, island_marker), [rock_island]);

    // A group linked by six markers is one shared object; links resolve last marker first, so
    // its parent is the first linking marker in list order (in StatsPagePygmy).
    let frame = display_group(&world, "StatScreenFrame");
    let parent = world.get::<DisplayObject>(frame).expect("display object").display_group;
    let parent = parent.expect("the shared group has a parent marker");
    let stats_page = display_group(&world, "StatsPagePygmy");
    assert!(group::children(&world, stats_page).contains(&parent));

    let mut linking_markers = 0;
    let mut query = world.try_query::<&group::DisplayGroup>().expect("DisplayGroup is registered");
    for display_group in query.iter(&world) {
        if display_group.children.contains(&frame) {
            linking_markers += 1;
        }
    }
    assert_eq!(linking_markers, 6);
}

#[test]
fn everything_starts_hibernating() {
    let mut world = loaded_world(false);
    let mut query = world.query::<&DisplayObject>();
    let mut count = 0;
    for object in query.iter(&world) {
        assert!(object.hibernate, "{:?} is awake", object.id);
        assert_eq!(object.hibernate_ref_count, 0);
        count += 1;
    }

    // 266 groups and 5160 nodes from the struct XML, 176 dynamic texts.
    assert_eq!(count, 266 + 5160 + 176);
}

#[test]
fn texts_get_their_font_and_justification() {
    let world = loaded_world(false);
    let start_screen = display_group(&world, "StartScreen");

    let mut found = false;
    for child in group::children(&world, start_screen) {
        let Some(text) = world.get::<Text>(*child) else {
            continue;
        };
        let object = world.get::<DisplayObject>(*child).expect("display object");
        if object.id.as_deref() != Some("Arial12") {
            continue;
        }
        assert_eq!(text.font.as_deref(), Some("Arial12"));
        assert_eq!(text.justification, Justification::Center);
        found = true;
        break;
    }
    assert!(found, "StartScreen has the centred Arial12 text");
}

#[test]
fn dynamic_texts_are_resolved_by_timeline_states() {
    let world = loaded_world(false);
    let library = world.resource::<Library>();
    let timelines = world.resource::<Timelines>();

    let first_text = library.dynamic_text("WhatsnewLine12").expect("dynamic text exists");
    let text = world.get::<Text>(first_text).expect("dynamic texts are BCText objects");
    assert_eq!(text.font.as_deref(), Some("Arial12"));
    assert_eq!(text.justification, Justification::Left);
    let object = world.get::<DisplayObject>(first_text).expect("display object");
    assert_eq!(object.display_group, None, "dynamic texts are in no group");

    let mut dynamic_states = 0;
    let mut textured_states = 0;
    for state in &timelines.states {
        if state.flags & state_flags::DYNAMIC_TEXT != 0 {
            assert!(state.dynamic_text.is_some());
            dynamic_states += 1;
        }
        if state.flags & state_flags::TEXTURE != 0 {
            assert!(state.texture.is_some());
            textured_states += 1;
        }
    }
    assert_eq!((dynamic_states, textured_states), (508, 78_556));

    let next = library.animation_def("GhostFlyTurnAnim").expect("animation exists");
    let next_name = timelines.animation(next).next_animation.as_deref();
    assert_eq!(next_name, Some("GhostFlyAnim"));
}
