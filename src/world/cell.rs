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
}

impl MaterialId {
    pub const ALL: [MaterialId; 14] = [
        MaterialId::Empty,
        MaterialId::Sand,
        MaterialId::Water,
        MaterialId::Stone,
        MaterialId::Lava,
        MaterialId::Wood,
        MaterialId::Flesh,
        MaterialId::Bone,
        MaterialId::Steam,
        MaterialId::Fire,
        MaterialId::Acid,
        MaterialId::Smoke,
        MaterialId::Grass,
        MaterialId::Dirt,
    ];

    pub fn from_u8(v: u8) -> Self {
        unsafe { std::mem::transmute(v) }
    }

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
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub material: MaterialId,
    pub temp: f32,
    pub updated_this_tick: bool,
    pub variant: u8,
}

impl Cell {
    pub fn empty() -> Self {
        Self {
            material: MaterialId::Empty,
            temp: 20.0,
            updated_this_tick: false,
            variant: 0,
        }
    }

    pub fn new(material: MaterialId) -> Self {
        let temp = match material {
            MaterialId::Lava => 1500.0,
            MaterialId::Fire => 800.0,
            MaterialId::Steam => 150.0,
            MaterialId::Smoke => 120.0,
            _ => 20.0,
        };
        Self {
            material,
            temp,
            updated_this_tick: false,
            variant: rand_u8(),
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
}

fn rand_u8() -> u8 {
    static mut COUNTER: u8 = 0;
    unsafe {
        COUNTER = COUNTER.wrapping_add(7);
        COUNTER
    }
}
