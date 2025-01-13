use bevy::{
    color::Color,
    math::I64Vec2,
    prelude::{BuildChildren, ChildBuild, Commands, Component, InheritedVisibility, Transform},
    sprite::Sprite,
};
use rand::Rng;

#[derive(Component)]
pub struct StarField {}

#[derive(Component)]
pub struct FieldComponent {
    pub points: Vec<I64Vec2>,
    pub distance: f32,
}

#[derive(Clone)]
pub struct Field {
    points: Vec<I64Vec2>,
    distance: f32,
}

impl Into<FieldComponent> for Field {
    fn into(self) -> FieldComponent {
        FieldComponent {
            points: self.points,
            distance: self.distance,
        }
    }
}

impl Field {
    pub fn generate(image_size: usize, distance: f32) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            points: crate::poisson::sample(
                image_size as isize,
                image_size as isize,
                200.0,
                90,
                rng.gen(),
            ),
            distance,
        }
    }
}

pub fn generate(image_size: usize, layers: usize) -> Vec<Field> {
    let fields: Vec<Field> = (0..layers + 1)
        .into_iter()
        .map(|layer| {
            let distance = layer as f32 / layers as f32;
            Field::generate(image_size, distance)
        })
        .collect();

    fields
}

pub fn spawn(commands: &mut Commands, fields: Vec<Field>) {
    commands
        .spawn((
            StarField {},
            InheritedVisibility::default(),
            Transform::default(),
        ))
        .with_children(|entity| {
            for field in fields {
                let field_component: FieldComponent = field.clone().into();
                entity
                    .spawn((
                        field_component,
                        Transform::default(),
                        InheritedVisibility::default(),
                    ))
                    .with_children(|entity| {
                        let distance = field.distance;
                        let size = distance * 4.0;
                        let custom_size = Some(bevy::math::Vec2 { x: size, y: size });
                        let color = Color::srgba(1.0, 0.95, 1.0 - (distance / 4.0), distance);
                        for point in field.points {
                            entity.spawn((
                                Sprite {
                                    custom_size,
                                    color,
                                    ..Default::default()
                                },
                                Transform::from_xyz(point.x as f32, point.y as f32, distance),
                                InheritedVisibility::default(),
                            ));
                        }
                    });
            }
        });
}

pub fn starfield_startup_system(mut commands: Commands) {
    let fields = generate(4096, 15);
    spawn(&mut commands, fields);
}
