use crate::world::cell::{Cell, MaterialId};

pub const CHUNK_SIZE: usize = 64;

pub struct Chunk {
    pub cells: Vec<Cell>,
    pub temps: Vec<f32>,
    pub pressure: Vec<u8>,
    pub gas_type: Vec<u8>,
    pub gas_density: Vec<u8>,
    pub light: Vec<[u8; 3]>,
    pub active: bool,
    pub modified: bool,
    pub was_modified: bool,
    pub generated: bool,
    pub dirty: Option<(i32, i32, i32, i32)>,
}

const CHUNK_AREA: usize = CHUNK_SIZE * CHUNK_SIZE;
const ATMOSPHERIC_PRESSURE: u8 = 128;

impl Chunk {
    pub fn new() -> Self {
        Self {
            cells: vec![Cell::empty(); CHUNK_AREA],
            temps: vec![20.0; CHUNK_AREA],
            pressure: vec![ATMOSPHERIC_PRESSURE; CHUNK_AREA],
            gas_type: vec![0; CHUNK_AREA],
            gas_density: vec![0; CHUNK_AREA],
            light: vec![[0, 0, 0]; CHUNK_AREA],
            active: false,
            modified: false,
            was_modified: false,
            generated: false,
            dirty: None,
        }
    }

    pub fn swap_modified_flags(&mut self) {
        self.was_modified = self.modified;
        self.modified = false;
    }

    pub fn reset_tick_flags(&mut self) {
        for c in &mut self.cells {
            c.updated_this_tick = false;
        }
    }

    #[inline]
    pub fn in_bounds(x: i32, y: i32) -> bool {
        x >= 0 && x < CHUNK_SIZE as i32 && y >= 0 && y < CHUNK_SIZE as i32
    }

    #[inline]
    fn idx(x: i32, y: i32) -> usize {
        (y as usize) * CHUNK_SIZE + (x as usize)
    }

    pub fn get(&self, x: i32, y: i32) -> Cell {
        if !Self::in_bounds(x, y) {
            return Cell::new(MaterialId::Stone);
        }
        self.cells[Self::idx(x, y)]
    }

    pub fn set(&mut self, x: i32, y: i32, cell: Cell) {
        if Self::in_bounds(x, y) {
            self.cells[Self::idx(x, y)] = cell;
            self.modified = true;
        }
    }

    pub fn set_material(&mut self, x: i32, y: i32, mat: MaterialId) {
        if Self::in_bounds(x, y) {
            self.cells[Self::idx(x, y)] = Cell::new(mat);
            self.temps[Self::idx(x, y)] = crate::world::cell::default_temp(mat);
            self.modified = true;
        }
    }

    #[inline]
    pub fn get_temp(&self, x: i32, y: i32) -> f32 {
        if !Self::in_bounds(x, y) {
            return 20.0;
        }
        self.temps[Self::idx(x, y)]
    }

    #[inline]
    pub fn set_temp(&mut self, x: i32, y: i32, t: f32) {
        if Self::in_bounds(x, y) {
            self.temps[Self::idx(x, y)] = t;
        }
    }

    #[inline]
    pub fn get_pressure(&self, x: i32, y: i32) -> u8 {
        if !Self::in_bounds(x, y) {
            return 128;
        }
        self.pressure[Self::idx(x, y)]
    }

    #[inline]
    pub fn set_pressure(&mut self, x: i32, y: i32, p: u8) {
        if Self::in_bounds(x, y) {
            self.pressure[Self::idx(x, y)] = p;
        }
    }

    #[inline]
    pub fn get_gas(&self, x: i32, y: i32) -> (u8, u8) {
        if !Self::in_bounds(x, y) {
            return (0, 0);
        }
        let i = Self::idx(x, y);
        (self.gas_type[i], self.gas_density[i])
    }

    #[inline]
    pub fn set_gas(&mut self, x: i32, y: i32, gas_type: u8, density: u8) {
        if Self::in_bounds(x, y) {
            let i = Self::idx(x, y);
            self.gas_type[i] = gas_type;
            self.gas_density[i] = density;
        }
    }

    #[inline]
    pub fn get_light(&self, x: i32, y: i32) -> [u8; 3] {
        if !Self::in_bounds(x, y) {
            return [0, 0, 0];
        }
        self.light[Self::idx(x, y)]
    }

    #[inline]
    pub fn set_light(&mut self, x: i32, y: i32, rgb: [u8; 3]) {
        if Self::in_bounds(x, y) {
            self.light[Self::idx(x, y)] = rgb;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cells.iter().all(|c| c.is_empty())
    }

    #[inline]
    pub fn mark_dirty(&mut self, x: i32, y: i32) {
        if !Self::in_bounds(x, y) {
            return;
        }
        let min_x = (x - 1).max(0);
        let min_y = (y - 1).max(0);
        let max_x = (x + 1).min(CHUNK_SIZE as i32 - 1);
        let max_y = (y + 1).min(CHUNK_SIZE as i32 - 1);
        match self.dirty {
            None => self.dirty = Some((min_x, min_y, max_x, max_y)),
            Some((dx0, dy0, dx1, dy1)) => {
                self.dirty = Some((
                    dx0.min(min_x),
                    dy0.min(min_y),
                    dx1.max(max_x),
                    dy1.max(max_y),
                ));
            }
        }
    }
}

pub fn world_to_chunk(world_x: i32, world_y: i32) -> (i32, i32, i32, i32) {
    let cx = world_x.div_euclid(CHUNK_SIZE as i32);
    let cy = world_y.div_euclid(CHUNK_SIZE as i32);
    let lx = world_x.rem_euclid(CHUNK_SIZE as i32);
    let ly = world_y.rem_euclid(CHUNK_SIZE as i32);
    (cx, cy, lx, ly)
}

#[derive(Clone)]
pub struct ChunkCell {
    pub x: i32,
    pub y: i32,
    pub cell: Cell,
}

pub fn chunk_cells() -> impl Iterator<Item = (i32, i32)> {
    (0..CHUNK_SIZE as i32).flat_map(|y| (0..CHUNK_SIZE as i32).map(move |x| (x, y)))
}
