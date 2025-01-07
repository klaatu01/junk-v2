mod poisson;
mod system;
mod unav;

use bevy::window::PrimaryWindow;
use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_prototype_lyon::prelude::*;

pub use system::{System, SystemId};
pub use unav::Connection;
pub use unav::UNav;

pub struct UNavPlugin {
    unav: UNav,
}

impl UNavPlugin {
    pub fn generate(seed: u32) -> Self {
        UNavPlugin {
            unav: UNav::generate(seed),
        }
    }
}

#[derive(Resource)]
pub struct UNavToggle {
    pub active: bool,
}

#[derive(Event)]
pub struct ToggleUNav(pub bool);

impl Plugin for UNavPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        let central_system_id = self.unav.get_most_central_system().id.clone();
        app.insert_resource(self.unav.clone())
            .insert_resource(CurrentSystem(central_system_id))
            .insert_resource(MouseWorldCoords(Vec2::ZERO))
            .insert_resource(UNavToggle { active: false })
            .add_event::<ToggleUNav>()
            .add_event::<HoveredSystemEvent>()
            .add_event::<UnhoveredSystemEvent>()
            .add_plugins(ShapePlugin)
            .add_systems(Startup, setup_camera)
            .add_systems(Startup, spawn_systems)
            .add_systems(Startup, spawn_connections)
            .add_systems(Startup, spawn_system_info_ui)
            .add_systems(PostStartup, check_toggle)
            .add_systems(
                Update,
                (
                    set_zoom.run_if(|toggle: Res<UNavToggle>| toggle.active),
                    change_zoom.run_if(|toggle: Res<UNavToggle>| toggle.active),
                    mouse_world_pos_update.run_if(|toggle: Res<UNavToggle>| toggle.active),
                    hover_system.run_if(|toggle: Res<UNavToggle>| toggle.active),
                    on_hover_event.run_if(|toggle: Res<UNavToggle>| toggle.active),
                    on_unhover_event.run_if(|toggle: Res<UNavToggle>| toggle.active),
                ),
            )
            .add_systems(Update, set_visibility);
    }
}

#[derive(Component)]
struct UNavEntity;

fn check_toggle(unav_toggle: Res<UNavToggle>, mut query: Query<&mut Visibility, With<UNavEntity>>) {
    for mut visibility in query.iter_mut() {
        *visibility = if unav_toggle.active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        }
    }
}

fn set_visibility(
    mut toggle_unav: EventReader<ToggleUNav>,
    mut unav_toggle: ResMut<UNavToggle>,
    mut query: Query<&mut Visibility, With<UNavEntity>>,
) {
    for ToggleUNav(toggle) in toggle_unav.read() {
        for mut visibility in query.iter_mut() {
            *visibility = if *toggle {
                unav_toggle.active = true;
                Visibility::Visible
            } else {
                unav_toggle.active = false;
                Visibility::Hidden
            }
        }
    }
}

#[derive(Component)]
struct UNavCamera {
    zoom: f32,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        OrthographicProjection::default_2d(),
        Transform::from_xyz(0.0, 0.0, 1.0),
        UNavCamera { zoom: 0.5 },
        MainCamera,
    ));
}

fn set_zoom(mut query: Query<(&UNavCamera, &mut OrthographicProjection)>) {
    for (camera, mut projection) in query.iter_mut() {
        projection.scale = camera.zoom;
    }
}

// change zoom on mouse scroll with max 0.7 and min 0.05
fn change_zoom(mut query: Query<&mut UNavCamera>, mut scroll: EventReader<MouseWheel>) {
    for mut camera in query.iter_mut() {
        for event in scroll.read() {
            camera.zoom += event.y * 0.001;
            camera.zoom = camera.zoom.clamp(0.05, 0.7);
        }
    }
}

/// Marker so we can find and update this text later
#[derive(Component)]
pub struct SystemInfoText;

pub fn spawn_system_info_ui(mut commands: Commands) {
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            width: Val::Px(300.0),
            height: Val::Auto,
            top: Val::Px(20.0),
            right: Val::Px(20.0),
            padding: UiRect::all(Val::Px(10.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },))
        .with_children(|parent| {
            parent.spawn((Text::new("System Info"), SystemInfoText, UNavEntity));
        });
}

