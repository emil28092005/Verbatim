use crate::world::cell::{default_temp, Cell, MaterialId};
use crate::world::chunk::CHUNK_SIZE;
use crate::world::chunked_grid::ChunkedGrid;

pub struct CellularAutomaton {
    tick: u64,
    rng_state: u64,
    temps_buf: Vec<f32>,
    light_tick: u64,
}

impl CellularAutomaton {
    pub fn new() -> Self {
        Self {
            tick: 0,
            rng_state: 0x1234567890ABCDEF,
            temps_buf: Vec::new(),
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

        let mut pre_dirty: Vec<((i32, i32), (i32, i32, i32, i32))> = Vec::new();
        for (cx, cy) in &active {
            if let Some(d) = grid.get_chunk_dirty(*cx, *cy) {
                pre_dirty.push(((*cx, *cy), d));
            }
        }

        for (cx, cy) in &active {
            let dirty = grid.get_chunk_dirty(*cx, *cy);
            if dirty.is_none() {
                continue;
            }
            let (min_x, min_y, max_x, max_y) = dirty.unwrap();

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let mut cell = grid.get(x, y);
                    if cell.updated_this_tick {
                        cell.updated_this_tick = false;
                        grid.set(x, y, cell);
                    }
                }
            }
        }

        for (cx, cy) in active {
            let dirty = grid.get_chunk_dirty(cx, cy);
            if dirty.is_none() {
                continue;
            }
            let (min_x, min_y, max_x, max_y) = dirty.unwrap();
            let dw = max_x - min_x + 1;
            let dh = max_y - min_y + 1;
            if dw * dh > 4096 {
                grid.set_chunk_dirty(cx, cy, None);
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

        self.heat_transfer(grid, &pre_dirty);
        self.gas_step(grid, &pre_dirty);
        self.pressure_step(grid, &pre_dirty);
        self.light_step(grid);
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
                    grid.set(nx, ny, Cell::new(MaterialId::Fire));
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
        let cell = grid.get(x, y);
        let temp = grid.get_temp(x, y);

        let (egt, egd) = grid.get_gas(x, y);
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
            grid.set(x, y, new);
            grid.set_temp(x, y, 400.0);
        }
    }

