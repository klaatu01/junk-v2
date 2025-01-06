use std::collections::HashSet;

use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext, LoadedFolder},
    prelude::*,
    sprite::Material2dPlugin,
};

mod mesh;
mod outline;
mod parts;
mod ship;

use outline::SpriteOutlineMaterial;
pub use parts::*;
pub use ship::*;

#[derive(Asset, TypePath, Debug)]
pub struct PartsAsset {
    pub name: String,
    pub parts: Parts,
}

#[derive(Default)]
pub struct PartsAssetLoader;

impl AssetLoader for PartsAssetLoader {
    type Asset = PartsAsset;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let parts = Parts::load_parts_from_bytes(&bytes);
        let name = load_context.path().to_str().unwrap().to_string();
        Ok(PartsAsset { parts, name })
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(Resource)]
pub struct PartsHandleState {
    pub handle: Handle<LoadedFolder>,
}

#[derive(Resource, Default)]
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

    pub(crate) fn add_parts(&mut self, parts: &HashSet<PartInfo>) {
        self.parts.extend(parts.clone());
    }
}

pub struct ShipPlugin;

impl bevy::app::Plugin for ShipPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_event::<SpawnShipEvent>()
            .init_resource::<PartsResource>()
            .init_asset::<PartsAsset>()
            .init_asset_loader::<PartsAssetLoader>()
            .add_plugins(Material2dPlugin::<SpriteOutlineMaterial>::default())
            .add_systems(PostStartup, setup)
            .add_systems(Update, ship_spawner)
            .add_systems(Update, load_parts_resource)
            .add_systems(Update, player_startup);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("Loading parts assets");
    let handle = asset_server.load_folder("parts");
    commands.insert_resource(PartsHandleState { handle });
    println!("Loaded parts assets");
}

fn load_parts_resource(
    mut parts_resource: ResMut<PartsResource>,
    mut parts_assets_event: EventReader<AssetEvent<PartsAsset>>,
    assets: ResMut<Assets<PartsAsset>>,
) {
    for event in parts_assets_event.read() {
        if let AssetEvent::Added { id } = event {
            let asset = assets.get(*id).unwrap();
            println!("Adding parts from asset {}", asset.name);
            parts_resource.add_parts(&asset.parts.parts);
        }
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SpriteOutlineMaterial>>,
    mut spawn_ship_event: EventReader<SpawnShipEvent>,
    asset_server: Res<AssetServer>,
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
        build_ship(
            &mut entity_commands,
            &mut meshes,
            &mut materials,
            &asset_server,
            parts_resource.all_parts(),
            &ship,
        );
    }
}

fn build_ship(
    entity_commands: &mut EntityCommands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<SpriteOutlineMaterial>>,
    asset_server: &Res<AssetServer>,
    parts: &HashSet<PartInfo>,
    ship: &Ship,
) {
    let ship_mesh = ship.mesh(parts);
    let mesh = meshes.add(ship_mesh);

    let texture_handle = asset_server.load("textures/ship_dev.png");

    let material = materials.add(SpriteOutlineMaterial {
        color: Vec4::new(1.0, 1.0, 1.0, 1.0),         // White tint
        outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0), // Black outline
        outline_thickness: 0.005,                     // Adjust based on texture size
        main_texture: texture_handle.clone(),
    });

    entity_commands.with_child((Mesh2d(mesh), MeshMaterial2d(material)));
}

fn player_startup(
    input: Res<ButtonInput<KeyCode>>,
    mut spawn_ship_event: EventWriter<SpawnShipEvent>,
) {
    if input.just_pressed(KeyCode::Enter) {
        spawn_ship_event.send(SpawnShipEvent {
            player: true,
            position: Vec2::new(0.0, 0.0),
            seed: 15,
        });
    }
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
