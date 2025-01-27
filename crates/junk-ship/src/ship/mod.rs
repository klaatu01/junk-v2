use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Formatter},
};

use bevy::math::{I8Vec2, IVec2, UVec2};
use bevy_mesh::Mesh;
use cellular_automata::CellType;
use rand::{
    rngs::StdRng,
    seq::{IteratorRandom, SliceRandom},
    Rng, SeedableRng,
};

mod cellular_automata;

use crate::{mesh::MeshPart, parts::*};

pub const SHIP_ID_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const SHIP_ID_LENGTH: usize = 6;

#[derive(Debug, Clone)]
pub struct ShipId(pub String);

impl ShipId {
    pub fn generate(rng: &mut StdRng) -> Self {
        let id: String = (0..SHIP_ID_LENGTH)
            .map(|_| {
                let idx = rng.gen_range(0..SHIP_ID_ALPHABET.len()) as usize;
                SHIP_ID_ALPHABET.chars().nth(idx).unwrap()
            })
            .collect();
        Self(id)
    }

    pub fn player_ship() -> Self {
        Self("PLAYER".to_string())
    }
}

#[derive(Debug, Clone)]
pub struct PartInstance {
    pub part_id: usize,
}

#[derive(Debug)]
pub struct ShipMetrics {
    pub crew_capacity: usize,
    pub armor: usize,
    pub cargo_capacity: usize,
    pub thrust: usize,
    pub weight: usize,
}

impl ShipMetrics {
    pub fn acceleration(&self) -> f64 {
        self.thrust as f64 / self.weight as f64
    }
}

impl Display for ShipMetrics {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Crew: {}\nArmor: {}\nCargo: {}m³\nThrust: {}KN\nWeight: {}tons\nAcceleration: {:.2}m/s²",
            self.crew_capacity, self.armor, self.cargo_capacity, self.thrust, self.weight, self.acceleration()
        )
    }
}

/// ----------------------------------------
/// Ship struct
/// ----------------------------------------
#[derive(Debug, Clone)]
pub struct Ship {
    pub id: ShipId,
    pub cells: HashMap<I8Vec2, PartInstance>,
}

impl Ship {
    pub fn new(id: ShipId) -> Self {
        Self {
            id,
            cells: HashMap::new(),
        }
    }

