use crate::world::cell::{Cell, MaterialId};
use crate::world::grid::Grid;
use crate::world::material::MaterialRegistry;

pub struct CellularAutomaton {
    tick: u64,
    rng_state: u64,
}

impl CellularAutomaton {
    pub fn new() -> Self {
        Self {
            tick: 0,
            rng_state: 0x1234567890ABCDEF,
        }
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

    pub fn step(&mut self, grid: &mut Grid) {
        grid.reset_tick_flags();
        let flip = self.rand_bool();
        let h = grid.height;
        let w = grid.width;

        for y_idx in (0..h).rev() {
            let y = y_idx as i32;
            let xs: Vec<i32> = if flip {
                (0..w as i32).collect()
            } else {
                (0..w as i32).rev().collect()
            };

            for x in xs {
                let cell = grid.get(x, y);
                if cell.updated_this_tick || cell.is_empty() || cell.is_static() {
                    continue;
                }
                match cell.material {
                    MaterialId::Sand => self.update_sand(grid, x, y),
                    MaterialId::Water => self.update_water(grid, x, y),
                    MaterialId::Lava => self.update_lava(grid, x, y),
                    MaterialId::Steam => self.update_steam(grid, x, y),
                    MaterialId::Fire => self.update_fire(grid, x, y),
                    MaterialId::Smoke => self.update_smoke(grid, x, y),
                    MaterialId::Acid => self.update_acid(grid, x, y),
                    MaterialId::Flesh => self.update_flesh(grid, x, y),
                    MaterialId::Grass => self.update_grass(grid, x, y),
                    MaterialId::Dirt => self.update_dirt(grid, x, y),
                    _ => {}
                }
            }
        }

        self.heat_transfer(grid);
        self.tick += 1;
    }

    fn try_move_down(&mut self, grid: &mut Grid, x: i32, y: i32, _mat: MaterialId, density: f32) {
        let below = grid.get(x, y + 1);
        if below.is_empty() || (below.is_liquid() && below.density() < density) {
            let src = grid.get(x, y);
            let i_dst = grid.idx(x, y + 1);
            let i_src = grid.idx(x, y);
            grid.cells[i_dst] = src;
            grid.cells[i_dst].updated_this_tick = true;
            grid.cells[i_src] = Cell::empty();
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
                self.do_swap(grid, x, y, x - dir, y + 1);
            } else {
                self.do_swap(grid, x, y, x + dir, y + 1);
            }
        } else if can_left {
            self.do_swap(grid, x, y, x - dir, y + 1);
        } else if can_right {
            self.do_swap(grid, x, y, x + dir, y + 1);
        }
    }

    #[inline]
    fn do_swap(&self, grid: &mut Grid, x1: i32, y1: i32, x2: i32, y2: i32) {
        let a = grid.get(x1, y1);
        let b = grid.get(x2, y2);
        let i1 = grid.idx(x1, y1);
        let i2 = grid.idx(x2, y2);
        grid.cells[i1] = b;
        grid.cells[i2] = a;
        grid.cells[i2].updated_this_tick = true;
    }

    fn update_sand(&mut self, grid: &mut Grid, x: i32, y: i32) {
        self.try_move_down(grid, x, y, MaterialId::Sand, 1.5);
    }

    fn update_water(&mut self, grid: &mut Grid, x: i32, y: i32) {
        let below = grid.get(x, y + 1);
        if below.is_empty() || (below.is_liquid() && below.density() < 1.0) {
            self.do_swap(grid, x, y, x, y + 1);
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
                self.do_swap(grid, x, y, x - dir, y + 1);
            } else {
                self.do_swap(grid, x, y, x + dir, y + 1);
            }
        } else if can_dl {
            self.do_swap(grid, x, y, x - dir, y + 1);
        } else if can_dr {
            self.do_swap(grid, x, y, x + dir, y + 1);
        } else {
            let can_l = grid.in_bounds(x - dir, y) && grid.get(x - dir, y).is_empty();
            let can_r = grid.in_bounds(x + dir, y) && grid.get(x + dir, y).is_empty();
            if can_l && can_r {
                if self.rand_bool() {
                    self.do_swap(grid, x, y, x - dir, y);
                } else {
                    self.do_swap(grid, x, y, x + dir, y);
                }
            } else if can_l {
                self.do_swap(grid, x, y, x - dir, y);
            } else if can_r {
                self.do_swap(grid, x, y, x + dir, y);
            }
        }

