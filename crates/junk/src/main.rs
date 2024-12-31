use std::fmt::Write;

use bevy::prelude::*;
use bevy::{input::mouse::MouseWheel, prelude::*, sprite::Anchor};
use bevy_prototype_lyon::prelude::*;
use junk_unav::{SystemId, UNav};
use mouse::{MainCamera, MouseWorldCoords};
use pyri_tooltip::Tooltip;
mod mouse;

fn main() {
    let unav = UNav::generate(0);

    let central_system_id = unav.get_most_central_system().id.clone();

    App::new()
        .insert_resource(unav)
        .insert_resource(CurrentSystem(central_system_id))
        .insert_resource(MouseWorldCoords(Vec2::ZERO))
        .add_plugins(pyri_tooltip::TooltipPlugin::default())
        .add_event::<HoveredSystemEvent>()
        .add_event::<UnhoveredSystemEvent>()
        .add_plugins(DefaultPlugins)
        .add_plugins(ShapePlugin)
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, spawn_systems)
        .add_systems(Startup, spawn_connections)
        .add_systems(Startup, spawn_system_info_ui)
        .add_systems(Update, set_zoom)
        .add_systems(Update, change_zoom)
        .add_systems(Update, mouse::mouse_world_pos_update)
        .add_systems(Update, hover_system)
        .add_systems(Update, on_hover_event)
        .add_systems(Update, on_unhover_event)
        .run();
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

pub fn spawn_system_info_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            width: Val::Px(300.0),
            height: Val::Auto,
            bottom: Val::Px(20.0),
            right: Val::Px(20.0),
            padding: UiRect::all(Val::Px(10.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },))
        .with_children(|parent| {
            parent.spawn((Text::new("System Info"), SystemInfoText));
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

/// Spawns a small sprite at the position of each system in the UNav.
/// In Bevy’s coordinate space, `x` goes right, `y` goes up (by default).
fn spawn_systems(mut commands: Commands, unav: Res<UNav>) {
    // For example, if `unav.systems` is a HashMap<SystemID, SystemNode>
    for (_id, node) in unav.systems.iter() {
        let node_temperature = node.properties.temperature;
        commands.spawn((
            Sprite {
                color: Color::rgb(node_temperature as f32, 0.0, 1.0 - node_temperature as f32),
                custom_size: Some(Vec2::new(5.0, 5.0)),
                ..default()
            },
            Transform::from_xyz(node.position.0 as f32, node.position.1 as f32, 1.0),
            SystemNode { id: _id.clone() },
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
        let distance = (system.position.0 - mouse_pos.0.x as isize).abs()
            + (system.position.1 - mouse_pos.0.y as isize).abs();
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
                **text = format!(
                    "System: {}\nTemperature: {:.2}\nDetected:\n{}",
                    system.properties.name.0,
                    system.properties.temperature,
                    system
                        .properties
                        .r#type
                        .iter()
                        .map(|c| format!(" - {:?}", c))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
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
    // Suppose your `connections(distance_filter)` returns a Vec<Connection>.
    let connections = unav.connections(37);

    for connection in connections.connections() {
        let from = &unav.systems[&connection.from];
        let to = &unav.systems[&connection.to];

        let from_pos = from.position;
        let to_pos = to.position;

        let line = shapes::Line(
            Vec2::new(from_pos.0 as f32, from_pos.1 as f32),
            Vec2::new(to_pos.0 as f32, to_pos.1 as f32),
        );

        let geo = GeometryBuilder::build_as(&line);

        commands.spawn((
            ShapeBundle {
                path: geo,
                ..default()
            },
            Stroke::new(Color::WHITE, 1.0),
            UNavConnectionLine,
        ));
    }
}