    /// Generate a new Ship with a random "walk" approach.
    /// We'll place in the order: cockpit -> cargo/hull -> engine.
    pub fn generate(seed: u64, parts: &HashSet<PartInfo>) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let id = ShipId::generate(&mut rng);
        Ship::new(id).cellular(seed, parts)
    }

    pub fn cellular(mut self, seed: u64, parts: &HashSet<PartInfo>) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut automata = cellular_automata::Automata::new();
        automata.run(7);

        let cells = automata.get_non_empty();
        let lookup = cells.clone();

        let directions = |x: usize, y: usize| -> Vec<Direction> {
            let mut directions = Vec::new();
            if let Some(_) = lookup.get(&(x, y + 1)) {
                directions.push(Direction::Up);
            }
            if let Some(_) = lookup.get(&(x, y - 1)) {
                directions.push(Direction::Down);
            }
            if let Some(_) = lookup.get(&(x - 1, y)) {
                directions.push(Direction::Left);
            }
            if let Some(_) = lookup.get(&(x + 1, y)) {
                directions.push(Direction::Right);
            }
            directions
        };

        for (position, part) in cells.iter() {
            match part {
                CellType::Cockpit => {
                    self.place_part(
                        parts
                            .iter()
                            .find(|p| matches!(p.properties.part_type, PartType::Cockpit { .. }))
                            .unwrap(),
                        I8Vec2::new(
                            position.0.try_into().unwrap(),
                            position.1.try_into().unwrap(),
                        ),
                    );
                }
                CellType::Hull => {
                    let directions = directions(position.0, position.1);
                    let part = Ship::find_parts_with_only_directions(parts, directions);
                    if !part.is_empty() {
                        let part = part.choose(&mut rng).unwrap();
                        self.place_part(
                            part,
                            I8Vec2::new(
                                position.0.try_into().unwrap(),
                                position.1.try_into().unwrap(),
                            ),
                        );
                    } else {
                        let part = Ship::find_parts_with_only_directions(
                            parts,
                            vec![
                                Direction::Up,
                                Direction::Down,
                                Direction::Left,
                                Direction::Right,
                            ],
                        );
                        let part = part.choose(&mut rng).unwrap();
                        self.place_part(
                            part,
                            I8Vec2::new(
                                position.0.try_into().unwrap(),
                                position.1.try_into().unwrap(),
                            ),
                        );
                    }
                }
                _ => {}
            }
        }

        self
    }

    pub fn random(&mut self, seed: u64, parts: &HashSet<PartInfo>, parts_count: usize) {
        let mut rng = StdRng::seed_from_u64(seed);

        // Place cockpit
        let cockpit = parts
            .iter()
            .find(|p| matches!(p.properties.part_type, PartType::Cockpit { .. }))
            .unwrap();

        self.place_part(cockpit, I8Vec2::new(0, 0));

        for _ in 0..parts_count {
            let mut current = I8Vec2::new(0, 0);
            let mut direction: Direction;

            loop {
                let directions = self.get_directions(current, parts);
                direction = *directions.choose(&mut rng).unwrap();
                let position = current + direction.to_vec2();

                if self.check_position_taken(position) {
                    current = position;
                    continue;
                } else {
                    break;
                }
            }

            let inverted = direction.invert();
            let part: PartInfo;
            loop {
                let target_part = match rng.gen_range(0..2) {
                    0 => |x: &PartType| matches!(x, PartType::Cargo { .. }),
                    _ => |x: &PartType| matches!(x, PartType::Hull { .. }),
                };

                if let Some(p) =
                    Ship::find_part_with_direction(&mut rng, parts, inverted, target_part)
                {
                    part = p.clone();
                    break;
                } else {
                    continue;
                }
            }

            let position = current + direction.to_vec2();

            self.place_part(&part, position);
        }

        let engine = parts
            .iter()
            .find(|p| matches!(p.properties.part_type, PartType::Engine { .. }))
            .unwrap();

        // find min y for each x so we can place an engine on the bottom of each 'column'
        let mut min_y = HashMap::new();
        for (position, part) in self.cells.iter() {
            let part_info = parts.iter().find(|p| p.id == part.part_id).unwrap();
            let y = position.y - part_info.size.y as i8;
            if let Some(min) = min_y.get(&position.x) {
                if y < *min {
                    min_y.insert(position.x, y);
                }
            } else {
                min_y.insert(position.x, y);
            }
        }

        let engine_count = rng.gen_range((min_y.len() / 2).max(1)..=min_y.len());

        for _ in 0..engine_count {
            let x = *min_y.keys().choose(&mut rng).unwrap();
            let y = min_y.get(&x).unwrap();
            self.place_part(engine, I8Vec2::new(x, *y));
        }
    }

    pub fn check_position_taken(&self, position: I8Vec2) -> bool {
        self.cells.contains_key(&position)
    }

    pub fn find_part_with_direction(
        rand: &mut StdRng,
        parts: &HashSet<PartInfo>,
        direction: Direction,
        type_filter: impl Fn(&PartType) -> bool,
    ) -> Option<PartInfo> {
        let parts = parts
            .iter()
            .filter(
                // not a cockpit
                |p| !matches!(p.properties.part_type, PartType::Cockpit { .. }),
            )
            .filter(|p| {
                p.connector_points.iter().any(|(_, directions)| {
                    directions.contains(&direction) && type_filter(&p.properties.part_type)
                })
            });
        let part = parts.choose(rand);
        part.cloned()
    }

    pub fn find_parts_with_only_directions(
        parts: &HashSet<PartInfo>,
        directions: Vec<Direction>,
    ) -> Vec<PartInfo> {
        let parts = parts
            .iter()
            .filter(|p| {
                p.connector_points.iter().all(|(_, part_directions)| {
                    part_directions.iter().all(|d| directions.contains(d))
                })
            })
            .filter(|p| {
                // not a cockpit
                !matches!(p.properties.part_type, PartType::Cockpit { .. })
            });
        parts.cloned().collect()
    }

    pub fn get_directions(&self, current: I8Vec2, parts: &HashSet<PartInfo>) -> Vec<Direction> {
        let mut directions = Vec::new();
        let current_part = self.cells.get(&current).unwrap();
        let part_info = parts
            .iter()
            .find(|p| p.id == current_part.part_id)
            .unwrap()
            .clone();

        for (_, direction) in part_info.connector_points.iter() {
            directions.extend(direction);
        }

        directions
    }

    pub fn place_part(&mut self, part: &PartInfo, position: I8Vec2) {
        self.cells
            .insert(position, PartInstance { part_id: part.id });
    }

    pub fn metrics(&self, parts: &HashSet<PartInfo>) -> ShipMetrics {
        let mut crew_capacity = 0;
        let mut armor = 0;
        let mut cargo_capacity = 0;
        let mut thrust = 0;
        let mut weight = 0;

        for (_, part) in self.cells.iter() {
            let part_info = parts.iter().find(|p| p.id == part.part_id).unwrap();
            let properties = &part_info.properties;
            weight += properties.weight;
            match properties.part_type {
                PartType::Cockpit { crew_capacity: c } => crew_capacity += c,
                PartType::Hull {
                    armor: a,
                    cargo_capacity: c,
                } => {
                    armor += a;
                    cargo_capacity += c;
                }
                PartType::Cargo { cargo_capacity: c } => cargo_capacity += c,
                PartType::Engine { thrust: t } => thrust += t,
            }
        }

        ShipMetrics {
            crew_capacity,
            armor,
            cargo_capacity,
            thrust,
            weight,
        }
    }

    pub fn mesh(&self, parts: &HashSet<PartInfo>) -> Mesh {
        let mut mesh_parts = Vec::new();

        for (position, part) in self.cells.iter() {
            let part_info = parts.iter().find(|p| p.id == part.part_id).unwrap();
            let size = UVec2::new(part_info.size.x as u32, part_info.size.y as u32);
            let uv_position = UVec2::new(part_info.uv.0, part_info.uv.1);
            let uv_size = UVec2::new(part_info.uv.2, part_info.uv.3);
            let position = IVec2::new(position.x as i32, position.y as i32);

            mesh_parts.push(MeshPart {
                position,
                size,
                uv_position,
                uv_size,
            });
        }

        crate::mesh::generate_mesh(mesh_parts)
    }

    pub fn print_ascii(&self, parts: &HashSet<PartInfo>) {
        let mut min_x = 0;
        let mut max_x = 0;
        let mut min_y = 0;
        let mut max_y = 0;

        for (position, part) in self.cells.iter() {
            let part_info = parts.iter().find(|p| p.id == part.part_id).unwrap();
            min_x = min_x.min(position.x);
            max_x = max_x.max(position.x + part_info.size.x as i8);
            min_y = min_y.min(position.y);
            max_y = max_y.max(position.y + part_info.size.y as i8);
        }

        for y in (min_y..=max_y).rev() {
            for x in min_x..=max_x {
                let part = self.cells.get(&I8Vec2::new(x, y));
                if let Some(part) = part {
                    let part_info = parts.iter().find(|p| p.id == part.part_id).unwrap();
                    print!(
                        "{}",
                        match part_info.properties.part_type {
                            PartType::Cockpit { .. } => "C",
                            PartType::Hull { .. } => "H",
                            PartType::Cargo { .. } => "O",
                            PartType::Engine { .. } => "E",
                        }
                    );
                } else {
                    print!(" ");
                }
            }
            println!();
        }
    }
}
