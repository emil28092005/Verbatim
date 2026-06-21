use crate::world::cell::{Cell, MaterialId};
use crate::world::chunk::{world_to_chunk, CHUNK_SIZE};
use std::io;
use std::path::Path;

pub const WORLD_W: usize = 250;
pub const WORLD_H: usize = 250;

pub const MAX_WORLD_W: usize = 2048;
pub const MAX_WORLD_H: usize = 2048;

pub struct ChunkMeta {
    pub active: bool,
    pub modified: bool,
    pub was_modified: bool,
    pub dirty: Option<(i32, i32, i32, i32)>,
}

pub struct Grid {
    pub cells: Vec<Cell>,
    pub width: usize,
    pub height: usize,
    pub chunk_size: usize,
    pub chunks_x: usize,
    pub chunks_y: usize,
    pub chunks: Vec<ChunkMeta>,
}

impl Grid {
    pub fn new() -> Self {
        Self::with_size(WORLD_W, WORLD_H)
    }

    pub fn with_size(width: usize, height: usize) -> Self {
        let size = width * height;
        let chunk_size = CHUNK_SIZE;
        let chunks_x = (width + chunk_size - 1) / chunk_size;
        let chunks_y = (height + chunk_size - 1) / chunk_size;
        let mut chunks = Vec::with_capacity(chunks_x * chunks_y);
        for _ in 0..chunks_x * chunks_y {
            chunks.push(ChunkMeta {
                active: true,
                modified: false,
                was_modified: false,
                dirty: None,
            });
        }
        Self {
            cells: vec![Cell::empty(); size],
            width,
            height,
            chunk_size,
            chunks_x,
            chunks_y,
            chunks,
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
    pub fn chunk_index(&self, cx: i32, cy: i32) -> usize {
        (cy as usize) * self.chunks_x + (cx as usize)
    }

    #[inline]
    pub fn chunk_at(&self, x: i32, y: i32) -> (i32, i32, i32, i32) {
        world_to_chunk(x, y)
    }

    #[inline]
    pub fn is_chunk_active(&self, cx: i32, cy: i32) -> bool {
        if cx < 0 || cy < 0 || cx >= self.chunks_x as i32 || cy >= self.chunks_y as i32 {
            return false;
        }
        self.chunks[self.chunk_index(cx, cy)].active
    }

    pub fn set_chunk_active(&mut self, cx: i32, cy: i32, active: bool) {
        if cx < 0 || cy < 0 || cx >= self.chunks_x as i32 || cy >= self.chunks_y as i32 {
            return;
        }
        let idx = self.chunk_index(cx, cy);
        self.chunks[idx].active = active;
    }

    pub fn activate_around(&mut self, x: i32, y: i32, radius: i32) {
        let (cx, cy, _, _) = self.chunk_at(x, y);
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                self.set_chunk_active(cx + dx, cy + dy, true);
            }
        }
    }

    pub fn deactivate_all(&mut self) {
        for c in &mut self.chunks {
            c.active = false;
        }
    }

    pub fn swap_modified_flags(&mut self) {
        for c in &mut self.chunks {
            c.was_modified = c.modified;
            c.modified = false;
        }
    }

    pub fn any_modified(&self) -> bool {
        self.chunks.iter().any(|c| c.modified)
    }

    pub fn any_was_modified(&self) -> bool {
        self.chunks.iter().any(|c| c.was_modified)
    }

