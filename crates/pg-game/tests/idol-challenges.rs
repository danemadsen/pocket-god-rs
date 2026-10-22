#![allow(non_snake_case)]

use bevy_app::App;
use bevy_ecs::prelude::*;
use bevy_math::Vec4;
use pg_engine::display;
use pg_engine::library;
use pg_engine::logic;
use pg_engine::object::{Isa, ObjectWorldExt};
use pg_engine::sequence;
use pg_engine::touch::{self, PlatformTouch, TouchPhase};
use pg_engine::view::{self, BcView};
use pg_game::controller::{self, ControllerBootAndFrame};
use pg_game::logic::idol_screen::idol_screen_logic::{self, IdolScreenLogic};
use pg_game::managers::idol_manager;

fn newChallengesWorld(width: i32, height: i32) -> (World, Entity, Entity) {
    let mut app = App::new();
    app.add_plugins(pg_engine::EnginePlugin);
    let mut world = std::mem::take(app.world_mut());
    view::configure_screen(&mut world, width, height, width as f32, height as f32);

    let controller = world
        .spawn(Isa(&controller::POCKET_GOD_VIEW_CONTROLLER))
        .id();
    controller::init(&mut world, controller);
    controller::load_game(&mut world, controller);

    // Exercise the real screen and input stack without continuing the unrelated boot UI.
    // The default preference backend keeps all writes in this world's memory.
    let boot_sequence = world
        .obj::<ControllerBootAndFrame>(controller)
        .sequence
        .unwrap();
    sequence::stop(&mut world, boot_sequence);
    let game_root = library::display_group(&world, "PocketGod");
    view::set_active_display_group(&mut world, game_root);
    world.resource_mut::<BcView>().touch_active = true;

    let manager = idol_manager::instance(&world).unwrap();
    idol_manager::open_idol_list_screen(&mut world, manager);
    let screen = idol_screen_logic::instance(&world).unwrap();
    advanceFrames(&mut world, 60);

    (world, screen, manager)
}

fn advanceFrames(world: &mut World, frame_count: usize) {
    for _ in 0..frame_count {
        view::game_loop(world, 1.0 / 60.0);
    }
}

fn iconCenter(world: &mut World, screen: Entity, episode: usize) -> Vec4 {
    let icon = idol_manager::episode_table(world).nodes[episode]
        .idol_icon_logic
        .unwrap();
    let icon_object = logic::display_object(world, icon);
    let list_group = world
        .obj::<IdolScreenLogic>(screen)
        .idol_icon_list_display_group
        .unwrap();
    let screen_object = logic::display_object(world, screen);
    let screen_group = display::object(world, screen_object).display_group;

    // The list is drawn through a marker; its hit-testing parent is attached for each tap.
    display::object_mut(world, list_group).display_group = screen_group;
    let local_center = Vec4::new(50.0, 50.0, 0.0, 1.0);
    let center = display::display_object_to_view(world, icon_object, local_center);
    display::object_mut(world, list_group).display_group = None;
    center
}

fn sendTouch(world: &mut World, phase: TouchPhase, position: Vec4) {
    let platform_touch = PlatformTouch {
        id: 1,
        phase,
        time_stamp: 1.0,
        tap_count: 1,
        x: position.x,
        y: position.y,
    };
    touch::create_touch_event(world, platform_touch);
    touch::process_touch_events(world, 1.0);
}

fn offsetInScreen(
    world: &World,
    screen: Entity,
    position: Vec4,
    x_offset: f32,
    y_offset: f32,
) -> Vec4 {
    let screen_object = logic::display_object(world, screen);
    let screen_group = display::object(world, screen_object).display_group.unwrap();
    let mut logical_position = display::view_to_display_object(world, screen_group, position);
    logical_position.x += x_offset;
    logical_position.y += y_offset;
    display::display_object_to_view(world, screen_group, logical_position)
}

