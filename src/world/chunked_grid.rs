use crate::world::cell::{default_temp, Cell, MaterialId};
use crate::world::chunk::{Chunk, CHUNK_SIZE};
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

pub struct ChunkedGrid {
    pub chunk_size: usize,
    pub chunks: HashMap<(i64, i64), Chunk>,
    pub chunks_vec: Vec<Chunk>,
    pub bounds: Option<(i64, i64, i64, i64)>,
    pub seed: u64,
    pub cache_dir: Option<String>,
    pub width: usize,
    pub height: usize,
    pub chunks_x: usize,
    pub chunks_y: usize,
}

impl ChunkedGrid {
    pub fn with_size(width: usize, height: usize) -> Self {
        let chunk_size = CHUNK_SIZE;
        let chunks_x = (width + chunk_size - 1) / chunk_size;
        let chunks_y = (height + chunk_size - 1) / chunk_size;
        let mut chunks_vec = Vec::with_capacity(chunks_x * chunks_y);
        for _ in 0..chunks_x * chunks_y {
            let mut chunk = Chunk::new();
            chunk.active = true;
            chunks_vec.push(chunk);
        }
        Self {
            chunk_size,
            chunks: HashMap::new(),
            chunks_vec,
            bounds: Some((0, 0, width as i64, height as i64)),
            seed: 0,
            cache_dir: None,
            width,
            height,
            chunks_x,
            chunks_y,
        }
    }

    pub fn infinite(seed: u64, cache_dir: Option<String>) -> Self {
        Self {
            chunk_size: CHUNK_SIZE,
            chunks: HashMap::new(),
            chunks_vec: Vec::new(),
            bounds: None,
            seed,
            cache_dir,
            width: i64::MAX as usize,
            height: i64::MAX as usize,
            chunks_x: 0,
            chunks_y: 0,
        }
    }

    #[inline]
    fn chunk_index(&self, cx: i32, cy: i32) -> Option<usize> {
        if cx < 0 || cy < 0 || cx >= self.chunks_x as i32 || cy >= self.chunks_y as i32 {
            return None;
        }
        Some((cy as usize) * self.chunks_x + (cx as usize))
    }

    #[inline]
    fn is_bounded(&self) -> bool {
        self.bounds.is_some()
    }

