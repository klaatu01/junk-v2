use std::collections::HashSet;

use rand::{rngs::StdRng, Rng};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SystemId(pub String);

#[derive(Clone, Debug)]
pub struct SystemName(pub String);

impl SystemName {
    pub fn generate(rng: &mut StdRng) -> Self {
        // How many random letters and digits you want
        let letter_count = rng.gen_range(1..3);
        let digit_count = rng.gen_range(1..4);

        // Generate random letters (A-Z)
        let letters: String = (0..letter_count)
            .map(|_| {
                let idx = rng.gen_range(0..26) as u8;
                (b'A' + idx) as char // Convert to char in 'A'..'Z'
            })
            .collect();

        // Generate random digits (0-9)
        let digits: String = (0..digit_count)
            .map(|_| rng.gen_range(0..10).to_string())
            .collect();

        // Combine into a single name
        let name = format!("{}-{}", letters, digits);
        Self(name)
    }
}

pub struct System {
    pub id: SystemId,
    pub position: (isize, isize),
    pub properties: SystemProperties,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SystemComponents {
    Planet,
    Station,
    Asteroid,
    Wreckage,
    Anomaly,
}

pub struct SystemProperties {
    pub name: SystemName,
    pub r#type: HashSet<SystemComponents>,
    pub temperature: f64,
}

impl SystemProperties {
    pub fn from_rngs(rng: &mut StdRng) -> Self {
        let name = SystemName::generate(rng);

        let mut r#type = HashSet::new();
        let attempts = rng.gen_range(0..4);
        for _ in 0..attempts {
            let component = match rng.gen_range(0..4) {
                0 => SystemComponents::Planet,
                1 => SystemComponents::Station,
                2 => SystemComponents::Asteroid,
                3 => SystemComponents::Wreckage,
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
