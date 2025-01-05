use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufReader, Read},
};

use bevy::math::{I8Vec2, U8Vec2};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PartType {
    Cockpit { crew_capacity: usize },
    Hull { armor: usize, cargo_capacity: usize },
    Cargo { cargo_capacity: usize },
    Engine { thrust: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PartProperties {
    pub part_type: PartType,
    pub weight: usize,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct PartInfo {
    pub id: usize,
    pub name: String,
    pub size: U8Vec2,
    pub properties: PartProperties,
    pub connector_points: HashMap<U8Vec2, Vec<Direction>>,
    pub mount_points: HashSet<U8Vec2>,
    pub sprite_sheet: Option<String>,
    pub uv: (u32, u32, u32, u32),
}

/// For HashSet<PartInfo> usage, we only hash by 'id'.
impl std::hash::Hash for PartInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parts {
    pub sprite_sheet: String,
    pub parts: HashSet<PartInfo>,
}

impl Parts {
    pub fn get_part(&self, id: usize) -> Option<&PartInfo> {
        self.parts.iter().find(|p| p.id == id)
    }

    pub fn load_parts_from_bytes(bytes: &[u8]) -> Parts {
        let mut parts: Parts =
            ron::de::from_bytes(bytes).expect("Failed to deserialize RON into PartInfo list.");

        let sprite_sheet = parts.sprite_sheet.clone();

        let enriched_parts = parts.parts.iter().map(|part| {
            let mut part = part.clone();
            part.sprite_sheet = Some(sprite_sheet.clone());
            part
        });

        parts.parts = enriched_parts.collect();

        parts
    }
}
