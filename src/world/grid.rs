use crate::world::cell::{Cell, MaterialId};

pub const WORLD_W: usize = 250;
pub const WORLD_H: usize = 250;

pub struct Grid {
    pub cells: Vec<Cell>,
    pub next: Vec<Cell>,
    pub width: usize,
    pub height: usize,
}

impl Grid {
    pub fn new() -> Self {
        let size = WORLD_W * WORLD_H;
        Self {
            cells: vec![Cell::empty(); size],
            next: vec![Cell::empty(); size],
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
    pub fn get_mut(&mut self, x: i32, y: i32) -> &mut Cell {
        let i = self.idx(x, y);
        &mut self.cells[i]
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, cell: Cell) {
        if self.in_bounds(x, y) {
            let i = self.idx(x, y);
            self.cells[i] = cell;
            self.next[i] = cell;
        }
    }

    #[inline]
    pub fn set_material(&mut self, x: i32, y: i32, mat: MaterialId) {
        if self.in_bounds(x, y) {
            let cell = Cell::new(mat);
            let i = self.idx(x, y);
            self.cells[i] = cell;
            self.next[i] = cell;
        }
    }

    pub fn clear(&mut self) {
        for c in &mut self.cells {
            *c = Cell::empty();
        }
        for c in &mut self.next {
            *c = Cell::empty();
        }
    }

    pub fn fill_rect(&mut self, x0: i32, y0: i32, w: i32, h: i32, mat: MaterialId) {
        for dy in 0..h {
            for dx in 0..w {
                self.set_material(x0 + dx, y0 + dy, mat);
            }
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

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.cells, &mut self.next);
    }

    pub fn reset_tick_flags(&mut self) {
        for c in &mut self.cells {
            c.updated_this_tick = false;
        }
    }

    pub fn dump_region(&self, x0: i32, y0: i32, w: usize, h: usize) -> String {
        let mut buf = String::with_capacity(w * h + h);
        for dy in 0..h {
            for dx in 0..w {
                let x = x0 + dx as i32;
                let y = y0 + dy as i32;
                let ch = if self.in_bounds(x, y) {
                    self.get(x, y).display_char()
                } else {
                    '?'
                };
                buf.push(ch);
            }
            buf.push('\n');
        }
        buf
    }
}
