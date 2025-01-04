use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufReader,
    rc::Rc,
};

use bevy::math::{I8Vec2, U8Vec2};
use rand::{
    rngs::StdRng,
    seq::{IteratorRandom, SliceRandom},
    Rng, SeedableRng,
};
use serde::{Deserialize, Serialize};

/// --------------------
/// PartType & Direction
/// --------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PartType {
    Cockpit,
    Hull,
    Cargo,
    Engine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn invert(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub fn to_vec2(&self) -> I8Vec2 {
        match self {
            Direction::Up => I8Vec2::new(0, 1),
            Direction::Down => I8Vec2::new(0, -1),
            Direction::Left => I8Vec2::new(-1, 0),
            Direction::Right => I8Vec2::new(1, 0),
        }
    }
}
/// ----------------------------------
/// PartInfo (Definition) + Connector
/// ----------------------------------
///
/// 'PartInfo' describes the blueprint for a part:
/// - how big (size),
/// - local-space connectors,
/// - local-space mount points (if any),
/// - etc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct PartInfo {
    pub id: usize,
    pub name: String,
    pub size: U8Vec2, // e.g. (1,2) if it occupies 2 vertical cells
    pub part_type: PartType,
    /// Local connector points:
    ///   key = local cell (x, y), value = Direction (e.g. "up")
    pub connector_points: HashMap<U8Vec2, Vec<Direction>>,
    /// Local mount points, e.g. weapon attachment points
    pub mount_points: HashSet<U8Vec2>,
}

/// For HashSet<PartInfo> usage, we only hash by 'id'.
impl std::hash::Hash for PartInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// -----------------------------------
/// ShipId - for uniquely identifying
/// -----------------------------------
const SHIP_ID_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const SHIP_ID_LENGTH: usize = 6;

#[derive(Debug)]
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

/// ---------------------------------------
/// PartInstance - A placed instance
/// ---------------------------------------
/// We now store:
///   - part_id
///   - anchor (the global coordinate of this part's bottom-left cell)
///   - orientation (optional, we default to North if you want to expand later)
#[derive(Debug)]
pub struct PartInstance {
    pub part_id: usize,
}

/// ----------------------------------------
/// Ship struct
/// ----------------------------------------
pub struct Ship {
    pub id: ShipId,
    /// Each occupied cell => the Rc<PartInstance> that occupies it
    pub cells: HashMap<I8Vec2, Rc<PartInstance>>,
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

        let mut ship = Ship::new(id);
        let sizes = [3, 10, 30, 50];
        ship.random(seed, parts, *sizes.choose(&mut rng).unwrap());
        ship
    }

    pub fn random(&mut self, seed: u64, parts: &HashSet<PartInfo>, parts_count: usize) {
        let mut rng = StdRng::seed_from_u64(seed);

        // Place cockpit
        let cockpit = parts
            .iter()
            .find(|p| p.part_type == PartType::Cockpit)
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
                    0 => PartType::Cargo,
                    _ => PartType::Hull,
                };

                if let Some(p) = Ship::find_part_with_direction(parts, inverted, target_part) {
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
            .find(|p| p.part_type == PartType::Engine)
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
        parts: &HashSet<PartInfo>,
        direction: Direction,
        type_filter: PartType,
    ) -> Option<&PartInfo> {
        parts.iter().find(|p| {
            p.connector_points.iter().any(|(_, directions)| {
                directions.contains(&direction) && p.part_type == type_filter
            })
        })
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
            .insert(position, Rc::new(PartInstance { part_id: part.id }));
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
                        match part_info.part_type {
                            PartType::Cockpit => "C",
                            PartType::Hull => "H",
                            PartType::Cargo => "O",
                            PartType::Engine => "E",
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

/// ----------------------------------------
/// Loading from RON
/// ----------------------------------------
fn load_parts_from_ron(path: &str) -> HashSet<PartInfo> {
    let file = File::open(path).expect("Could not open RON file.");
    let reader = BufReader::new(file);

    let part_list: Vec<PartInfo> =
        ron::de::from_reader(reader).expect("Failed to deserialize RON into PartInfo list.");

    part_list.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use super::*;

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
        let parts = load_parts_from_ron("example_parts.ron");
        for i in 0..100 {
            let ship = Ship::generate(i, &parts);
            ship.print_ascii(&parts);
        }
    }
}
