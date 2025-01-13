mod poisson;
mod stars;

use bevy::{app::Startup, prelude::Plugin};
use stars::starfield_startup_system;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, starfield_startup_system);
    }
}