        let cell = grid.get(x, y);
        if cell.temp > 100.0 {
            let mut new = cell;
            new.material = MaterialId::Steam;
            new.temp = 110.0;
            let i = grid.idx(x, y);
            grid.cells[i] = new;
        }
    }

    fn update_lava(&mut self, grid: &mut Grid, x: i32, y: i32) {
        let cell = grid.get(x, y);
        if cell.temp < 400.0 {
            let mut new = cell;
            new.material = MaterialId::Stone;
            let i = grid.idx(x, y);
            grid.cells[i] = new;
            return;
        }

        let below = grid.get(x, y + 1);
        if below.is_empty() {
            self.do_swap(grid, x, y, x, y + 1);
            return;
        }

        let dir = if self.rand_bool() { 1 } else { -1 };
        if grid.in_bounds(x - dir, y + 1) && grid.get(x - dir, y + 1).is_empty() {
            self.do_swap(grid, x, y, x - dir, y + 1);
            return;
        }
        if grid.in_bounds(x + dir, y + 1) && grid.get(x + dir, y + 1).is_empty() {
            self.do_swap(grid, x, y, x + dir, y + 1);
            return;
        }

        if self.rand() % 10 == 0 {
            if grid.in_bounds(x - dir, y) && grid.get(x - dir, y).is_empty() {
                self.do_swap(grid, x, y, x - dir, y);
            } else if grid.in_bounds(x + dir, y) && grid.get(x + dir, y).is_empty() {
                self.do_swap(grid, x, y, x + dir, y);
            }
        }

        self.lava_interact(grid, x, y);
    }

    fn lava_interact(&mut self, grid: &mut Grid, x: i32, y: i32) {
        for &(dx, dy) in &NEIGHBORS4 {
            let nx = x + dx;
            let ny = y + dy;
            if !grid.in_bounds(nx, ny) {
                continue;
            }
            let neighbor = grid.get(nx, ny);
            match neighbor.material {
                MaterialId::Water => {
                    let i_n = grid.idx(nx, ny);
                    grid.cells[i_n] = Cell::new(MaterialId::Steam);
                    let lava = grid.get(x, y);
                    let mut new_lava = lava;
                    new_lava.temp -= 50.0;
                    let i_l = grid.idx(x, y);
                    grid.cells[i_l] = new_lava;
                }
                MaterialId::Wood | MaterialId::Grass | MaterialId::Flesh if neighbor.temp < 300.0 => {
                    let i_n = grid.idx(nx, ny);
                    grid.cells[i_n] = Cell::new(MaterialId::Fire);
                }
                MaterialId::Sand if neighbor.temp > 1700.0 => {
                    let i_n = grid.idx(nx, ny);
                    grid.cells[i_n] = Cell::new(MaterialId::Stone);
                }
                _ => {}
            }
        }
    }

    fn update_steam(&mut self, grid: &mut Grid, x: i32, y: i32) {
        let cell = grid.get(x, y);
        if cell.temp < 80.0 {
            let mut new = cell;
            new.material = MaterialId::Water;
            new.temp = 50.0;
            let i = grid.idx(x, y);
            grid.cells[i] = new;
            return;
        }

        if y > 0 && grid.get(x, y - 1).is_empty() {
            self.do_swap(grid, x, y, x, y - 1);
            return;
        }

        let dir = if self.rand_bool() { 1 } else { -1 };
        if grid.in_bounds(x - dir, y - 1) && grid.get(x - dir, y - 1).is_empty() {
            self.do_swap(grid, x, y, x - dir, y - 1);
            return;
        }
        if grid.in_bounds(x + dir, y - 1) && grid.get(x + dir, y - 1).is_empty() {
            self.do_swap(grid, x, y, x + dir, y - 1);
            return;
        }

        if self.rand() % 3 == 0 {
            if grid.in_bounds(x - dir, y) && grid.get(x - dir, y).is_empty() {
                self.do_swap(grid, x, y, x - dir, y);
            } else if grid.in_bounds(x + dir, y) && grid.get(x + dir, y).is_empty() {
                self.do_swap(grid, x, y, x + dir, y);
            }
        }
    }

    fn update_fire(&mut self, grid: &mut Grid, x: i32, y: i32) {
        let cell = grid.get(x, y);
        if cell.temp < 100.0 || self.rand() % 20 == 0 {
            let i = grid.idx(x, y);
            if self.rand() % 3 == 0 {
                grid.cells[i] = Cell::new(MaterialId::Smoke);
            } else {
                grid.cells[i] = Cell::empty();
            }
            return;
        }

        let mut new = cell;
        new.temp -= 15.0;
        let i = grid.idx(x, y);
        grid.cells[i] = new;

        if y > 0 && grid.get(x, y - 1).is_empty() && self.rand() % 2 == 0 {
            self.do_swap(grid, x, y, x, y - 1);
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
            if mat.flammable && neighbor.temp < mat.ignition_temp {
                let mut new_n = neighbor;
            new_n.material = MaterialId::Fire;
            new_n.temp = 400.0;
            let i_n = grid.idx(nx, ny);
            grid.cells[i_n] = new_n;
            }
        }
    }

    fn update_smoke(&mut self, grid: &mut Grid, x: i32, y: i32) {
        if self.rand() % 60 == 0 {
            let i = grid.idx(x, y);
            grid.cells[i] = Cell::empty();
            return;
        }

        if y > 0 && grid.get(x, y - 1).is_empty() {
            self.do_swap(grid, x, y, x, y - 1);
            return;
        }

        let dir = if self.rand_bool() { 1 } else { -1 };
        if grid.in_bounds(x - dir, y - 1) && grid.get(x - dir, y - 1).is_empty() {
            self.do_swap(grid, x, y, x - dir, y - 1);
        } else if grid.in_bounds(x + dir, y - 1) && grid.get(x + dir, y - 1).is_empty() {
            self.do_swap(grid, x, y, x + dir, y - 1);
        } else if grid.in_bounds(x - dir, y) && grid.get(x - dir, y).is_empty() {
            self.do_swap(grid, x, y, x - dir, y);
        } else if grid.in_bounds(x + dir, y) && grid.get(x + dir, y).is_empty() {
            self.do_swap(grid, x, y, x + dir, y);
        }
    }

    fn update_acid(&mut self, grid: &mut Grid, x: i32, y: i32) {
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
                let i_n = grid.idx(nx, ny);
                grid.cells[i_n] = Cell::empty();
                if self.rand() % 2 == 0 {
                    let i = grid.idx(x, y);
                    grid.cells[i] = Cell::empty();
                    return;
                }
            }
        }

        let below = grid.get(x, y + 1);
        if below.is_empty() || (below.is_liquid() && below.density() < 1.2) {
            self.do_swap(grid, x, y, x, y + 1);
            return;
        }

        let dir = if self.rand_bool() { 1 } else { -1 };
        if grid.in_bounds(x - dir, y + 1) && grid.get(x - dir, y + 1).is_empty() {
            self.do_swap(grid, x, y, x - dir, y + 1);
        } else if grid.in_bounds(x + dir, y + 1) && grid.get(x + dir, y + 1).is_empty() {
            self.do_swap(grid, x, y, x + dir, y + 1);
        } else if grid.in_bounds(x - dir, y) && grid.get(x - dir, y).is_empty() {
            self.do_swap(grid, x, y, x - dir, y);
        } else if grid.in_bounds(x + dir, y) && grid.get(x + dir, y).is_empty() {
            self.do_swap(grid, x, y, x + dir, y);
        }
    }

    fn update_flesh(&mut self, grid: &mut Grid, x: i32, y: i32) {
        let cell = grid.get(x, y);
        if cell.temp > 200.0 {
            let mut new = cell;
            new.material = MaterialId::Fire;
            new.temp = 400.0;
            let i = grid.idx(x, y);
            grid.cells[i] = new;
        }
    }

    fn update_grass(&mut self, grid: &mut Grid, x: i32, y: i32) {
        let cell = grid.get(x, y);
        if cell.temp > 250.0 {
            let i = grid.idx(x, y);
            grid.cells[i] = Cell::new(MaterialId::Fire);
        }
    }

    fn update_dirt(&mut self, grid: &mut Grid, x: i32, y: i32) {
        let cell = grid.get(x, y);
        if cell.temp < 0.0 {
            let mut new = cell;
            new.material = MaterialId::Stone;
            let i = grid.idx(x, y);
            grid.cells[i] = new;
        }
    }

    fn heat_transfer(&mut self, grid: &mut Grid) {
        let w = grid.width;
        let h = grid.height;
        let temps: Vec<f32> = grid.cells.iter().map(|c| c.temp).collect();

        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let cell = grid.cells[i];
                if cell.is_empty() || cell.is_static() {
                    continue;
                }
                let reg = crate::world::material::MaterialRegistry::instance();
                let mat = reg.get(cell.material);
                let k = mat.heat_conductivity;
                if k == 0.0 {
                    continue;
                }

                let mut sum = 0.0;
                let mut count = 0;
                for &(dx, dy) in &NEIGHBORS4 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || nx >= w as i32 || ny < 0 || ny >= h as i32 {
                        continue;
                    }
                    let ni = ny as usize * w + nx as usize;
                    sum += temps[ni];
                    count += 1;
                }
                if count > 0 {
                    let avg = sum / count as f32;
                    let mut new = cell;
                    new.temp += (avg - cell.temp) * k * 0.1;
                    grid.cells[i] = new;
                }
            }
        }
    }

    pub fn tick_count(&self) -> u64 {
        self.tick
    }
}

const NEIGHBORS4: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
