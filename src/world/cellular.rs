use crate::world::cell::{Cell, MaterialId};
use crate::world::chunk::CHUNK_SIZE;
use crate::world::chunked_grid::ChunkedGrid;

pub struct CellularAutomaton {
    tick: u64,
    rng_state: u64,
    light_tick: u64,
}

impl CellularAutomaton {
    pub fn new() -> Self {
        Self {
            tick: 0,
            rng_state: 0x1234567890ABCDEF,
            light_tick: 0,
        }
    }

    pub fn seed(&mut self, state: u64) {
        self.rng_state = state;
    }

    #[inline]
    fn rand(&mut self) -> u32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        (self.rng_state & 0xFFFFFFFF) as u32
    }

    #[inline]
    fn rand_bool(&mut self) -> bool {
        self.rand() & 1 == 1
    }

    #[inline]
    fn apply_cell_rule(&mut self, grid: &mut ChunkedGrid, x: i32, y: i32) {
        let cell = grid.get(x, y);
        if cell.updated_this_tick || cell.is_empty() || cell.is_static() {
            return;
        }
        match cell.material {
            MaterialId::Sand => self.update_sand(grid, x, y),
            MaterialId::Water => {
                self.update_water(grid, x, y);
                grid.mark_dirty(x, y);
            }
            MaterialId::Lava => {
                self.update_lava(grid, x, y);
                grid.mark_dirty(x, y);
            }
            MaterialId::Steam => {
                self.update_steam(grid, x, y);
                grid.mark_dirty(x, y);
            }
            MaterialId::Fire => {
                self.update_fire(grid, x, y);
                grid.mark_dirty(x, y);
            }
            MaterialId::Smoke => {
                self.update_smoke(grid, x, y);
                grid.mark_dirty(x, y);
            }
            MaterialId::Acid => {
                self.update_acid(grid, x, y);
                grid.mark_dirty(x, y);
            }
            MaterialId::Flesh => self.update_flesh(grid, x, y),
            MaterialId::Grass => self.update_grass(grid, x, y),
            MaterialId::Dirt => self.update_dirt(grid, x, y),
            _ => {}
        }
    }

    pub fn random_u32(&mut self) -> u32 {
        self.rand()
    }

    pub fn random_usize(&mut self, max: usize) -> usize {
        if max == 0 {
            return 0;
        }
        (self.rand() as usize) % max
    }

    pub fn step(&mut self, grid: &mut ChunkedGrid) {
        let flip = self.rand_bool();

        let mut active = grid.active_chunks();
        active.sort_by(|(ax, ay), (bx, by)| by.cmp(ay).then(bx.cmp(ax)));

        let mut pre_dirty: std::collections::HashMap<(i32, i32), (i32, i32, i32, i32)> =
            std::collections::HashMap::new();
        for &(cx, cy) in &active {
            if let Some(d) = grid.get_chunk_dirty(cx, cy) {
                pre_dirty.insert((cx, cy), d);
            }
        }

        for &(cx, cy) in &active {
            let dirty = match grid.get_chunk_dirty(cx, cy) {
                Some(d) => d,
                None => continue,
            };
            let (min_x, min_y, max_x, max_y) = dirty;
            let cs = grid.chunk_size as i32;
            let ox = cx * cs;
            let oy = cy * cs;
            if let Some(chunk) = grid.get_chunk_mut(cx, cy) {
                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        let lx = x - ox;
                        let ly = y - oy;
                        if lx >= 0 && lx < cs && ly >= 0 && ly < cs {
                            let idx = (ly as usize) * CHUNK_SIZE + (lx as usize);
                            chunk.cells[idx].updated_this_tick = false;
                        }
                    }
                }
            }
        }

        for &(cx, cy) in &active {
            let dirty = match grid.get_chunk_dirty(cx, cy) {
                Some(d) => d,
                None => continue,
            };
            let (min_x, min_y, max_x, max_y) = dirty;
            let dw = max_x - min_x + 1;
            let dh = max_y - min_y + 1;
            if dw * dh > 2048 {
                grid.set_chunk_dirty(cx, cy, None);
                grid.mark_dirty(min_x + dw / 4, min_y + dh / 4);
                grid.mark_dirty(max_x - dw / 4, max_y - dh / 4);
                continue;
            }

            grid.set_chunk_dirty(cx, cy, None);

            for y in (min_y..=max_y).rev() {
                if flip {
                    for x in min_x..=max_x {
                        self.apply_cell_rule(grid, x, y);
                    }
                } else {
                    for x in (min_x..=max_x).rev() {
                        self.apply_cell_rule(grid, x, y);
                    }
                }
            }
        }

        self.heat_transfer(grid, &active, &pre_dirty);
        self.gas_step(grid, &active, &pre_dirty);
        self.pressure_step(grid, &active, &pre_dirty);
        self.light_step(grid, &active);
        self.tick += 1;
    }

    fn try_move_down(
        &mut self,
        grid: &mut ChunkedGrid,
        x: i32,
        y: i32,
        _mat: MaterialId,
        density: f32,
    ) {
        let below = grid.get(x, y + 1);
        if below.is_empty() || (below.is_liquid() && below.density() < density) {
            grid.cells_swap(x, y, x, y + 1);
            return;
        }

        let dir = if self.rand_bool() { 1 } else { -1 };
        let dl = grid.get(x - dir, y + 1);
        let dr = grid.get(x + dir, y + 1);

        let can_left = grid.in_bounds(x - dir, y + 1)
            && (dl.is_empty() || (dl.is_liquid() && dl.density() < density));
        let can_right = grid.in_bounds(x + dir, y + 1)
            && (dr.is_empty() || (dr.is_liquid() && dr.density() < density));

        if can_left && can_right {
            if self.rand_bool() {
                grid.cells_swap(x, y, x - dir, y + 1);
            } else {
                grid.cells_swap(x, y, x + dir, y + 1);
            }
        } else if can_left {
            grid.cells_swap(x, y, x - dir, y + 1);
        } else if can_right {
            grid.cells_swap(x, y, x + dir, y + 1);
        }
    }

    fn update_sand(&mut self, grid: &mut ChunkedGrid, x: i32, y: i32) {
        self.try_move_down(grid, x, y, MaterialId::Sand, 1.5);
    }

    fn update_water(&mut self, grid: &mut ChunkedGrid, x: i32, y: i32) {
        let below = grid.get(x, y + 1);
        if below.is_empty() || (below.is_liquid() && below.density() < 1.0) {
            grid.cells_swap(x, y, x, y + 1);
        } else {
            let dir = if self.rand_bool() { 1 } else { -1 };
            let dl = grid.get(x - dir, y + 1);
            let dr = grid.get(x + dir, y + 1);

            let can_dl = grid.in_bounds(x - dir, y + 1)
                && (dl.is_empty() || (dl.is_liquid() && dl.density() < 1.0));
            let can_dr = grid.in_bounds(x + dir, y + 1)
                && (dr.is_empty() || (dr.is_liquid() && dr.density() < 1.0));

            if can_dl && can_dr {
                if self.rand_bool() {
                    grid.cells_swap(x, y, x - dir, y + 1);
                } else {
                    grid.cells_swap(x, y, x + dir, y + 1);
                }
            } else if can_dl {
                grid.cells_swap(x, y, x - dir, y + 1);
            } else if can_dr {
                grid.cells_swap(x, y, x + dir, y + 1);
            } else {
                let can_l = grid.in_bounds(x - dir, y) && grid.get(x - dir, y).is_empty();
                let can_r = grid.in_bounds(x + dir, y) && grid.get(x + dir, y).is_empty();
                if can_l && can_r {
                    if self.rand_bool() {
                        grid.cells_swap(x, y, x - dir, y);
                    } else {
                        grid.cells_swap(x, y, x + dir, y);
                    }
                } else if can_l {
                    grid.cells_swap(x, y, x - dir, y);
                } else if can_r {
                    grid.cells_swap(x, y, x + dir, y);
                }
            }
        }

        let temp = grid.get_temp(x, y);
        if temp > 100.0 {
            let mut new = grid.get(x, y);
            new.material = MaterialId::Steam;
            new.updated_this_tick = true;
            grid.set(x, y, new);
            grid.set_temp(x, y, 110.0);
        }
    }

    fn update_lava(&mut self, grid: &mut ChunkedGrid, x: i32, y: i32) {
        let temp = grid.get_temp(x, y);
        if temp < 400.0 {
            let mut new = grid.get(x, y);
            new.material = MaterialId::Stone;
            grid.set(x, y, new);
            return;
        }

        let below = grid.get(x, y + 1);
        if below.is_empty() {
            grid.cells_swap(x, y, x, y + 1);
            return;
        }

        let dir = if self.rand_bool() { 1 } else { -1 };
        if grid.in_bounds(x - dir, y + 1) && grid.get(x - dir, y + 1).is_empty() {
            grid.cells_swap(x, y, x - dir, y + 1);
            return;
        }
        if grid.in_bounds(x + dir, y + 1) && grid.get(x + dir, y + 1).is_empty() {
            grid.cells_swap(x, y, x + dir, y + 1);
            return;
        }

        if self.rand() % 10 == 0 {
            if grid.in_bounds(x - dir, y) && grid.get(x - dir, y).is_empty() {
                grid.cells_swap(x, y, x - dir, y);
            } else if grid.in_bounds(x + dir, y) && grid.get(x + dir, y).is_empty() {
                grid.cells_swap(x, y, x + dir, y);
            }
        }

        self.lava_interact(grid, x, y);
    }

    fn lava_interact(&mut self, grid: &mut ChunkedGrid, x: i32, y: i32) {
        for &(dx, dy) in &NEIGHBORS4 {
            let nx = x + dx;
            let ny = y + dy;
            if !grid.in_bounds(nx, ny) {
                continue;
            }
            let neighbor = grid.get(nx, ny);
            let n_temp = grid.get_temp(nx, ny);
            match neighbor.material {
                MaterialId::Water => {
                    grid.set(nx, ny, Cell::new(MaterialId::Steam));
                    grid.set_temp(nx, ny, 150.0);
                    let lava_temp = grid.get_temp(x, y);
                    grid.set_temp(x, y, lava_temp - 50.0);
                }
                MaterialId::Wood | MaterialId::Grass | MaterialId::Flesh if n_temp < 300.0 => {
                    let mut new_n = neighbor;
                    new_n.material = MaterialId::Fire;
                    new_n.updated_this_tick = true;
                    grid.set(nx, ny, new_n);
                    grid.set_temp(nx, ny, 400.0);
                }
                MaterialId::Sand if n_temp > 1700.0 => {
                    grid.set(nx, ny, Cell::new(MaterialId::Stone));
                }
                _ => {}
            }
        }
    }

    fn update_steam(&mut self, grid: &mut ChunkedGrid, x: i32, y: i32) {
        let temp = grid.get_temp(x, y);
        if temp < 80.0 {
            let mut new = grid.get(x, y);
            new.material = MaterialId::Water;
            grid.set(x, y, new);
            grid.set_temp(x, y, 50.0);
            return;
        }

        if y > 0 && grid.get(x, y - 1).is_empty() {
            grid.cells_swap(x, y, x, y - 1);
            return;
        }

        let dir = if self.rand_bool() { 1 } else { -1 };
        if grid.in_bounds(x - dir, y - 1) && grid.get(x - dir, y - 1).is_empty() {
            grid.cells_swap(x, y, x - dir, y - 1);
            return;
        }
        if grid.in_bounds(x + dir, y - 1) && grid.get(x + dir, y - 1).is_empty() {
            grid.cells_swap(x, y, x + dir, y - 1);
            return;
        }

        if self.rand() % 3 == 0 {
            if grid.in_bounds(x - dir, y) && grid.get(x - dir, y).is_empty() {
                grid.cells_swap(x, y, x - dir, y);
            } else if grid.in_bounds(x + dir, y) && grid.get(x + dir, y).is_empty() {
                grid.cells_swap(x, y, x + dir, y);
            }
        }
    }

    fn update_fire(&mut self, grid: &mut ChunkedGrid, x: i32, y: i32) {
        let _cell = grid.get(x, y);
        let temp = grid.get_temp(x, y);

        let (_egt, egd) = grid.get_gas(x, y);
        if egd < 200 {
            grid.set_gas(x, y, 3, (egd + 5).min(255));
        }
        if y > 0 {
            let above = grid.get(x, y - 1);
            if above.is_empty() {
                let (agt, agd) = grid.get_gas(x, y - 1);
                if agt == 0 && agd < 100 {
                    grid.set_gas(x, y - 1, 1, (agd + 3).min(200));
                }
            }
        }

        for &(dx, dy) in &NEIGHBORS4 {
            let nx = x + dx;
            let ny = y + dy;
            if !grid.in_bounds(nx, ny) {
                continue;
            }
            let neighbor = grid.get(nx, ny);
            let reg = crate::world::material::MaterialRegistry::instance();
            let mat = reg.get(neighbor.material);
            let n_temp = grid.get_temp(nx, ny);
            if mat.flammable && n_temp < mat.ignition_temp {
                let mut new_n = neighbor;
                new_n.material = MaterialId::Fire;
                new_n.updated_this_tick = true;
                grid.set(nx, ny, new_n);
                grid.set_temp(nx, ny, 400.0);
            }
        }

        let (_gt, gd) = grid.get_gas(x, y);
        if temp < 100.0 || self.rand() % 20 == 0 || gd > 150 {
            if self.rand() % 3 == 0 {
                grid.set(x, y, Cell::new(MaterialId::Smoke));
                grid.set_temp(x, y, 120.0);
            } else {
                grid.set(x, y, Cell::empty());
            }
            return;
        }

        grid.set_temp(x, y, temp - 15.0);

        if y > 0 && grid.get(x, y - 1).is_empty() && self.rand() % 2 == 0 {
            grid.cells_swap(x, y, x, y - 1);
        }
    }

    fn update_smoke(&mut self, grid: &mut ChunkedGrid, x: i32, y: i32) {
        if self.rand() % 60 == 0 {
            grid.set(x, y, Cell::empty());
            return;
        }

        if y > 0 && grid.get(x, y - 1).is_empty() {
            grid.cells_swap(x, y, x, y - 1);
            return;
        }

        let dir = if self.rand_bool() { 1 } else { -1 };
        if grid.in_bounds(x - dir, y - 1) && grid.get(x - dir, y - 1).is_empty() {
            grid.cells_swap(x, y, x - dir, y - 1);
        } else if grid.in_bounds(x + dir, y - 1) && grid.get(x + dir, y - 1).is_empty() {
            grid.cells_swap(x, y, x + dir, y - 1);
        } else if grid.in_bounds(x - dir, y) && grid.get(x - dir, y).is_empty() {
            grid.cells_swap(x, y, x - dir, y);
        } else if grid.in_bounds(x + dir, y) && grid.get(x + dir, y).is_empty() {
            grid.cells_swap(x, y, x + dir, y);
        }
    }

    fn update_acid(&mut self, grid: &mut ChunkedGrid, x: i32, y: i32) {
        for &(dx, dy) in &NEIGHBORS4 {
            let nx = x + dx;
            let ny = y + dy;
            if !grid.in_bounds(nx, ny) {
                continue;
            }
            let neighbor = grid.get(nx, ny);
            if neighbor.material != MaterialId::Empty
                && neighbor.material != MaterialId::Acid
                && neighbor.material != MaterialId::Stone
                && self.rand() % 4 == 0
            {
                if neighbor.material == MaterialId::Flesh || neighbor.material == MaterialId::Wood {
                    let (gt, _gd) = grid.get_gas(nx, ny);
                    if gt == 0 {
                        grid.set_gas(nx, ny, 2, 80);
                    }
                }
                grid.set(nx, ny, Cell::empty());
                if self.rand() % 2 == 0 {
                    grid.set(x, y, Cell::empty());
                    return;
                }
            }
        }

        let below = grid.get(x, y + 1);
        if below.is_empty() || (below.is_liquid() && below.density() < 1.2) {
            grid.cells_swap(x, y, x, y + 1);
            return;
        }

        let dir = if self.rand_bool() { 1 } else { -1 };
        if grid.in_bounds(x - dir, y + 1) && grid.get(x - dir, y + 1).is_empty() {
            grid.cells_swap(x, y, x - dir, y + 1);
        } else if grid.in_bounds(x + dir, y + 1) && grid.get(x + dir, y + 1).is_empty() {
            grid.cells_swap(x, y, x + dir, y + 1);
        } else if grid.in_bounds(x - dir, y) && grid.get(x - dir, y).is_empty() {
            grid.cells_swap(x, y, x - dir, y);
        } else if grid.in_bounds(x + dir, y) && grid.get(x + dir, y).is_empty() {
            grid.cells_swap(x, y, x + dir, y);
        }
    }

    fn update_flesh(&mut self, grid: &mut ChunkedGrid, x: i32, y: i32) {
        let temp = grid.get_temp(x, y);
        if temp > 200.0 {
            let mut new = grid.get(x, y);
            new.material = MaterialId::Fire;
            new.updated_this_tick = true;
            grid.set(x, y, new);
            grid.set_temp(x, y, 400.0);
        }
    }

    fn update_grass(&mut self, grid: &mut ChunkedGrid, x: i32, y: i32) {
        let temp = grid.get_temp(x, y);
        if temp > 250.0 {
            let mut new = grid.get(x, y);
            new.material = MaterialId::Fire;
            new.updated_this_tick = true;
            grid.set(x, y, new);
            grid.set_temp(x, y, 400.0);
        }
    }

    fn update_dirt(&mut self, grid: &mut ChunkedGrid, x: i32, y: i32) {
        let temp = grid.get_temp(x, y);
        if temp < 0.0 {
            let mut new = grid.get(x, y);
            new.material = MaterialId::Stone;
            grid.set(x, y, new);
        }
    }

    fn heat_transfer(
        &mut self,
        grid: &mut ChunkedGrid,
        active: &[(i32, i32)],
        pre_dirty: &std::collections::HashMap<(i32, i32), (i32, i32, i32, i32)>,
    ) {
        let reg = crate::world::material::MaterialRegistry::instance();
        let cs = CHUNK_SIZE as i32;
        for &(cx, cy) in active {
            let dirty = grid.get_chunk_dirty(cx, cy);
            let pre = pre_dirty.get(&(cx, cy)).copied();
            let (min_x, min_y, max_x, max_y) = match (dirty, pre) {
                (Some(d), Some(p)) => (d.0.min(p.0), d.1.min(p.1), d.2.max(p.2), d.3.max(p.3)),
                (Some(d), None) => d,
                (None, Some(p)) => p,
                (None, None) => continue,
            };
            let w = max_x - min_x + 1;
            let h = max_y - min_y + 1;
            if w * h > 2048 {
                continue;
            }
            let ox = cx * cs;
            let oy = cy * cs;
            let mut edge_temps: Vec<((i32, i32), f32)> = Vec::new();
            if min_x < ox || max_x >= ox + cs || min_y < oy || max_y >= oy + cs {
                for y in (min_y - 1)..=(max_y + 1) {
                    for x in (min_x - 1)..=(max_x + 1) {
                        let lx = x - ox;
                        let ly = y - oy;
                        if lx < 0 || lx >= cs || ly < 0 || ly >= cs {
                            edge_temps.push(((x, y), grid.get_temp(x, y)));
                        }
                    }
                }
            }
            let chunk = match grid.get_chunk_mut(cx, cy) {
                Some(c) => c,
                None => continue,
            };
            for ly in 0..h {
                let wy = min_y + ly - oy;
                if wy < 0 || wy >= cs {
                    continue;
                }
                for lx in 0..w {
                    let wx = min_x + lx - ox;
                    if wx < 0 || wx >= cs {
                        continue;
                    }
                    let idx = (wy as usize) * CHUNK_SIZE + (wx as usize);
                    let mat = reg.get(chunk.cells[idx].material);
                    let k = mat.heat_conductivity;
                    if k == 0.0 {
                        continue;
                    }
                    let cur = chunk.temps[idx];
                    let mut sum = 0.0f32;
                    let mut count = 0u32;
                    for &(dx, dy) in &NEIGHBORS4 {
                        let nx = wx + dx;
                        let ny = wy + dy;
                        if nx < 0 || nx >= cs || ny < 0 || ny >= cs {
                            let wx2 = ox + nx;
                            let wy2 = oy + ny;
                            if let Some((_, t)) = edge_temps
                                .iter()
                                .find(|((ex, ey), _)| *ex == wx2 && *ey == wy2)
                            {
                                sum += *t;
                            } else {
                                sum += 20.0;
                            }
                            count += 1;
                            continue;
                        }
                        let ni = (ny as usize) * CHUNK_SIZE + (nx as usize);
                        sum += chunk.temps[ni];
                        count += 1;
                    }
                    if count > 0 {
                        let avg = sum / count as f32;
                        let new_temp = cur + (avg - cur) * k * 0.1;
                        if (new_temp - cur).abs() > 0.01 {
                            chunk.temps[idx] = new_temp;
                            chunk.mark_dirty(wx, wy);
                        }
                    }
                }
            }
        }
    }

    fn gas_step(
        &mut self,
        grid: &mut ChunkedGrid,
        active: &[(i32, i32)],
        pre_dirty: &std::collections::HashMap<(i32, i32), (i32, i32, i32, i32)>,
    ) {
        let cs = CHUNK_SIZE as i32;
        for &(cx, cy) in active {
            let dirty = grid.get_chunk_dirty(cx, cy);
            let pre = pre_dirty.get(&(cx, cy)).copied();
            let (min_x, min_y, max_x, max_y) = match (dirty, pre) {
                (Some(d), Some(p)) => (d.0.min(p.0), d.1.min(p.1), d.2.max(p.2), d.3.max(p.3)),
                (Some(d), None) => d,
                (None, Some(p)) => p,
                (None, None) => continue,
            };
            let w = max_x - min_x + 1;
            let h = max_y - min_y + 1;
            if w * h > 2048 {
                continue;
            }
            let ox = cx * cs;
            let oy = cy * cs;
            let chunk = match grid.get_chunk_mut(cx, cy) {
                Some(c) => c,
                None => continue,
            };
            let has_gas = chunk.gas_density.iter().any(|&d| d > 0);
            if !has_gas {
                continue;
            }
            for ly in 0..h {
                let wy = min_y + ly - oy;
                if wy < 0 || wy >= cs {
                    continue;
                }
                for lx in 0..w {
                    let wx = min_x + lx - ox;
                    if wx < 0 || wx >= cs {
                        continue;
                    }
                    let idx = (wy as usize) * CHUNK_SIZE + (wx as usize);
                    let gt = chunk.gas_type[idx];
                    let gd = chunk.gas_density[idx];
                    if gd == 0 {
                        continue;
                    }

                    if gt == 4 && chunk.temps[idx] < 80.0 {
                        let mut new = chunk.cells[idx];
                        new.material = MaterialId::Water;
                        chunk.cells[idx] = new;
                        chunk.temps[idx] = 50.0;
                        chunk.gas_type[idx] = 0;
                        chunk.gas_density[idx] = 0;
                        chunk.modified = true;
                        chunk.mark_dirty(wx, wy);
                        continue;
                    }

                    if wy > 0 {
                        let above_idx = ((wy - 1) as usize) * CHUNK_SIZE + (wx as usize);
                        let above_cell = chunk.cells[above_idx];
                        if above_cell.is_empty() || above_cell.is_gas() {
                            let agd = chunk.gas_density[above_idx];
                            if agd < gd {
                                chunk.gas_type[above_idx] = gt;
                                chunk.gas_density[above_idx] = gd;
                                chunk.gas_type[idx] = 0;
                                chunk.gas_density[idx] = 0;
                                chunk.mark_dirty(wx, wy);
                                chunk.mark_dirty(wx, wy - 1);
                                continue;
                            }
                        }
                    }

                    let dir = if self.rand_bool() { 1i32 } else { -1i32 };
                    for &d in &[dir, -dir] {
                        let nx = wx + d;
                        if nx >= 0 && nx < cs {
                            let side_cell = chunk.cells[(wy as usize) * CHUNK_SIZE + (nx as usize)];
                            if side_cell.is_empty() || side_cell.is_gas() {
                                let ni = (wy as usize) * CHUNK_SIZE + (nx as usize);
                                let sgd = chunk.gas_density[ni];
                                if sgd < gd.saturating_sub(5) {
                                    let avg = (gd + sgd) / 2;
                                    chunk.gas_type[ni] = gt;
                                    chunk.gas_density[ni] = avg;
                                    chunk.gas_density[idx] = gd.saturating_sub(avg - sgd);
                                    chunk.mark_dirty(wx, wy);
                                    chunk.mark_dirty(nx, wy);
                                    break;
                                }
                            }
                        }
                    }

                    if (gt == 1 || gt == 2) && gd > 0 && self.rand() % 120 == 0 {
                        chunk.gas_density[idx] = gd.saturating_sub(1);
                    }
                }
            }
        }
    }

    fn pressure_step(
        &mut self,
        grid: &mut ChunkedGrid,
        active: &[(i32, i32)],
        pre_dirty: &std::collections::HashMap<(i32, i32), (i32, i32, i32, i32)>,
    ) {
        let cs = CHUNK_SIZE as i32;
        for &(cx, cy) in active {
            let dirty = grid.get_chunk_dirty(cx, cy);
            let pre = pre_dirty.get(&(cx, cy)).copied();
            let (min_x, min_y, max_x, max_y) = match (dirty, pre) {
                (Some(d), Some(p)) => (d.0.min(p.0), d.1.min(p.1), d.2.max(p.2), d.3.max(p.3)),
                (Some(d), None) => d,
                (None, Some(p)) => p,
                (None, None) => continue,
            };
            let w = max_x - min_x + 1;
            let h = max_y - min_y + 1;
            if w * h > 2048 {
                continue;
            }
            let ox = cx * cs;
            let oy = cy * cs;
            let chunk = match grid.get_chunk_mut(cx, cy) {
                Some(c) => c,
                None => continue,
            };
            let needs_pressure = chunk.pressure.iter().any(|&p| p != 128);
            if !needs_pressure {
                continue;
            }
            for ly in 0..h {
                let wy = min_y + ly - oy;
                if wy < 0 || wy >= cs {
                    continue;
                }
                for lx in 0..w {
                    let wx = min_x + lx - ox;
                    if wx < 0 || wx >= cs {
                        continue;
                    }
                    let idx = (wy as usize) * CHUNK_SIZE + (wx as usize);
                    if chunk.cells[idx].is_solid() {
                        continue;
                    }
                    let cur_p = chunk.pressure[idx] as i32;
                    let mut sum = 0i32;
                    let mut count = 0i32;
                    for &(dx, dy) in &NEIGHBORS4 {
                        let nx = wx + dx;
                        let ny = wy + dy;
                        if nx < 0 || nx >= cs || ny < 0 || ny >= cs {
                            continue;
                        }
                        let ni = (ny as usize) * CHUNK_SIZE + (nx as usize);
                        if chunk.cells[ni].is_solid() {
                            continue;
                        }
                        sum += chunk.pressure[ni] as i32;
                        count += 1;
                    }
                    if count > 0 {
                        let avg = sum / count;
                        let new_p = cur_p + (avg - cur_p) / 8;
                        if new_p != cur_p {
                            chunk.pressure[idx] = new_p.clamp(0, 255) as u8;
                            chunk.mark_dirty(wx, wy);
                        }
                    }
                }
            }
        }
    }

    fn light_step(&mut self, grid: &mut ChunkedGrid, active: &[(i32, i32)]) {
        self.light_tick += 1;
        if self.light_tick % 20 != 0 {
            return;
        }
        let cs = CHUNK_SIZE as i32;
        let mut sources: Vec<(i32, i32, i32, [u8; 3])> = Vec::new();
        for &(cx, cy) in active {
            let ox = cx * cs;
            let oy = cy * cs;
            if let Some(chunk) = grid.get_chunk(cx, cy) {
                for ly in 0..cs {
                    for lx in 0..cs {
                        let idx = (ly as usize) * CHUNK_SIZE + (lx as usize);
                        if let Some((radius, color)) = material_light(chunk.cells[idx].material) {
                            sources.push((ox + lx, oy + ly, radius as i32, color));
                            if sources.len() >= 64 {
                                break;
                            }
                        }
                    }
                    if sources.len() >= 64 {
                        break;
                    }
                }
            }
        }
        if sources.is_empty() {
            for &(cx, cy) in active {
                if let Some(chunk) = grid.get_chunk_mut(cx, cy) {
                    for l in chunk.light.iter_mut() {
                        *l = [0, 0, 0];
                    }
                }
            }
            return;
        }
        for &(cx, cy) in active {
            if let Some(chunk) = grid.get_chunk_mut(cx, cy) {
                for l in chunk.light.iter_mut() {
                    *l = [0, 0, 0];
                }
            }
        }
        for &(sx, sy, radius, color) in &sources {
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let tx = sx + dx;
                    let ty = sy + dy;
                    if !grid.in_bounds(tx, ty) {
                        continue;
                    }
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq > radius * radius {
                        continue;
                    }
                    let dist = (dist_sq as f32).sqrt();
                    if !line_of_sight(grid, sx, sy, tx, ty) {
                        continue;
                    }
                    let t = 1.0 - dist / radius as f32;
                    let atten = t * t;
                    let cur = grid.get_light(tx, ty);
                    let nr = (cur[0] as f32 + color[0] as f32 * atten).min(255.0) as u8;
                    let ng = (cur[1] as f32 + color[1] as f32 * atten).min(255.0) as u8;
                    let nb = (cur[2] as f32 + color[2] as f32 * atten).min(255.0) as u8;
                    grid.set_light(tx, ty, [nr, ng, nb]);
                }
            }
        }
    }
}

fn material_light(mat: MaterialId) -> Option<(u32, [u8; 3])> {
    match mat {
        MaterialId::Lava => Some((12, [255, 120, 30])),
        MaterialId::Fire => Some((6, [255, 180, 60])),
        _ => None,
    }
}

fn line_of_sight(grid: &ChunkedGrid, x0: i32, y0: i32, x1: i32, y1: i32) -> bool {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut cx = x0;
    let mut cy = y0;
    loop {
        if cx == x1 && cy == y1 {
            return true;
        }
        if cx != x0 || cy != y0 {
            if grid.get(cx, cy).is_solid() {
                return false;
            }
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            cx += sx;
        }
        if e2 < dx {
            err += dx;
            cy += sy;
        }
    }
}

const NEIGHBORS4: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

pub fn random_seed() -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    (nanos ^ (nanos >> 32)) as u64
}
