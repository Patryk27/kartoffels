use crate::{Event, Events};
use bevy_ecs::event::EventReader;
use bevy_ecs::system::ResMut;

pub fn track(
    events: Option<ResMut<Events>>,
    mut new_events: EventReader<Event>,
) {
    let Some(mut events) = events else {
        return;
    };

    for event in new_events.read() {
        events.pending.push(*event);
    }
}
