use crate::world::cell::{Cell, MaterialId};

pub const WORLD_W: usize = 250;
pub const WORLD_H: usize = 250;

pub struct Grid {
    pub cells: Vec<Cell>,
    pub width: usize,
    pub height: usize,
}

impl Grid {
    pub fn new() -> Self {
        let size = WORLD_W * WORLD_H;
        Self {
            cells: vec![Cell::empty(); size],
            width: WORLD_W,
            height: WORLD_H,
        }
    }

    #[inline]
    pub fn idx(&self, x: i32, y: i32) -> usize {
        (y as usize) * self.width + (x as usize)
    }

    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32) -> Cell {
        if !self.in_bounds(x, y) {
            return Cell::new(MaterialId::Stone);
        }
        self.cells[self.idx(x, y)]
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, cell: Cell) {
        if self.in_bounds(x, y) {
            let i = (y as usize) * self.width + (x as usize);
            self.cells[i] = cell;
        }
    }

    #[inline]
    pub fn set_material(&mut self, x: i32, y: i32, mat: MaterialId) {
        if self.in_bounds(x, y) {
            let i = (y as usize) * self.width + (x as usize);
            self.cells[i] = Cell::new(mat);
        }
    }

    pub fn fill_border(&mut self, mat: MaterialId) {
        for x in 0..self.width {
            self.set_material(x as i32, 0, mat);
            self.set_material(x as i32, (self.height - 1) as i32, mat);
        }
        for y in 0..self.height {
            self.set_material(0, y as i32, mat);
            self.set_material((self.width - 1) as i32, y as i32, mat);
        }
    }

    pub fn reset_tick_flags(&mut self) {
        for c in &mut self.cells {
            c.updated_this_tick = false;
        }
    }
}
