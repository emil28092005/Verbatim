use crate::world::cell::{Cell, MaterialId};

pub const CHUNK_SIZE: usize = 64;

pub struct Chunk {
    pub cells: Vec<Cell>,
    pub active: bool,
    pub modified: bool,
}

impl Chunk {
    pub fn new() -> Self {
        let size = CHUNK_SIZE * CHUNK_SIZE;
        Self {
            cells: vec![Cell::empty(); size],
            active: false,
            modified: false,
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
            self.modified = true;
        }
    }

    pub fn reset_tick_flags(&mut self) {
        for c in &mut self.cells {
            c.updated_this_tick = false;
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
