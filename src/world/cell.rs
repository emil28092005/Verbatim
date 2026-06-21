use crate::world::material::MaterialRegistry;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum MaterialId {
    Empty = 0,
    Sand = 1,
    Water = 2,
    Stone = 3,
    Lava = 4,
    Wood = 5,
    Flesh = 6,
    Bone = 7,
    Steam = 8,
    Fire = 9,
    Acid = 10,
    Smoke = 11,
    Grass = 12,
    Dirt = 13,
    Stairs = 14,
}

impl MaterialId {
    pub fn display_char(self) -> char {
        match self {
            MaterialId::Empty => ' ',
            MaterialId::Sand => '.',
            MaterialId::Water => '~',
            MaterialId::Stone => '#',
            MaterialId::Lava => '#',
            MaterialId::Wood => 'T',
            MaterialId::Flesh => '%',
            MaterialId::Bone => '`',
            MaterialId::Steam => '~',
            MaterialId::Fire => '^',
            MaterialId::Acid => '~',
            MaterialId::Smoke => '*',
            MaterialId::Grass => '"',
            MaterialId::Dirt => ':',
            MaterialId::Stairs => '>',
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub material: MaterialId,
    pub updated_this_tick: bool,
    pub variant: u8,
    pub fg: [u8; 3],
    pub bg: [u8; 3],
}

pub fn default_temp(material: MaterialId) -> f32 {
    match material {
        MaterialId::Lava => 1500.0,
        MaterialId::Fire => 800.0,
        MaterialId::Steam => 150.0,
        MaterialId::Smoke => 120.0,
        _ => 20.0,
    }
}

impl Cell {
    pub fn empty() -> Self {
        Self {
            material: MaterialId::Empty,
            updated_this_tick: false,
            variant: 0,
            fg: [15, 15, 20],
            bg: [10, 10, 15],
        }
    }

    pub fn new(material: MaterialId) -> Self {
        let reg = MaterialRegistry::instance();
        let mat = reg.get(material);
        Self {
            material,
            updated_this_tick: false,
            variant: rand_u8(),
            fg: [mat.color_fg.0, mat.color_fg.1, mat.color_fg.2],
            bg: [mat.color_bg.0, mat.color_bg.1, mat.color_bg.2],
        }
    }

    pub fn is_empty(self) -> bool {
        self.material == MaterialId::Empty
    }

    pub fn is_solid(self) -> bool {
        let reg = MaterialRegistry::instance();
        reg.get(self.material).solid
    }

    pub fn is_liquid(self) -> bool {
        let reg = MaterialRegistry::instance();
        reg.get(self.material).liquid
    }

    pub fn is_gas(self) -> bool {
        let reg = MaterialRegistry::instance();
        reg.get(self.material).gas
    }

    pub fn is_static(self) -> bool {
        let reg = MaterialRegistry::instance();
        reg.get(self.material).static_
    }

    pub fn density(self) -> f32 {
        let reg = MaterialRegistry::instance();
        reg.get(self.material).density
    }

    pub fn display_char(self) -> char {
        self.material.display_char()
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        let mut out = [0u8; 8];
        out[0] = self.material as u8;
        out[1] = self.variant;
        out[2..5].copy_from_slice(&self.fg);
        out[5..8].copy_from_slice(&self.bg);
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let material = if bytes.is_empty() {
            MaterialId::Empty
        } else {
            match bytes[0] {
                0 => MaterialId::Empty,
                1 => MaterialId::Sand,
                2 => MaterialId::Water,
                3 => MaterialId::Stone,
                4 => MaterialId::Lava,
                5 => MaterialId::Wood,
                6 => MaterialId::Flesh,
                7 => MaterialId::Bone,
                8 => MaterialId::Steam,
                9 => MaterialId::Fire,
                10 => MaterialId::Acid,
                11 => MaterialId::Smoke,
                12 => MaterialId::Grass,
                13 => MaterialId::Dirt,
                14 => MaterialId::Stairs,
                _ => MaterialId::Stone,
            }
        };
        let variant = bytes.get(1).copied().unwrap_or(0);
        let fg = if bytes.len() >= 5 {
            [bytes[2], bytes[3], bytes[4]]
        } else {
            [15, 15, 20]
        };
        let bg = if bytes.len() >= 8 {
            [bytes[5], bytes[6], bytes[7]]
        } else {
            [10, 10, 15]
        };
        Self {
            material,
            updated_this_tick: false,
            variant,
            fg,
            bg,
        }
    }
}

use std::sync::atomic::{AtomicU8, Ordering};

fn rand_u8() -> u8 {
    static COUNTER: AtomicU8 = AtomicU8::new(0);
    COUNTER.fetch_add(7, Ordering::Relaxed)
}