    #[inline]
    pub fn chunk_at(&self, x: i32, y: i32) -> (i32, i32, i32, i32) {
        let cs = self.chunk_size as i32;
        let cx = x.div_euclid(cs);
        let cy = y.div_euclid(cs);
        let lx = x.rem_euclid(cs);
        let ly = y.rem_euclid(cs);
        (cx, cy, lx, ly)
    }

    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        match self.bounds {
            Some((x0, y0, x1, y1)) => {
                let wx = x as i64;
                let wy = y as i64;
                wx >= x0 && wx < x1 && wy >= y0 && wy < y1
            }
            None => true,
        }
    }

    #[inline]
    pub fn get_chunk(&self, cx: i32, cy: i32) -> Option<&Chunk> {
        if self.is_bounded() {
            self.chunk_index(cx, cy)
                .and_then(|idx| self.chunks_vec.get(idx))
        } else {
            self.chunks.get(&(cx as i64, cy as i64))
        }
    }

    #[inline]
    pub fn get_chunk_mut(&mut self, cx: i32, cy: i32) -> Option<&mut Chunk> {
        if self.is_bounded() {
            self.chunk_index(cx, cy)
                .and_then(|idx| self.chunks_vec.get_mut(idx))
        } else {
            self.chunks.get_mut(&(cx as i64, cy as i64))
        }
    }

    pub fn ensure_chunk(&mut self, cx: i32, cy: i32) -> Option<&mut Chunk> {
        let origin_x = cx * self.chunk_size as i32;
        let origin_y = cy * self.chunk_size as i32;
        if !self.in_bounds(origin_x, origin_y) {
            return None;
        }
        if self.is_bounded() {
            return self.get_chunk_mut(cx, cy);
        }
        let key = (cx as i64, cy as i64);
        if !self.chunks.contains_key(&key) {
            let chunk = Chunk::new();
            if let Some(ref dir) = self.cache_dir {
                let path = chunk_path(dir, self.seed, cx, cy);
                if path.exists() {
                    if let Err(e) = self.load_chunk_from_path(&path, cx, cy) {
                        eprintln!("Chunk load failed {} {}: {}", cx, cy, e);
                    } else {
                        return self.chunks.get_mut(&key);
                    }
                }
            }
            self.chunks.insert(key, chunk);
        }
        self.chunks.get_mut(&key)
    }

    pub fn get_or_create_chunk(&mut self, cx: i32, cy: i32) -> &mut Chunk {
        if self.is_bounded() {
            let idx = self.chunk_index(cx, cy).unwrap();
            return &mut self.chunks_vec[idx];
        }
        let key = (cx as i64, cy as i64);
        if !self.chunks.contains_key(&key) {
            self.chunks.insert(key, Chunk::new());
        }
        self.chunks.get_mut(&key).unwrap()
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32) -> Cell {
        if !self.in_bounds(x, y) {
            return Cell::new(MaterialId::Stone);
        }
        let (cx, cy, lx, ly) = self.chunk_at(x, y);
        if self.is_bounded() {
            if let Some(idx) = self.chunk_index(cx, cy) {
                if let Some(chunk) = self.chunks_vec.get(idx) {
                    return chunk.get(lx, ly);
                }
            }
        } else if let Some(chunk) = self.chunks.get(&(cx as i64, cy as i64)) {
            return chunk.get(lx, ly);
        }
        Cell::new(MaterialId::Stone)
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, cell: Cell) {
        if !self.in_bounds(x, y) {
            return;
        }
        let (cx, cy, lx, ly) = self.chunk_at(x, y);
        if self.is_bounded() {
            if let Some(idx) = self.chunk_index(cx, cy) {
                if let Some(chunk) = self.chunks_vec.get_mut(idx) {
                    chunk.set(lx, ly, cell);
                    chunk.mark_dirty(lx, ly);
                }
            }
        } else {
            let chunk = self.get_or_create_chunk(cx, cy);
            chunk.set(lx, ly, cell);
            chunk.mark_dirty(lx, ly);
        }
    }

    #[inline]
    pub fn set_material(&mut self, x: i32, y: i32, mat: MaterialId) {
        if !self.in_bounds(x, y) {
            return;
        }
        let (cx, cy, lx, ly) = self.chunk_at(x, y);
        let t = default_temp(mat);
        if self.is_bounded() {
            if let Some(idx) = self.chunk_index(cx, cy) {
                if let Some(chunk) = self.chunks_vec.get_mut(idx) {
                    chunk.set_material(lx, ly, mat);
                    chunk.set_temp(lx, ly, t);
                    chunk.mark_dirty(lx, ly);
                }
            }
        } else {
            let chunk = self.get_or_create_chunk(cx, cy);
            chunk.set_material(lx, ly, mat);
            chunk.set_temp(lx, ly, t);
            chunk.mark_dirty(lx, ly);
        }
    }

    #[inline]
    pub fn get_temp(&self, x: i32, y: i32) -> f32 {
        if !self.in_bounds(x, y) {
            return 20.0;
        }
        let (cx, cy, lx, ly) = self.chunk_at(x, y);
        if self.is_bounded() {
            if let Some(idx) = self.chunk_index(cx, cy) {
                if let Some(chunk) = self.chunks_vec.get(idx) {
                    return chunk.get_temp(lx, ly);
                }
            }
        } else if let Some(chunk) = self.chunks.get(&(cx as i64, cy as i64)) {
            return chunk.get_temp(lx, ly);
        }
        20.0
    }

    #[inline]
    pub fn set_temp(&mut self, x: i32, y: i32, t: f32) {
        if !self.in_bounds(x, y) {
            return;
        }
        let (cx, cy, lx, ly) = self.chunk_at(x, y);
        if self.is_bounded() {
            if let Some(idx) = self.chunk_index(cx, cy) {
                if let Some(chunk) = self.chunks_vec.get_mut(idx) {
                    chunk.set_temp(lx, ly, t);
                }
            }
        } else {
            let chunk = self.get_or_create_chunk(cx, cy);
            chunk.set_temp(lx, ly, t);
        }
    }

    #[inline]
    pub fn get_pressure(&self, x: i32, y: i32) -> u8 {
        if !self.in_bounds(x, y) {
            return 128;
        }
        let (cx, cy, lx, ly) = self.chunk_at(x, y);
        if self.is_bounded() {
            if let Some(idx) = self.chunk_index(cx, cy) {
                if let Some(chunk) = self.chunks_vec.get(idx) {
                    return chunk.get_pressure(lx, ly);
                }
            }
        } else if let Some(chunk) = self.chunks.get(&(cx as i64, cy as i64)) {
            return chunk.get_pressure(lx, ly);
        }
        128
    }

    #[inline]
    pub fn set_pressure(&mut self, x: i32, y: i32, p: u8) {
        if !self.in_bounds(x, y) {
            return;
        }
        let (cx, cy, lx, ly) = self.chunk_at(x, y);
        if self.is_bounded() {
            if let Some(idx) = self.chunk_index(cx, cy) {
                if let Some(chunk) = self.chunks_vec.get_mut(idx) {
                    chunk.set_pressure(lx, ly, p);
                    chunk.mark_dirty(lx, ly);
                }
            }
        } else {
            let chunk = self.get_or_create_chunk(cx, cy);
            chunk.set_pressure(lx, ly, p);
            chunk.mark_dirty(lx, ly);
        }
    }

    #[inline]
    pub fn get_gas(&self, x: i32, y: i32) -> (u8, u8) {
        if !self.in_bounds(x, y) {
            return (0, 0);
        }
        let (cx, cy, lx, ly) = self.chunk_at(x, y);
        if self.is_bounded() {
            if let Some(idx) = self.chunk_index(cx, cy) {
                if let Some(chunk) = self.chunks_vec.get(idx) {
                    return chunk.get_gas(lx, ly);
                }
            }
        } else if let Some(chunk) = self.chunks.get(&(cx as i64, cy as i64)) {
            return chunk.get_gas(lx, ly);
        }
        (0, 0)
    }

    #[inline]
    pub fn set_gas(&mut self, x: i32, y: i32, gas_type: u8, density: u8) {
        if !self.in_bounds(x, y) {
            return;
        }
        let (cx, cy, lx, ly) = self.chunk_at(x, y);
        if self.is_bounded() {
            if let Some(idx) = self.chunk_index(cx, cy) {
                if let Some(chunk) = self.chunks_vec.get_mut(idx) {
                    chunk.set_gas(lx, ly, gas_type, density);
                    chunk.mark_dirty(lx, ly);
                }
            }
        } else {
            let chunk = self.get_or_create_chunk(cx, cy);
            chunk.set_gas(lx, ly, gas_type, density);
            chunk.mark_dirty(lx, ly);
        }
    }

    #[inline]
    pub fn get_light(&self, x: i32, y: i32) -> [u8; 3] {
        if !self.in_bounds(x, y) {
            return [0, 0, 0];
        }
        let (cx, cy, lx, ly) = self.chunk_at(x, y);
        if self.is_bounded() {
            if let Some(idx) = self.chunk_index(cx, cy) {
                if let Some(chunk) = self.chunks_vec.get(idx) {
                    return chunk.get_light(lx, ly);
                }
            }
        } else if let Some(chunk) = self.chunks.get(&(cx as i64, cy as i64)) {
            return chunk.get_light(lx, ly);
        }
        [0, 0, 0]
    }

    #[inline]
    pub fn set_light(&mut self, x: i32, y: i32, rgb: [u8; 3]) {
        if !self.in_bounds(x, y) {
            return;
        }
        let (cx, cy, lx, ly) = self.chunk_at(x, y);
        if self.is_bounded() {
            if let Some(idx) = self.chunk_index(cx, cy) {
                if let Some(chunk) = self.chunks_vec.get_mut(idx) {
                    chunk.set_light(lx, ly, rgb);
                }
            }
        } else {
            let chunk = self.get_or_create_chunk(cx, cy);
            chunk.set_light(lx, ly, rgb);
        }
    }

    #[inline]
    pub fn mark_dirty(&mut self, x: i32, y: i32) {
        if !self.in_bounds(x, y) {
            return;
        }
        let (cx, cy, lx, ly) = self.chunk_at(x, y);
        let cs = self.chunk_size as i32;
        if self.is_bounded() {
            if let Some(idx) = self.chunk_index(cx, cy) {
                if let Some(chunk) = self.chunks_vec.get_mut(idx) {
                    chunk.mark_dirty(lx, ly);
                }
            }
            if lx <= 1 {
                if let Some(idx) = self.chunk_index(cx - 1, cy) {
                    if let Some(chunk) = self.chunks_vec.get_mut(idx) {
                        chunk.mark_dirty(cs - 1, ly);
                    }
                }
            }
            if lx >= cs - 2 {
                if let Some(idx) = self.chunk_index(cx + 1, cy) {
                    if let Some(chunk) = self.chunks_vec.get_mut(idx) {
                        chunk.mark_dirty(0, ly);
                    }
                }
            }
            if ly <= 1 {
                if let Some(idx) = self.chunk_index(cx, cy - 1) {
                    if let Some(chunk) = self.chunks_vec.get_mut(idx) {
                        chunk.mark_dirty(lx, cs - 1);
                    }
                }
            }
            if ly >= cs - 2 {
                if let Some(idx) = self.chunk_index(cx, cy + 1) {
                    if let Some(chunk) = self.chunks_vec.get_mut(idx) {
                        chunk.mark_dirty(lx, 0);
                    }
                }
            }
        } else {
            let cx64 = cx as i64;
            let cy64 = cy as i64;
            if let Some(chunk) = self.chunks.get_mut(&(cx64, cy64)) {
                chunk.mark_dirty(lx, ly);
            }
            if lx <= 1 {
                if let Some(chunk) = self.chunks.get_mut(&(cx64 - 1, cy64)) {
                    chunk.mark_dirty(cs - 1, ly);
                }
            }
            if lx >= cs - 2 {
                if let Some(chunk) = self.chunks.get_mut(&(cx64 + 1, cy64)) {
                    chunk.mark_dirty(0, ly);
                }
            }
            if ly <= 1 {
                if let Some(chunk) = self.chunks.get_mut(&(cx64, cy64 - 1)) {
                    chunk.mark_dirty(lx, cs - 1);
                }
            }
            if ly >= cs - 2 {
                if let Some(chunk) = self.chunks.get_mut(&(cx64, cy64 + 1)) {
                    chunk.mark_dirty(lx, 0);
                }
            }
        }
    }

    #[inline]
    pub fn cells_swap(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        if !self.in_bounds(x1, y1) || !self.in_bounds(x2, y2) {
            return;
        }
        let (cx1, cy1, lx1, ly1) = self.chunk_at(x1, y1);
        let (cx2, cy2, lx2, ly2) = self.chunk_at(x2, y2);
        if self.is_bounded() {
            let idx1 = self.chunk_index(cx1, cy1);
            let idx2 = self.chunk_index(cx2, cy2);
            match (idx1, idx2) {
                (Some(i1), Some(i2)) if i1 == i2 => {
                    if let Some(chunk) = self.chunks_vec.get_mut(i1) {
                        let ci1 = (ly1 as usize) * self.chunk_size + (lx1 as usize);
                        let ci2 = (ly2 as usize) * self.chunk_size + (lx2 as usize);
                        chunk.cells.swap(ci1, ci2);
                        chunk.temps.swap(ci1, ci2);
                        chunk.pressure.swap(ci1, ci2);
                        chunk.gas_type.swap(ci1, ci2);
                        chunk.gas_density.swap(ci1, ci2);
                        chunk.electricity.swap(ci1, ci2);
                        chunk.cells[ci2].updated_this_tick = true;
                        chunk.modified = true;
                        chunk.mark_dirty(lx1, ly1);
                        chunk.mark_dirty(lx2, ly2);
                    }
                }
                (Some(i1), Some(i2)) => {
                    let (lo, hi) = if i1 < i2 { (i1, i2) } else { (i2, i1) };
                    let (left, right) = self.chunks_vec.split_at_mut(hi);
                    let (ch_lo, ch_hi) = if i1 < i2 {
                        (left.get_mut(lo), right.first_mut())
                    } else {
                        (right.first_mut(), left.get_mut(lo))
                    };
                    if let (Some(ch1), Some(ch2)) = (ch_lo, ch_hi) {
                        let ci1 = (ly1 as usize) * self.chunk_size + (lx1 as usize);
                        let ci2 = (ly2 as usize) * self.chunk_size + (lx2 as usize);
                        let (c1, t1, p1, gt1, gd1) = (
                            ch1.cells[ci1],
                            ch1.temps[ci1],
                            ch1.pressure[ci1],
                            ch1.gas_type[ci1],
                            ch1.gas_density[ci1],
                        );
                        ch1.cells[ci1] = ch2.cells[ci2];
                        ch1.temps[ci1] = ch2.temps[ci2];
                        ch1.pressure[ci1] = ch2.pressure[ci2];
                        ch1.gas_type[ci1] = ch2.gas_type[ci2];
                        ch1.gas_density[ci1] = ch2.gas_density[ci2];
                        ch1.cells[ci1].updated_this_tick = true;
                        ch1.modified = true;
                        ch1.mark_dirty(lx1, ly1);
                        ch2.cells[ci2] = c1;
                        ch2.temps[ci2] = t1;
                        ch2.pressure[ci2] = p1;
                        ch2.gas_type[ci2] = gt1;
                        ch2.gas_density[ci2] = gd1;
                        ch2.modified = true;
                        ch2.mark_dirty(lx2, ly2);
                    }
                }
                _ => {}
            }
        } else if cx1 == cx2 && cy1 == cy2 {
            let cs = self.chunk_size;
            let chunk = self.get_or_create_chunk(cx1, cy1);
            let i1 = (ly1 as usize) * cs + (lx1 as usize);
            let i2 = (ly2 as usize) * cs + (lx2 as usize);
            chunk.cells.swap(i1, i2);
            chunk.temps.swap(i1, i2);
            chunk.pressure.swap(i1, i2);
            chunk.gas_type.swap(i1, i2);
            chunk.gas_density.swap(i1, i2);
            chunk.electricity.swap(i1, i2);
            chunk.cells[i2].updated_this_tick = true;
            chunk.modified = true;
            chunk.mark_dirty(lx1, ly1);
            chunk.mark_dirty(lx2, ly2);
        } else {
            let c1 = self.get(x1, y1);
            let c2 = self.get(x2, y2);
            let t1 = self.get_temp(x1, y1);
            let t2 = self.get_temp(x2, y2);
            let p1 = self.get_pressure(x1, y1);
            let p2 = self.get_pressure(x2, y2);
            let g1 = self.get_gas(x1, y1);
            let g2 = self.get_gas(x2, y2);
            self.set(x1, y1, c2);
            self.set_temp(x1, y1, t2);
            self.set_pressure(x1, y1, p2);
            self.set_gas(x1, y1, g2.0, g2.1);
            self.set(x2, y2, c1);
            self.set_temp(x2, y2, t1);
            self.set_pressure(x2, y2, p1);
            self.set_gas(x2, y2, g1.0, g1.1);
            let cs = self.chunk_size;
            let chunk = self.get_or_create_chunk(cx1, cy1);
            let i = (ly1 as usize) * cs + (lx1 as usize);
            chunk.cells[i].updated_this_tick = true;
        }
    }

    pub fn set_cell_index(&mut self, i: usize, cell: Cell) {
        let x = (i % self.chunk_size) as i32;
        let y = (i / self.chunk_size) as i32;
        self.set(x, y, cell);
    }

    pub fn reset_tick_flags(&mut self) {
        if self.is_bounded() {
            for chunk in &mut self.chunks_vec {
                if !chunk.active {
                    continue;
                }
                for c in &mut chunk.cells {
                    c.updated_this_tick = false;
                }
            }
        } else {
            for chunk in self.chunks.values_mut() {
                if !chunk.active {
                    continue;
                }
                for c in &mut chunk.cells {
                    c.updated_this_tick = false;
                }
            }
        }
    }

    pub fn swap_modified_flags(&mut self) {
        if self.is_bounded() {
            for chunk in &mut self.chunks_vec {
                chunk.swap_modified_flags();
            }
        } else {
            for chunk in self.chunks.values_mut() {
                chunk.swap_modified_flags();
            }
        }
    }

    pub fn any_modified(&self) -> bool {
        if self.is_bounded() {
            self.chunks_vec.iter().any(|c| c.modified)
        } else {
            self.chunks.values().any(|c| c.modified)
        }
    }

    pub fn any_was_modified(&self) -> bool {
        if self.is_bounded() {
            self.chunks_vec.iter().any(|c| c.was_modified)
        } else {
            self.chunks.values().any(|c| c.was_modified)
        }
    }

    pub fn active_chunks(&self) -> Vec<(i32, i32)> {
        let mut out = Vec::new();
        if self.is_bounded() {
            for cy in 0..self.chunks_y as i32 {
                for cx in 0..self.chunks_x as i32 {
                    if let Some(idx) = self.chunk_index(cx, cy) {
                        if self.chunks_vec[idx].active {
                            out.push((cx, cy));
                        }
                    }
                }
            }
        } else {
            for (&(cx, cy), chunk) in &self.chunks {
                if chunk.active {
                    out.push((cx as i32, cy as i32));
                }
            }
        }
        out
    }

    pub fn all_chunk_coords(&self) -> Vec<(i32, i32)> {
        let mut out = Vec::new();
        if self.is_bounded() {
            for cy in 0..self.chunks_y as i32 {
                for cx in 0..self.chunks_x as i32 {
                    out.push((cx, cy));
                }
            }
        } else {
            for (&(cx, cy), _) in &self.chunks {
                out.push((cx as i32, cy as i32));
            }
        }
        out
    }

    pub fn is_chunk_modified(&self, cx: i32, cy: i32) -> bool {
        if self.is_bounded() {
            self.chunk_index(cx, cy)
                .and_then(|idx| self.chunks_vec.get(idx))
                .map(|c| c.modified || c.was_modified)
                .unwrap_or(false)
        } else {
            self.chunks
                .get(&(cx as i64, cy as i64))
                .map(|c| c.modified || c.was_modified)
                .unwrap_or(false)
        }
    }

    pub fn is_chunk_generated(&self, cx: i32, cy: i32) -> bool {
        if self.is_bounded() {
            self.chunk_index(cx, cy)
                .and_then(|idx| self.chunks_vec.get(idx))
                .map(|c| c.generated)
                .unwrap_or(false)
        } else {
            self.chunks
                .get(&(cx as i64, cy as i64))
                .map(|c| c.generated)
                .unwrap_or(false)
        }
    }

    pub fn is_chunk_empty(&self, cx: i32, cy: i32) -> bool {
        if self.is_bounded() {
            self.chunk_index(cx, cy)
                .and_then(|idx| self.chunks_vec.get(idx))
                .map(|c| c.is_empty())
                .unwrap_or(true)
        } else {
            self.chunks
                .get(&(cx as i64, cy as i64))
                .map(|c| c.is_empty())
                .unwrap_or(true)
        }
    }

    pub fn unload_chunk(&mut self, cx: i32, cy: i32) {
        if self.is_bounded() {
            return;
        }
        self.chunks.remove(&(cx as i64, cy as i64));
    }

    pub fn chunk_bounds(&self, cx: i32, cy: i32) -> (i32, i32, i32, i32) {
        let cs = self.chunk_size as i32;
        let x0 = cx * cs;
        let y0 = cy * cs;
        let x1 = x0 + cs;
        let y1 = y0 + cs;
        match self.bounds {
            Some((bx0, by0, bx1, by1)) => {
                let bx0_i = bx0 as i32;
                let by0_i = by0 as i32;
                let bx1_i = bx1 as i32;
                let by1_i = by1 as i32;
                (x0.max(bx0_i), y0.max(by0_i), x1.min(bx1_i), y1.min(by1_i))
            }
            None => (x0, y0, x1, y1),
        }
    }

    pub fn is_chunk_active(&self, cx: i32, cy: i32) -> bool {
        if self.is_bounded() {
            self.chunk_index(cx, cy)
                .and_then(|idx| self.chunks_vec.get(idx))
                .map(|c| c.active)
                .unwrap_or(false)
        } else {
            self.chunks
                .get(&(cx as i64, cy as i64))
                .map(|c| c.active)
                .unwrap_or(false)
        }
    }

    pub fn set_chunk_active(&mut self, cx: i32, cy: i32, active: bool) {
        if self.is_bounded() {
            if let Some(idx) = self.chunk_index(cx, cy) {
                if let Some(chunk) = self.chunks_vec.get_mut(idx) {
                    chunk.active = active;
                }
            }
        } else if let Some(chunk) = self.chunks.get_mut(&(cx as i64, cy as i64)) {
            chunk.active = active;
        }
    }

    pub fn get_chunk_dirty(&self, cx: i32, cy: i32) -> Option<(i32, i32, i32, i32)> {
        let ox = cx * self.chunk_size as i32;
        let oy = cy * self.chunk_size as i32;
        let dirty = if self.is_bounded() {
            self.chunk_index(cx, cy)
                .and_then(|idx| self.chunks_vec.get(idx))
                .and_then(|c| c.dirty)
        } else {
            self.chunks
                .get(&(cx as i64, cy as i64))
                .and_then(|c| c.dirty)
        };
        dirty.map(|(x0, y0, x1, y1)| (x0 + ox, y0 + oy, x1 + ox, y1 + oy))
    }

    pub fn set_chunk_dirty(&mut self, cx: i32, cy: i32, dirty: Option<(i32, i32, i32, i32)>) {
        let ox = cx * self.chunk_size as i32;
        let oy = cy * self.chunk_size as i32;
        let local = dirty.map(|(x0, y0, x1, y1)| (x0 - ox, y0 - oy, x1 - ox, y1 - oy));
        if self.is_bounded() {
            if let Some(idx) = self.chunk_index(cx, cy) {
                if let Some(chunk) = self.chunks_vec.get_mut(idx) {
                    chunk.dirty = local;
                }
            }
        } else if let Some(chunk) = self.chunks.get_mut(&(cx as i64, cy as i64)) {
            chunk.dirty = local;
        }
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
        if self.is_bounded() {
            for chunk in &mut self.chunks_vec {
                chunk.active = false;
            }
        } else {
            for chunk in self.chunks.values_mut() {
                chunk.active = false;
            }
        }
    }

    pub fn is_infinite(&self) -> bool {
        self.bounds.is_none()
    }

    pub fn cell_active(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return false;
        }
        let (cx, cy, _, _) = self.chunk_at(x, y);
        self.is_chunk_active(cx, cy)
    }

    pub fn chunk_cells(&self, cx: i32, cy: i32) -> Vec<(i32, i32, Cell)> {
        let (x0, y0, x1, y1) = self.chunk_bounds(cx, cy);
        let mut out = Vec::with_capacity(((x1 - x0) as usize) * ((y1 - y0) as usize));
        for y in y0..y1 {
            for x in x0..x1 {
                out.push((x, y, self.get(x, y)));
            }
        }
        out
    }

    pub fn load_chunk_cells(&mut self, cx: i32, cy: i32, cells: &[Cell]) {
        let cs = self.chunk_size as i32;
        let (x0, y0, x1, y1) = self.chunk_bounds(cx, cy);
        let chunk = self.get_or_create_chunk(cx, cy);
        let w = (x1 - x0) as usize;
        for y in y0..y1 {
            for x in x0..x1 {
                let i = ((y - y0) as usize) * w + (x - x0) as usize;
                if let Some(cell) = cells.get(i) {
                    let lx = x - cx * cs;
                    let ly = y - cy * cs;
                    chunk.set(lx, ly, *cell);
                }
            }
        }
        chunk.active = true;
    }

    pub fn fill_border(&mut self, mat: MaterialId) {
        let (x0, y0, x1, y1) = match self.bounds {
            Some(b) => b,
            None => return,
        };
        for x in x0..x1 {
            self.set_material(x as i32, y0 as i32, mat);
            self.set_material(x as i32, (y1 - 1) as i32, mat);
        }
        for y in y0..y1 {
            self.set_material(x0 as i32, y as i32, mat);
            self.set_material((x1 - 1) as i32, y as i32, mat);
        }
    }

    pub fn save_chunk(&self, path: &str, cx: i32, cy: i32) -> io::Result<()> {
        let chunk = if self.is_bounded() {
            match self
                .chunk_index(cx, cy)
                .and_then(|idx| self.chunks_vec.get(idx))
            {
                Some(c) => c,
                None => return Ok(()),
            }
        } else {
            match self.chunks.get(&(cx as i64, cy as i64)) {
                Some(c) => c,
                None => return Ok(()),
            }
        };
        let area = self.chunk_size * self.chunk_size;
        let mut bytes = Vec::with_capacity(5 + area * 8 + area * 4 + area * 2 + area + area * 3);
        bytes.extend_from_slice(b"VWM1");
        bytes.push(1);
        for c in &chunk.cells {
            bytes.extend_from_slice(&c.to_bytes());
        }
        for t in &chunk.temps {
            bytes.extend_from_slice(&t.to_le_bytes());
        }
        for i in 0..area {
            bytes.push(chunk.gas_type[i]);
            bytes.push(chunk.gas_density[i]);
        }
        for &p in &chunk.pressure {
            bytes.push(p);
        }
        for l in &chunk.light {
            bytes.extend_from_slice(&l[..]);
        }
        for &e in &chunk.electricity {
            bytes.push(e);
        }
        let dir = Path::new(path);
        if let Some(parent) = dir.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, bytes)
    }

    pub fn load_chunk(&mut self, path: &str, cx: i32, cy: i32) -> io::Result<()> {
        self.load_chunk_from_path(Path::new(path), cx, cy)
    }

    fn load_chunk_from_path(&mut self, path: &Path, cx: i32, cy: i32) -> io::Result<()> {
        let data = std::fs::read(path)?;
        let cs = self.chunk_size;
        let area = cs * cs;
        let bounds = self.chunk_bounds(cx, cy);
        let chunk = self.get_or_create_chunk(cx, cy);
        if data.len() >= 5 && &data[0..4] == b"VWM1" {
            let mut off = 5;
            for i in 0..area {
                if off + 8 > data.len() {
                    break;
                }
                let cell = Cell::from_bytes(&data[off..off + 8]);
                let lx = (i % cs) as i32;
                let ly = (i / cs) as i32;
                chunk.set(lx, ly, cell);
                off += 8;
            }
            for i in 0..area {
                if off + 4 > data.len() {
                    break;
                }
                chunk.temps[i] =
                    f32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
                off += 4;
            }
            for i in 0..area {
                if off + 2 > data.len() {
                    break;
                }
                chunk.gas_type[i] = data[off];
                chunk.gas_density[i] = data[off + 1];
                off += 2;
            }
            for i in 0..area {
                if off >= data.len() {
                    break;
                }
                chunk.pressure[i] = data[off];
                off += 1;
            }
            for i in 0..area {
                if off + 3 > data.len() {
                    break;
                }
                chunk.light[i] = [data[off], data[off + 1], data[off + 2]];
                off += 3;
            }
            for i in 0..area {
                if off >= data.len() {
                    break;
                }
                chunk.electricity[i] = data[off];
                off += 1;
            }
        } else {
            let (x0, y0, x1, y1) = bounds;
            let w = (x1 - x0) as usize;
            let h = (y1 - y0) as usize;
            let expected = w * h * 12;
            if data.len() != expected {
                return Err(io::Error::other("chunk file size mismatch"));
            }
            let mut i = 0usize;
            for y in y0..y1 {
                for x in x0..x1 {
                    let lx = x - cx * cs as i32;
                    let ly = y - cy * cs as i32;
                    let cell = Cell::from_bytes(&data[i * 12..(i + 1) * 12]);
                    chunk.set(lx, ly, cell);
                    if cell.material != MaterialId::Empty {
                        chunk.set_temp(lx, ly, default_temp(cell.material));
                    }
                    i += 1;
                }
            }
        }
        chunk.active = true;
        chunk.generated = true;
        Ok(())
    }

    pub fn load_all_modified(&mut self) -> io::Result<()> {
        let cache_dir = match self.cache_dir {
            Some(ref dir) => dir,
            None => return Ok(()),
        };
        let base = Path::new(cache_dir).join(format!("seed_{}", self.seed));
        if !base.exists() {
            return Ok(());
        }
        for entry in std::fs::read_dir(base)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(rest) = name.strip_prefix("chunk_") {
                let rest = rest.strip_suffix(".bin").unwrap_or(rest);
                let parts: Vec<&str> = rest.split('_').collect();
                if parts.len() == 2 {
                    if let (Ok(cx), Ok(cy)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                        let path = entry.path();
                        self.load_chunk_from_path(&path, cx, cy)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn save_all_modified(&self) -> io::Result<()> {
        let cache_dir = match self.cache_dir {
            Some(ref dir) => dir,
            None => return Ok(()),
        };
        if self.is_bounded() {
            for cy in 0..self.chunks_y as i32 {
                for cx in 0..self.chunks_x as i32 {
                    if let Some(idx) = self.chunk_index(cx, cy) {
                        let chunk = &self.chunks_vec[idx];
                        if chunk.modified || chunk.was_modified {
                            let path = chunk_path(cache_dir, self.seed, cx, cy);
                            self.save_chunk(path.to_str().unwrap(), cx, cy)?;
                        }
                    }
                }
            }
        } else {
            for (&(cx, cy), chunk) in &self.chunks {
                if chunk.modified || chunk.was_modified {
                    let path = chunk_path(cache_dir, self.seed, cx as i32, cy as i32);
                    self.save_chunk(path.to_str().unwrap(), cx as i32, cy as i32)?;
                }
            }
        }
        Ok(())
    }

    pub fn unload_distant(&mut self, px: i32, py: i32, radius: i32) {
        if self.is_bounded() {
            return;
        }
        let (pcx, pcy, _, _) = self.chunk_at(px, py);
        let mut to_remove = Vec::new();
        for (&(cx, cy), chunk) in &self.chunks {
            if (cx - pcx as i64).abs() > radius as i64 || (cy - pcy as i64).abs() > radius as i64 {
                if chunk.modified || chunk.was_modified {
                    if let Some(ref dir) = self.cache_dir {
                        let path = chunk_path(dir, self.seed, cx as i32, cy as i32);
                        let _ = self.save_chunk(path.to_str().unwrap(), cx as i32, cy as i32);
                    }
                }
                to_remove.push((cx, cy));
            }
        }
        for key in to_remove {
            self.chunks.remove(&key);
        }
    }

    pub fn ensure_loaded(&mut self, px: i32, py: i32, radius: i32) {
        let (pcx, pcy, _, _) = self.chunk_at(px, py);
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let cx = pcx + dx;
                let cy = pcy + dy;
                let _ = self.ensure_chunk(cx, cy);
                self.set_chunk_active(cx, cy, true);
            }
        }
    }
}

pub fn chunk_path(root: &str, seed: u64, cx: i32, cy: i32) -> PathBuf {
    Path::new(root)
        .join(format!("seed_{}", seed))
        .join(format!("chunk_{}_{}.bin", cx, cy))
}
