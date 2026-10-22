//! The `UITextField` stand-in: first-responder changes and the synchronous end-of-editing callback.

use bevy_ecs::prelude::*;
use pg_engine::platform::{self, TextField};

#[derive(Resource, Default)]
struct Ended(Vec<Entity>);

fn on_end(world: &mut World, delegate: Entity) {
    world.resource_mut::<Ended>().0.push(delegate);
}

#[test]
fn resigning_runs_did_end_editing_once() {
    let mut world = World::new();
    world.init_resource::<TextField>();
    world.init_resource::<Ended>();
    let delegate = world.spawn_empty().id();
    {
        let mut field = world.resource_mut::<TextField>();
        field.delegate = Some(delegate);
        field.did_end_editing = Some(on_end);
        field.text = "Old".to_owned();
    }

    platform::become_first_responder(&mut world);
    assert!(world.resource::<TextField>().first_responder);
    assert!(world.resource::<TextField>().text.is_empty());

    platform::resign_first_responder(&mut world);
    platform::resign_first_responder(&mut world);
    assert!(!world.resource::<TextField>().first_responder);
    assert_eq!(world.resource::<Ended>().0, vec![delegate]);
}