    fn update_grass(&mut self, grid: &mut ChunkedGrid, x: i32, y: i32) {
        let temp = grid.get_temp(x, y);
        if temp > 250.0 {
            grid.set(x, y, Cell::new(MaterialId::Fire));
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
        pre_dirty: &[((i32, i32), (i32, i32, i32, i32))],
    ) {
        let reg = crate::world::material::MaterialRegistry::instance();
        let active = grid.active_chunks();
        for (cx, cy) in active {
            let dirty = grid.get_chunk_dirty(cx, cy);
            let pre = pre_dirty
                .iter()
                .find(|(c, _)| *c == (cx, cy))
                .map(|(_, d)| *d);
            let (min_x, min_y, max_x, max_y) = match (dirty, pre) {
                (Some(d), Some(p)) => (d.0.min(p.0), d.1.min(p.1), d.2.max(p.2), d.3.max(p.3)),
                (Some(d), None) => d,
                (None, Some(p)) => p,
                (None, None) => continue,
            };
            if (max_x - min_x + 1) * (max_y - min_y + 1) > 4096 {
                continue;
            }
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let cell = grid.get(x, y);
                    if cell.is_empty() {
                        continue;
                    }
                    let mat = reg.get(cell.material);
                    let k = mat.heat_conductivity;
                    if k == 0.0 {
                        continue;
                    }
                    let cur_temp = grid.get_temp(x, y);
                    let mut sum = 0.0;
                    let mut count = 0;
                    for &(dx, dy) in &NEIGHBORS4 {
                        let nx = x + dx;
                        let ny = y + dy;
                        sum += grid.get_temp(nx, ny);
                        count += 1;
                    }
                    if count > 0 {
                        let avg = sum / count as f32;
                        let new_temp = cur_temp + (avg - cur_temp) * k * 0.1;
                        if (new_temp - cur_temp).abs() > 0.01 {
                            grid.set_temp(x, y, new_temp);
                            grid.mark_dirty(x, y);
                        }
                    }
                }
            }
        }
    }

    fn gas_step(
        &mut self,
        grid: &mut ChunkedGrid,
        pre_dirty: &[((i32, i32), (i32, i32, i32, i32))],
    ) {
        let active = grid.active_chunks();
        for (cx, cy) in active {
            let dirty = grid.get_chunk_dirty(cx, cy);
            let pre = pre_dirty
                .iter()
                .find(|(c, _)| *c == (cx, cy))
                .map(|(_, d)| *d);
            let (min_x, min_y, max_x, max_y) = match (dirty, pre) {
                (Some(d), Some(p)) => (d.0.min(p.0), d.1.min(p.1), d.2.max(p.2), d.3.max(p.3)),
                (Some(d), None) => d,
                (None, Some(p)) => p,
                (None, None) => continue,
            };
            if (max_x - min_x + 1) * (max_y - min_y + 1) > 4096 {
                continue;
            }
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let (gt, gd) = grid.get_gas(x, y);
                    if gd == 0 {
                        continue;
                    }

                    if gt == 4 && grid.get_temp(x, y) < 80.0 {
                        let mut new = grid.get(x, y);
                        new.material = MaterialId::Water;
                        grid.set(x, y, new);
                        grid.set_temp(x, y, 50.0);
                        grid.set_gas(x, y, 0, 0);
                        continue;
                    }

                    if y > 0 {
                        let above = grid.get(x, y - 1);
                        if above.is_empty() || above.is_gas() {
                            let (agt, agd) = grid.get_gas(x, y - 1);
                            if agd < gd {
                                grid.set_gas(x, y - 1, gt, gd);
                                grid.set_gas(x, y, agt, agd);
                                continue;
                            }
                        }
                    }

                    let dir = if self.rand_bool() { 1 } else { -1 };
                    for &d in &[dir, -dir] {
                        if grid.in_bounds(x + d, y) {
                            let side = grid.get(x + d, y);
                            if side.is_empty() || side.is_gas() {
                                let (sgt, sgd) = grid.get_gas(x + d, y);
                                if sgd < gd.saturating_sub(5) {
                                    let avg = (gd + sgd) / 2;
                                    grid.set_gas(x + d, y, gt, avg);
                                    grid.set_gas(x, y, gt, gd.saturating_sub(avg - sgd));
                                    break;
                                }
                            }
                        }
                    }

                    if (gt == 1 || gt == 2) && gd > 0 && self.rand() % 120 == 0 {
                        grid.set_gas(x, y, gt, gd.saturating_sub(1));
                    }
                }
            }
        }
    }

    fn pressure_step(
        &mut self,
        grid: &mut ChunkedGrid,
        pre_dirty: &[((i32, i32), (i32, i32, i32, i32))],
    ) {
        let active = grid.active_chunks();
        for (cx, cy) in active {
            let dirty = grid.get_chunk_dirty(cx, cy);
            let pre = pre_dirty
                .iter()
                .find(|(c, _)| *c == (cx, cy))
                .map(|(_, d)| *d);
            let (min_x, min_y, max_x, max_y) = match (dirty, pre) {
                (Some(d), Some(p)) => (d.0.min(p.0), d.1.min(p.1), d.2.max(p.2), d.3.max(p.3)),
                (Some(d), None) => d,
                (None, Some(p)) => p,
                (None, None) => continue,
            };
            if (max_x - min_x + 1) * (max_y - min_y + 1) > 4096 {
                continue;
            }
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let cell = grid.get(x, y);
                    if cell.is_solid() {
                        continue;
                    }
                    let cur_p = grid.get_pressure(x, y) as i32;
                    let mut sum = 0i32;
                    let mut count = 0i32;
                    for &(dx, dy) in &NEIGHBORS4 {
                        let nx = x + dx;
                        let ny = y + dy;
                        if !grid.in_bounds(nx, ny) {
                            continue;
                        }
                        let n = grid.get(nx, ny);
                        if n.is_solid() {
                            continue;
                        }
                        sum += grid.get_pressure(nx, ny) as i32;
                        count += 1;
                    }
                    if count > 0 {
                        let avg = sum / count;
                        let new_p = cur_p + (avg - cur_p) / 8;
                        if new_p != cur_p {
                            grid.set_pressure(x, y, new_p.clamp(0, 255) as u8);
                        }
                    }
                }
            }
        }
    }

    fn light_step(&mut self, grid: &mut ChunkedGrid) {
        self.light_tick += 1;
        if self.light_tick % 10 != 0 {
            return;
        }

        let active = grid.active_chunks();
        let cs = CHUNK_SIZE as i32;

        for &(cx, cy) in &active {
            if let Some(chunk) = grid.get_chunk_mut(cx, cy) {
                for l in chunk.light.iter_mut() {
                    *l = [0, 0, 0];
                }
            }
        }

        for &(cx, cy) in &active {
            let ox = cx * cs;
            let oy = cy * cs;
            for ly in 0..cs {
                for lx in 0..cs {
                    let wx = ox + lx;
                    let wy = oy + ly;
                    let cell = grid.get(wx, wy);
                    if let Some(src) = material_light(cell.material) {
                        let radius = src.0 as i32;
                        let color = src.1;
                        for dy in -radius..=radius {
                            for dx in -radius..=radius {
                                let tx = wx + dx;
                                let ty = wy + dy;
                                if !grid.in_bounds(tx, ty) {
                                    continue;
                                }
                                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                                if dist > radius as f32 {
                                    continue;
                                }
                                if !line_of_sight(grid, wx, wy, tx, ty) {
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
        }
    }
}

fn material_light(mat: MaterialId) -> Option<(u32, [u8; 3])> {
    match mat {
        MaterialId::Lava => Some((25, [255, 120, 30])),
        MaterialId::Fire => Some((15, [255, 180, 60])),
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