#[test]
fn tapsSelectAndActivateEpisodesAcrossLayouts() {
    let layouts = [(480, 320), (960, 640), (568, 320), (1024, 768)];
    let movements = [None, Some((0.0, 0.0)), Some((1.0, 1.0))];
    for (width, height) in layouts {
        for movement in movements {
            let (mut world, screen, manager) = newChallengesWorld(width, height);
            let episode = 1;
            let center = iconCenter(&mut world, screen, episode);
            sendTouch(&mut world, TouchPhase::Began, center);
            let release_position = match movement {
                Some((x_offset, y_offset)) => {
                    let position = offsetInScreen(&world, screen, center, x_offset, y_offset);
                    sendTouch(&mut world, TouchPhase::Moved, position);
                    position
                }
                None => center,
            };
            sendTouch(&mut world, TouchPhase::Ended, release_position);

            let state = world.obj::<IdolScreenLogic>(screen);
            assert_eq!(
                state.idol_screen_mode, 2,
                "layout {width}x{height}, movement {movement:?}"
            );
            assert_eq!(state.struct_xml_episode_node_def, Some(episode));
            assert_eq!(
                idol_manager::active_episode_node_def(&mut world, manager),
                None
            );

            advanceFrames(&mut world, 60);
            let button = world
                .obj::<IdolScreenLogic>(screen)
                .idol_detail_panel_challenge_button_logic
                .unwrap();
            let button_object = logic::display_object(&world, button);
            let button_bounds = display::object(&world, button_object);
            let local_center = Vec4::new(
                (button_bounds.col_x_min + button_bounds.col_x_max) * 0.5,
                (button_bounds.col_y_min + button_bounds.col_y_max) * 0.5,
                0.0,
                1.0,
            );
            let button_center =
                display::display_object_to_view(&world, button_object, local_center);
            sendTouch(&mut world, TouchPhase::Began, button_center);
            sendTouch(&mut world, TouchPhase::Ended, button_center);

            let active_episode = idol_manager::active_episode_node_def(&mut world, manager);
            assert_eq!(
                active_episode,
                Some(episode),
                "Activate button must start the selected episode"
            );
        }
    }
}

#[test]
fn smallMovesAccumulateIntoScrollingAndDoNotSelectAnEpisode() {
    let (mut world, screen, manager) = newChallengesWorld(960, 640);
    let episode = 0;
    let center = iconCenter(&mut world, screen, episode);
    let list_group = world
        .obj::<IdolScreenLogic>(screen)
        .idol_icon_list_display_group
        .unwrap();
    let initial_y_pos = display::object(&world, list_group).pos.y;
    sendTouch(&mut world, TouchPhase::Began, center);

    // Each individual movement is small, but the total eventually becomes a scroll.
    for y_offset in [2.0, 4.0, 6.0, 8.0] {
        let position = offsetInScreen(&world, screen, center, 0.0, y_offset);
        sendTouch(&mut world, TouchPhase::Moved, position);
        assert!(!world.obj::<IdolScreenLogic>(screen).drag_panel);
        assert_eq!(display::object(&world, list_group).pos.y, initial_y_pos);
    }

    let drag_position = offsetInScreen(&world, screen, center, 0.0, 12.0);
    sendTouch(&mut world, TouchPhase::Moved, drag_position);
    assert!(world.obj::<IdolScreenLogic>(screen).drag_panel);
    let scrolled_y_pos = display::object(&world, list_group).pos.y;
    assert!((scrolled_y_pos - initial_y_pos - 12.0).abs() < 0.001);

    // Returning to the starting point after a drag must still suppress the tap.
    sendTouch(&mut world, TouchPhase::Moved, center);
    sendTouch(&mut world, TouchPhase::Ended, center);

    let state = world.obj::<IdolScreenLogic>(screen);
    assert_eq!(state.idol_screen_mode, 1);
    assert_eq!(state.struct_xml_episode_node_def, None);
    assert_eq!(
        idol_manager::active_episode_node_def(&mut world, manager),
        None
    );
}

#[test]
fn tapSelectsANewEpisodeAfterScrollingTheList() {
    let (mut world, screen, _) = newChallengesWorld(568, 320);
    let initial_center = iconCenter(&mut world, screen, 0);
    let drag_position = offsetInScreen(&world, screen, initial_center, 0.0, 130.0);
    sendTouch(&mut world, TouchPhase::Began, initial_center);
    sendTouch(&mut world, TouchPhase::Moved, drag_position);
    sendTouch(&mut world, TouchPhase::Ended, drag_position);
    advanceFrames(&mut world, 120);
    assert_eq!(world.obj::<IdolScreenLogic>(screen).idol_screen_mode, 1);

    let screen_object = logic::display_object(&world, screen);
    let screen_group = display::object(&world, screen_object)
        .display_group
        .unwrap();
    let mut next_episode = None;
    for episode in 3..idol_manager::EPISODE_LIST_LEN {
        let icon = idol_manager::episode_table(&mut world).nodes[episode].idol_icon_logic;
        if icon.is_none() {
            continue;
        }

        let center = iconCenter(&mut world, screen, episode);
        let logical_center = display::view_to_display_object(&world, screen_group, center);
        if logical_center.y > 80.0 && logical_center.y < 240.0 {
            next_episode = Some((episode, center));
            break;
        }
    }

    let (episode, center) =
        next_episode.expect("scrolling must populate another visible episode row");
    let jitter_position = offsetInScreen(&world, screen, center, 1.0, 1.0);
    sendTouch(&mut world, TouchPhase::Began, center);
    sendTouch(&mut world, TouchPhase::Moved, jitter_position);
    sendTouch(&mut world, TouchPhase::Ended, jitter_position);

    let state = world.obj::<IdolScreenLogic>(screen);
    assert_eq!(state.idol_screen_mode, 2);
    assert_eq!(state.struct_xml_episode_node_def, Some(episode));
}
