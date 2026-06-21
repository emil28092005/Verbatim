use crate::world::cell::{Cell, MaterialId};

pub const CHUNK_SIZE: usize = 64;

pub struct Chunk {
    pub cells: Vec<Cell>,
    pub active: bool,
    pub modified: bool,
    pub was_modified: bool,
    pub generated: bool,
    pub dirty: Option<(i32, i32, i32, i32)>,
}

impl Chunk {
    pub fn new() -> Self {
        let size = CHUNK_SIZE * CHUNK_SIZE;
        Self {
            cells: vec![Cell::empty(); size],
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
            self.modified = true;
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
