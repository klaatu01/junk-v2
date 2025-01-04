use std::{collections::HashSet, fmt::Display};

use bevy::math::I64Vec2;
use rand::{rngs::StdRng, Rng, SeedableRng};

const SYSTEM_ID_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const SYSTEM_ID_LENGTH: usize = 10;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SystemId(pub String);

impl SystemId {
    pub fn generate(rng: &mut StdRng) -> Self {
        let id: String = (0..SYSTEM_ID_LENGTH)
            .map(|_| {
                let idx = rng.gen_range(0..SYSTEM_ID_ALPHABET.len()) as u8;
                SYSTEM_ID_ALPHABET.chars().nth(idx as usize).unwrap()
            })
            .collect();
        Self(id)
    }
}

#[derive(Clone, Debug)]
pub struct SystemName(pub String);

impl SystemName {
    pub fn generate(rng: &mut StdRng) -> Self {
        // How many random letters and digits you want
        let letter_count = rng.gen_range(1..3);
        let digit_count = rng.gen_range(1..4);

        let letters: String = (0..letter_count)
            .map(|_| {
                let idx = rng.gen_range(0..26) as u8;
                (b'A' + idx) as char
            })
            .collect();

        let digits: String = (0..digit_count)
            .map(|_| rng.gen_range(0..10).to_string())
            .collect();

        let name = format!("{}-{}", letters, digits);
        Self(name)
    }
}

#[derive(Clone, Debug)]
pub struct System {
    pub id: SystemId,
    pub position: I64Vec2,
    pub seed: u64,
    pub properties: SystemProperties,
}

impl System {
    pub fn new(seed: u64, position: I64Vec2) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let id = SystemId::generate(&mut rng);
        let properties = SystemProperties::from_rngs(&mut rng);
        Self {
            id,
            position,
            seed,
            properties,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SystemComponents {
    Planet,
    Station,
    Asteroid,
    Wreckage,
    Pirates,
    Anomaly,
}

#[derive(Clone, Debug)]
pub struct SystemProperties {
    pub name: SystemName,
    pub r#type: HashSet<SystemComponents>,
    pub temperature: f64,
}

impl Display for SystemProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "System: {}\nTemperature: {:.2}\nDetected:{}",
            self.name.0,
            self.temperature,
            if self.r#type.is_empty() {
                " ?".to_string()
            } else {
                format!(
                    "\n{}",
                    self.r#type
                        .iter()
                        .map(|c| format!(" - {:?}", c))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
        )
    }
}

impl SystemProperties {
    pub fn from_rngs(rng: &mut StdRng) -> Self {
        let name = SystemName::generate(rng);

        let mut r#type = HashSet::new();
        let attempts = rng.gen_range(0..4);
        for _ in 0..attempts {
            let component = match rng.gen_range(0..6) {
                0 => SystemComponents::Planet,
                1 => SystemComponents::Station,
                2 => SystemComponents::Asteroid,
                3 => SystemComponents::Wreckage,
                4 => SystemComponents::Pirates,
                _ => SystemComponents::Anomaly,
            };
            r#type.insert(component);
        }

        Self {
            name,
            r#type,
            temperature: rng.gen_range(0.0..1.0),
        }
    }
}
