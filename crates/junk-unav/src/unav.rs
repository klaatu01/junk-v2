use crate::{System, SystemId};
use bevy_ecs::system::Resource;
use nanoid::nanoid;
use rand::SeedableRng;
use std::collections::HashMap;

const X_MAX: isize = 256;
const Y_MAX: isize = 256;

pub struct Connection {
    pub from: SystemId,
    pub to: SystemId,
    pub distance: usize,
}

pub struct Connections {
    pub connections: Vec<Connection>,
}

impl Connections {
    pub fn new(connections: Vec<Connection>) -> Connections {
        Connections { connections }
    }

    pub fn can_navigate_to(&self, from: &SystemId, to: &SystemId) -> bool {
        self.connections
            .iter()
            .any(|connection| connection.from == *from && connection.to == *to)
    }

    pub fn get_navigatable_systems(&self, from: &SystemId) -> Vec<SystemId> {
        self.connections
            .iter()
            .filter(|connection| connection.from == *from)
            .map(|connection| connection.to.clone())
            .collect()
    }

    pub fn connections(&self) -> &Vec<Connection> {
        &self.connections
    }
}

#[derive(Resource)]
pub struct UNav {
    pub systems: HashMap<SystemId, System>,
}

impl UNav {
    pub fn generate(random_seed: u32) -> UNav {
        let positions = crate::poisson::sample(X_MAX, Y_MAX, 20.0, 30, random_seed as u64);
        let mut system_propery_rng = rand::rngs::StdRng::seed_from_u64(random_seed as u64);
        let systems = positions
            .into_iter()
            .map(|point| {
                let id = SystemId(nanoid!());
                (
                    id.clone(),
                    System {
                        id,
                        position: (point.x - X_MAX / 2, point.y - Y_MAX / 2),
                        properties: crate::system::SystemProperties::from_rngs(
                            &mut system_propery_rng,
                        ),
                    },
                )
            })
            .collect();
        UNav { systems }
    }

    pub fn connections(&self, distance_filter: usize) -> Connections {
        let mut connections = Vec::new();
        for (from_id, from_node) in self.systems.iter() {
            for (to_id, to_node) in self.systems.iter() {
                if from_id != to_id {
                    let distance = ((from_node.position.0 - to_node.position.0).abs()
                        + (from_node.position.1 - to_node.position.1).abs())
                        as usize;
                    if distance <= distance_filter {
                        connections.push(Connection {
                            from: from_id.clone(),
                            to: to_id.clone(),
                            distance,
                        });
                    }
                }
            }
        }
        Connections::new(connections)
    }

    pub fn get_system(&self, id: &SystemId) -> Option<&System> {
        self.systems.get(id)
    }

    pub fn get_most_central_system(&self) -> &System {
        self.systems
            .values()
            .min_by_key(|system| system.position.0.abs() + system.position.1.abs())
            .unwrap()
    }
}
