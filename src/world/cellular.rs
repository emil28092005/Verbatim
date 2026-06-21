use crate::world::cell::{Cell, MaterialId};
use crate::world::chunked_grid::ChunkedGrid;

pub struct CellularAutomaton {
    tick: u64,
    rng_state: u64,
    temps: Vec<f32>,
}

impl CellularAutomaton {
    pub fn new() -> Self {
        Self {
            tick: 0,
            rng_state: 0x1234567890ABCDEF,
            temps: Vec::new(),
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

        for (cx, cy) in active {
            let dirty = grid.get_chunk_dirty(cx, cy);
            if dirty.is_none() {
                continue;
            }
            let (min_x, min_y, max_x, max_y) = dirty.unwrap();

            grid.set_chunk_dirty(cx, cy, None);

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let mut cell = grid.get(x, y);
                    cell.updated_this_tick = false;
                    grid.set(x, y, cell);
                }
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

        self.heat_transfer(grid);
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
            return;
        }

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

        let cell = grid.get(x, y);
        if cell.temp > 100.0 {
            let mut new = cell;
            new.material = MaterialId::Steam;
            new.temp = 110.0;
            grid.set(x, y, new);
        }
    }

    fn update_lava(&mut self, grid: &mut ChunkedGrid, x: i32, y: i32) {
        let cell = grid.get(x, y);
        if cell.temp < 400.0 {
            let mut new = cell;
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
            match neighbor.material {
                MaterialId::Water => {
                    grid.set(nx, ny, Cell::new(MaterialId::Steam));
                    let lava = grid.get(x, y);
                    let mut new_lava = lava;
                    new_lava.temp -= 50.0;
                    grid.set(x, y, new_lava);
                }
                MaterialId::Wood | MaterialId::Grass | MaterialId::Flesh
                    if neighbor.temp < 300.0 =>
                {
                    grid.set(nx, ny, Cell::new(MaterialId::Fire));
                }
                MaterialId::Sand if neighbor.temp > 1700.0 => {
                    grid.set(nx, ny, Cell::new(MaterialId::Stone));
                }
                _ => {}
            }
        }
    }

    fn update_steam(&mut self, grid: &mut ChunkedGrid, x: i32, y: i32) {
        let cell = grid.get(x, y);
        if cell.temp < 80.0 {
            let mut new = cell;
            new.material = MaterialId::Water;
            new.temp = 50.0;
            grid.set(x, y, new);
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

        for &(dx, dy) in &NEIGHBORS4 {
            let nx = x + dx;
            let ny = y + dy;
            if !grid.in_bounds(nx, ny) {
                continue;
            }
            let neighbor = grid.get(nx, ny);
            let reg = crate::world::material::MaterialRegistry::instance();
            let mat = reg.get(neighbor.material);
            if mat.flammable && neighbor.temp < mat.ignition_temp {
                let mut new_n = neighbor;
                new_n.material = MaterialId::Fire;
                new_n.temp = 400.0;
                grid.set(nx, ny, new_n);
            }
        }

        if cell.temp < 100.0 || self.rand() % 20 == 0 {
            if self.rand() % 3 == 0 {
                grid.set(x, y, Cell::new(MaterialId::Smoke));
            } else {
                grid.set(x, y, Cell::empty());
            }
            return;
        }

        let mut new = cell;
        new.temp -= 15.0;
        grid.set(x, y, new);

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
        let cell = grid.get(x, y);
        if cell.temp > 200.0 {
            let mut new = cell;
            new.material = MaterialId::Fire;
            new.temp = 400.0;
            grid.set(x, y, new);
        }
    }

    fn update_grass(&mut self, grid: &mut ChunkedGrid, x: i32, y: i32) {
        let cell = grid.get(x, y);
        if cell.temp > 250.0 {
            grid.set(x, y, Cell::new(MaterialId::Fire));
        }
    }

    fn update_dirt(&mut self, grid: &mut ChunkedGrid, x: i32, y: i32) {
        let cell = grid.get(x, y);
        if cell.temp < 0.0 {
            let mut new = cell;
            new.material = MaterialId::Stone;
            grid.set(x, y, new);
        }
    }

    fn heat_transfer(&mut self, grid: &mut ChunkedGrid) {
        let reg = crate::world::material::MaterialRegistry::instance();
        let gw = grid.width as i32;
        let gh = grid.height as i32;

        let active = grid.active_chunks();
        for (cx, cy) in active {
            let dirty = grid.get_chunk_dirty(cx, cy);
            if dirty.is_none() {
                continue;
            }
            let (min_x, min_y, max_x, max_y) = dirty.unwrap();

            let ex_min_x = (min_x - 1).max(0);
            let ex_min_y = (min_y - 1).max(0);
            let (ex_max_x, ex_max_y) = if grid.is_infinite() {
                ((max_x + 1), (max_y + 1))
            } else {
                ((max_x + 1).min(gw - 1), (max_y + 1).min(gh - 1))
            };

            let ew = (ex_max_x - ex_min_x + 1) as usize;
            let eh = (ex_max_y - ex_min_y + 1) as usize;
            let ecount = ew.saturating_mul(eh);
            if ecount > 10000 {
                eprintln!(
                    "heat_transfer skipping huge dirty rect: chunk=({}, {}) dirty=({},{},{},{}) ew={} eh={}",
                    cx, cy, min_x, min_y, max_x, max_y, ew, eh
                );
                continue;
            }
            if self.temps.len() < ecount {
                self.temps.resize(ecount, 0.0);
            }

            for y in ex_min_y..=ex_max_y {
                for x in ex_min_x..=ex_max_x {
                    let idx = ((y - ex_min_y) as usize) * ew + (x - ex_min_x) as usize;
                    self.temps[idx] = grid.get(x, y).temp;
                }
            }

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let cell = grid.get(x, y);
                    if cell.is_empty() || cell.is_static() {
                        continue;
                    }
                    let mat = reg.get(cell.material);
                    let k = mat.heat_conductivity;
                    if k == 0.0 {
                        continue;
                    }

                    let mut sum = 0.0;
                    let mut count = 0;
                    for &(dx, dy) in &NEIGHBORS4 {
                        let nx = x + dx;
                        let ny = y + dy;
                        if nx < ex_min_x || nx > ex_max_x || ny < ex_min_y || ny > ex_max_y {
                            continue;
                        }
                        let ni = ((ny - ex_min_y) as usize) * ew + (nx - ex_min_x) as usize;
                        sum += self.temps[ni];
                        count += 1;
                    }
                    if count > 0 {
                        let avg = sum / count as f32;
                        let mut new = cell;
                        new.temp += (avg - cell.temp) * k * 0.1;
                        grid.set(x, y, new);
                        grid.mark_dirty(x, y);
                    }
                }
            }
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
