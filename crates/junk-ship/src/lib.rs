use std::collections::HashSet;

use bevy::{
    asset::{AssetLoader, AssetLoaderError},
    prelude::*,
};

mod parts;
mod ship;

pub use parts::*;
pub use ship::*;

pub struct PartConfigAsset {
    pub parts: Vec<PartInfo>,
}

#[derive(Resource)]
pub struct PartsHandles {
    pub parts: Vec<Handle<PartConfigAsset>>,
}

#[derive(Resource)]
pub struct PartsResource {
    parts: HashSet<PartInfo>,
}

impl PartsResource {
    pub fn load() -> Self {
        Self {
            parts: HashSet::new(),
        }
    }

    pub fn all_parts(&self) -> &HashSet<PartInfo> {
        &self.parts
    }
}

pub struct PartsLoader;

impl AssetLoader for PartsLoader {
    type Asset = PartsResource;

    fn from_bytes(
        &self,
        _asset_path: &AssetPath,
        bytes: Vec<u8>,
    ) -> Result<PartsResource, AssetLoaderError> {
        let parts = Parts::load_parts_from_bytes(&bytes);
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

pub struct ShipPlugin;

impl bevy::app::Plugin for ShipPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        let parts_resource = PartsResource::load();
        app.insert_resource(parts_resource)
            .add_event::<SpawnShipEvent>()
            .add_systems(PostStartup, player_startup)
            .add_systems(Update, ship_spawner);
    }
}

#[derive(Event)]
pub struct SpawnShipEvent {
    pub player: bool,
    pub position: Vec2,
    pub seed: u64,
}

#[derive(Component)]
pub struct ShipComponent {
    pub ship: Ship,
}

#[derive(Component)]
pub struct PlayerShip;

#[derive(Component)]
pub struct PartInfoComponent {
    pub part: PartInfo,
}

fn ship_spawner(
    mut commands: Commands,
    parts_resource: Res<PartsResource>,
    mut spawn_ship_event: EventReader<SpawnShipEvent>,
) {
    for event in spawn_ship_event.read() {
        let ship = Ship::generate(event.seed, parts_resource.all_parts());
        let ship_component = ShipComponent { ship: ship.clone() };
        let transform =
            Transform::from_translation(Vec3::new(event.position.x, event.position.y, 0.0));
        let mut entity_commands = commands.spawn((ship_component, transform));
        if event.player {
            entity_commands.insert(PlayerShip);
        }
        build_ship(&mut entity_commands, parts_resource.all_parts(), &ship);
    }
}

fn build_ship(entity_commands: &mut EntityCommands, parts: &HashSet<PartInfo>, ship: &Ship) {
    for (position, part_id) in &ship.cells {
        let part = parts.iter().find(|p| p.id == part_id.part_id).unwrap();
        let transform =
            Transform::from_translation(Vec3::new(position.x as f32, position.y as f32, 0.0));
        let sprite = Sprite {
            custom_size: Some(Vec2::new(part.size.x as f32, part.size.y as f32)),
            ..Default::default()
        };
        entity_commands.with_child((PartInfoComponent { part: part.clone() }, transform, sprite));
    }
}

fn player_startup(mut spawn_ship_event: EventWriter<SpawnShipEvent>) {
    spawn_ship_event.send(SpawnShipEvent {
        player: true,
        position: Vec2::new(0.0, 0.0),
        seed: 0,
    });
}

#[cfg(test)]
mod tests {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    use crate::parts::*;
    use crate::ship::*;

    #[test]
    fn test_ship_id_generate() {
        let mut rng = StdRng::seed_from_u64(0);
        let ship_id = ShipId::generate(&mut rng);
        assert_eq!(ship_id.0.len(), SHIP_ID_LENGTH);
    }

    #[test]
    fn test_ship_id_player_ship() {
        let ship_id = ShipId::player_ship();
        assert_eq!(ship_id.0, "PLAYER");
    }

    #[test]
    fn test_ship() {
        let parts = Parts::load_parts_from_ron("parts.ron");
        assert_eq!(parts.sprite_sheet, "stock.png");

        for i in 0..10 {
            let ship = Ship::generate(i, &parts.parts);
            let metrics = ship.metrics(&parts.parts);
            println!("{:?}", ship.id);
            println!("{}", metrics);
            ship.print_ascii(&parts.parts);
        }
    }
}