#[derive(Resource)]
pub struct CurrentSystem(pub SystemId);

#[derive(Component)]
pub struct SystemNode {
    pub id: SystemId,
}

#[derive(Component)]
pub struct HoveredSystem(pub SystemId);

fn spawn_systems(mut commands: Commands, unav: Res<UNav>) {
    for (_id, node) in unav.systems.iter() {
        let node_temperature = node.properties.temperature;
        commands.spawn((
            Sprite {
                color: Color::srgb(node_temperature as f32, 0.0, 1.0 - node_temperature as f32),
                custom_size: Some(Vec2::new(5.0, 5.0)),
                ..default()
            },
            Transform::from_xyz(node.position.x as f32, node.position.y as f32, 1.0),
            SystemNode { id: _id.clone() },
            UNavEntity,
        ));
    }
}

#[derive(Event)]
pub struct HoveredSystemEvent(pub SystemId);

#[derive(Event)]
pub struct UnhoveredSystemEvent;

fn hover_system(
    mouse_pos: Res<MouseWorldCoords>,
    unav: Res<UNav>,
    system_query: Query<(Entity, &SystemNode, Option<&HoveredSystem>)>,
    mut commands: Commands,
    mut hover_ew: EventWriter<HoveredSystemEvent>,
    mut unhover_ew: EventWriter<UnhoveredSystemEvent>,
) {
    for (entity, node, hovered) in system_query.iter() {
        let system = unav.systems.get(&node.id).unwrap();
        let distance = (system.position.x - mouse_pos.0.x as i64).abs()
            + (system.position.y - mouse_pos.0.y as i64).abs();
        if distance < 10 {
            if hovered.is_none() {
                commands
                    .entity(entity)
                    .insert_if_new(HoveredSystem(node.id.clone()));
                let _ = hover_ew.send(HoveredSystemEvent(node.id.clone()));
            }
        } else if hovered.is_some() {
            let _ = unhover_ew.send(UnhoveredSystemEvent);
            commands.entity(entity).remove::<HoveredSystem>();
        }
    }
}

fn on_hover_event(
    unav: Res<UNav>,
    mut hover_er: EventReader<HoveredSystemEvent>,
    mut query: Query<&mut Text, With<SystemInfoText>>,
) {
    for HoveredSystemEvent(system_id) in hover_er.read() {
        if let Some(system) = unav.get_system(system_id) {
            for mut text in &mut query {
                **text = system.properties.to_string();
            }
        }
    }
}

fn on_unhover_event(
    mut unhover_er: EventReader<UnhoveredSystemEvent>,
    mut query: Query<&mut Text, With<SystemInfoText>>,
) {
    for UnhoveredSystemEvent in unhover_er.read() {
        for mut text in &mut query {
            **text = "".to_string();
        }
    }
}

#[derive(Component)]
pub struct UNavConnectionLine;

fn spawn_connections(mut commands: Commands, unav: Res<UNav>) {
    let connections = unav.connections(35);

    for connection in connections.connections() {
        let from = &unav.systems[&connection.from];
        let to = &unav.systems[&connection.to];

        let from_pos = from.position;
        let to_pos = to.position;

        let line = shapes::Line(
            Vec2::new(from_pos.x as f32, from_pos.y as f32),
            Vec2::new(to_pos.x as f32, to_pos.y as f32),
        );

        let geo = GeometryBuilder::build_as(&line);

        commands.spawn((
            ShapeBundle {
                path: geo,
                ..default()
            },
            Stroke::new(Color::WHITE, 1.0),
            UNavConnectionLine,
            UNavEntity,
        ));
    }
}

#[derive(Resource, Default)]
pub struct MouseWorldCoords(pub Vec2);

#[derive(Component)]
pub struct MainCamera;

pub fn mouse_world_pos_update(
    mut mycoords: ResMut<MouseWorldCoords>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = q_camera.single();

    let window = q_window.single();

    if let Some(world_position) = window
        .cursor_position()
        .map(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.unwrap().origin.truncate())
    {
        mycoords.0 = world_position;
    }
}
