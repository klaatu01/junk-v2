use bevy::prelude::*;
use junk_ship::{ShipPlugin, SpawnShipEvent};
use junk_unav::{ToggleUNav, UNavPlugin};

#[derive(Resource, Clone)]
pub enum Focus {
    UNav,
    Game,
}

#[derive(Event)]
pub struct FocusChanged {
    pub from: Focus,
    pub to: Focus,
}

fn main() {
    App::new()
        .insert_resource(Focus::Game)
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(UNavPlugin::generate(19940131))
        .add_plugins(ShipPlugin)
        .add_event::<FocusChanged>()
        .add_systems(Update, focus_systems)
        .add_systems(Update, on_focus_changed)
        .run();
}

fn focus_systems(
    mut focus: ResMut<Focus>,
    input: Res<ButtonInput<KeyCode>>,
    mut focus_changed: EventWriter<FocusChanged>,
) {
    let focus_change = if input.just_pressed(KeyCode::KeyU) {
        match *focus {
            Focus::UNav => Some(Focus::Game),
            Focus::Game => Some(Focus::UNav),
        }
    } else {
        None
    };

    if let Some(new_focus) = focus_change {
        focus_changed.send(FocusChanged {
            from: focus.clone(),
            to: new_focus.clone(),
        });
        *focus = new_focus;
    }
}

fn on_focus_changed(
    mut focus_changed: EventReader<FocusChanged>,
    mut toggle_unav: EventWriter<ToggleUNav>,
) {
    for event in focus_changed.read() {
        match event.to {
            Focus::UNav => {
                toggle_unav.send(ToggleUNav(true));
            }
            _ => {
                toggle_unav.send(ToggleUNav(false));
            }
        }
    }
}