    pub fn active_chunks(&self) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for cy in 0..self.chunks_y {
            for cx in 0..self.chunks_x {
                if self.chunks[cy * self.chunks_x + cx].active {
                    out.push((cx, cy));
                }
            }
        }
        out
    }

    pub fn chunk_bounds(&self, cx: usize, cy: usize) -> (i32, i32, i32, i32) {
        let x0 = (cx * self.chunk_size) as i32;
        let y0 = (cy * self.chunk_size) as i32;
        let x1 = (x0 + self.chunk_size as i32).min(self.width as i32);
        let y1 = (y0 + self.chunk_size as i32).min(self.height as i32);
        (x0, y0, x1, y1)
    }

    #[inline]
    pub fn cell_active(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return false;
        }
        let (cx, cy, _, _) = self.chunk_at(x, y);
        self.is_chunk_active(cx, cy)
    }

    pub fn chunk_cells(&self, cx: usize, cy: usize) -> Vec<(i32, i32, Cell)> {
        let (x0, y0, x1, y1) = self.chunk_bounds(cx, cy);
        let mut out = Vec::with_capacity((x1 - x0) as usize * (y1 - y0) as usize);
        for y in y0..y1 {
            for x in x0..x1 {
                out.push((x, y, self.get(x, y)));
            }
        }
        out
    }

    pub fn load_chunk_cells(&mut self, cx: usize, cy: usize, cells: &[Cell]) {
        let (x0, y0, x1, y1) = self.chunk_bounds(cx, cy);
        let w = (x1 - x0) as usize;
        for y in y0..y1 {
            for x in x0..x1 {
                let i = ((y - y0) as usize) * w + (x - x0) as usize;
                if let Some(cell) = cells.get(i) {
                    self.set(x, y, *cell);
                }
            }
        }
        self.set_chunk_active(cx as i32, cy as i32, true);
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
            let (cx, cy, _, _) = self.chunk_at(x, y);
            let idx = self.chunk_index(cx, cy);
            if let Some(c) = self.chunks.get_mut(idx) {
                c.modified = true;
            }
            self.mark_dirty(x, y);
        }
    }

    #[inline]
    pub fn set_material(&mut self, x: i32, y: i32, mat: MaterialId) {
        if self.in_bounds(x, y) {
            let i = (y as usize) * self.width + (x as usize);
            self.cells[i] = Cell::new(mat);
            let (cx, cy, _, _) = self.chunk_at(x, y);
            let idx = self.chunk_index(cx, cy);
            if let Some(c) = self.chunks.get_mut(idx) {
                c.modified = true;
            }
            self.mark_dirty(x, y);
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

    #[inline]
    pub fn mark_dirty(&mut self, x: i32, y: i32) {
        if !self.in_bounds(x, y) {
            return;
        }
        let (cx, cy, _, _) = self.chunk_at(x, y);
        self.expand_chunk_dirty(cx, cy, x, y);
        let cs = self.chunk_size as i32;
        let lx = x - cx * cs;
        let ly = y - cy * cs;
        if lx <= 1 {
            self.expand_chunk_dirty(cx - 1, cy, x - 1, y);
        }
        if lx >= cs - 2 {
            self.expand_chunk_dirty(cx + 1, cy, x + 1, y);
        }
        if ly <= 1 {
            self.expand_chunk_dirty(cx, cy - 1, x, y - 1);
        }
        if ly >= cs - 2 {
            self.expand_chunk_dirty(cx, cy + 1, x, y + 1);
        }
    }

    #[inline]
    fn expand_chunk_dirty(&mut self, cx: i32, cy: i32, x: i32, y: i32) {
        if cx < 0 || cy < 0 || cx >= self.chunks_x as i32 || cy >= self.chunks_y as i32 {
            return;
        }
        if !self.in_bounds(x, y) {
            return;
        }
        let idx = self.chunk_index(cx, cy);
        let min_x = (x - 1).max(0);
        let min_y = (y - 1).max(0);
        let max_x = (x + 1).min(self.width as i32 - 1);
        let max_y = (y + 1).min(self.height as i32 - 1);
        let chunk = &mut self.chunks[idx];
        match chunk.dirty {
            None => chunk.dirty = Some((min_x, min_y, max_x, max_y)),
            Some((dx0, dy0, dx1, dy1)) => {
                chunk.dirty = Some((
                    dx0.min(min_x),
                    dy0.min(min_y),
                    dx1.max(max_x),
                    dy1.max(max_y),
                ));
            }
        }
    }

    #[inline]
    pub fn cells_swap(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        if !self.in_bounds(x1, y1) || !self.in_bounds(x2, y2) {
            return;
        }
        let i1 = self.idx(x1, y1);
        let i2 = self.idx(x2, y2);
        let tmp = self.cells[i1];
        self.cells[i1] = self.cells[i2];
        self.cells[i2] = tmp;
        self.cells[i2].updated_this_tick = true;
        self.mark_dirty(x1, y1);
        self.mark_dirty(x2, y2);
    }

    #[inline]
    pub fn set_cell_index(&mut self, i: usize, cell: Cell) {
        self.cells[i] = cell;
        let x = (i % self.width) as i32;
        let y = (i / self.width) as i32;
        self.mark_dirty(x, y);
    }

    pub fn reset_tick_flags(&mut self) {
        for cy in 0..self.chunks_y {
            for cx in 0..self.chunks_x {
                if !self.chunks[cy * self.chunks_x + cx].active {
                    continue;
                }
                let (x0, y0, x1, y1) = self.chunk_bounds(cx, cy);
                for y in y0..y1 {
                    let row = y as usize * self.width;
                    for x in x0..x1 {
                        self.cells[row + x as usize].updated_this_tick = false;
                    }
                }
            }
        }
    }

    pub fn save_chunk(&self, path: &str, cx: i32, cy: i32) -> io::Result<()> {
        if cx < 0 || cy < 0 || cx >= self.chunks_x as i32 || cy >= self.chunks_y as i32 {
            return Err(io::Error::other("chunk out of bounds"));
        }
        let (x0, y0, x1, y1) = self.chunk_bounds(cx as usize, cy as usize);
        let w = (x1 - x0) as usize;
        let h = (y1 - y0) as usize;
        let mut bytes = Vec::with_capacity(w * h * 12);
        for y in y0..y1 {
            for x in x0..x1 {
                bytes.extend_from_slice(&self.get(x, y).to_bytes());
            }
        }
        let dir = Path::new(path);
        if let Some(parent) = dir.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, bytes)
    }

    pub fn load_chunk(&mut self, path: &str, cx: i32, cy: i32) -> io::Result<()> {
        if cx < 0 || cy < 0 || cx >= self.chunks_x as i32 || cy >= self.chunks_y as i32 {
            return Err(io::Error::other("chunk out of bounds"));
        }
        let data = std::fs::read(path)?;
        let (x0, y0, x1, y1) = self.chunk_bounds(cx as usize, cy as usize);
        let w = (x1 - x0) as usize;
        let h = (y1 - y0) as usize;
        let expected = w * h * 12;
        if data.len() != expected {
            return Err(io::Error::other("chunk file size mismatch"));
        }
        let mut i = 0usize;
        for y in y0..y1 {
            for x in x0..x1 {
                let cell = Cell::from_bytes(&data[i * 12..(i + 1) * 12]);
                self.set(x, y, cell);
                i += 1;
            }
        }
        self.set_chunk_active(cx, cy, true);
        Ok(())
    }
}
