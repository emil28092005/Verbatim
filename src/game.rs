use std::time::{Duration, Instant};

use crate::entity::{EntityManager, EntityKind};
use crate::input::{Action, InputHandler};
use crate::physics::verlet::VerletSolver;
use crate::physics::collision::resolve_grid_collision;
use crate::render::Renderer;
use crate::world::cell::MaterialId;
use crate::world::grid::Grid;
use crate::world::cellular::CellularAutomaton;
use crate::entity::player::Player;

pub struct Game {
    pub grid: Grid,
    pub ca: CellularAutomaton,
    pub verlet: VerletSolver,
    pub entities: EntityManager,
    pub player: Player,
    pub input: InputHandler,
    pub cam_x: i32,
    pub cam_y: i32,
    pub running: bool,
    pub tick: u64,
    pub fixed_dt: Duration,
    pub accumulator: Duration,
    pub last_time: Instant,
}

impl Game {
    pub fn new() -> Self {
        let mut entities = EntityManager::new();
        let player = Player::new(&mut entities);
        Self {
            grid: Grid::new(),
            ca: CellularAutomaton::new(),
            verlet: VerletSolver::new(),
            entities,
            player,
            input: InputHandler::new(),
            cam_x: 100,
            cam_y: 100,
            running: true,
            tick: 0,
            fixed_dt: Duration::from_millis(16),
            accumulator: Duration::ZERO,
            last_time: Instant::now(),
        }
    }

    pub fn init_world(&mut self) {
        let w = self.grid.width;
        let h = self.grid.height;

        for x in 0..w {
            self.grid.set_material(x as i32, (h - 1) as i32, MaterialId::Stone);
            self.grid.set_material(x as i32, (h - 2) as i32, MaterialId::Dirt);
        }

        for x in 0..w {
            let surface = (h as i32 - 3) - ((x as f32 * 0.1).sin() * 5.0) as i32;
            let surface = surface.max(10).min(h as i32 - 3);
            for y in surface..(h as i32 - 2) {
                if y == surface {
                    self.grid.set_material(x as i32, y, MaterialId::Grass);
                } else {
                    self.grid.set_material(x as i32, y, MaterialId::Dirt);
                }
            }
        }

        // Water pool (left side)
        let water_x = 40;
        let water_surface = (h as i32 - 3) - ((water_x as f32 * 0.1).sin() * 5.0) as i32;
        let _water_surface = water_surface.max(10).min(h as i32 - 3);
        for x in water_x - 12..=water_x + 12 {
            let s = (h as i32 - 3) - ((x as f32 * 0.1).sin() * 5.0) as i32;
            let s = s.max(10).min(h as i32 - 3);
            for y in s - 8..s {
                if self.grid.get(x as i32, y).is_empty() {
                    self.grid.set_material(x as i32, y, MaterialId::Water);
                }
            }
        }

        // Lava pool (right side)
        let lava_x = 200;
        for x in lava_x - 10..=lava_x + 10 {
            let s = (h as i32 - 3) - ((x as f32 * 0.1).sin() * 5.0) as i32;
            let s = s.max(10).min(h as i32 - 3);
            for y in s - 5..s {
                if self.grid.get(x as i32, y).is_empty() {
                    self.grid.set_material(x as i32, y, MaterialId::Lava);
                }
            }
        }

        // Wood structure near center-left
        let wood_x = 90;
        let wood_surface = (h as i32 - 3) - ((wood_x as f32 * 0.1).sin() * 5.0) as i32;
        let wood_surface = wood_surface.max(10).min(h as i32 - 3);
        for y in wood_surface - 8..wood_surface {
            self.grid.set_material(wood_x, y, MaterialId::Wood);
            self.grid.set_material(wood_x + 4, y, MaterialId::Wood);
        }
        for x in wood_x..=wood_x + 4 {
            self.grid.set_material(x, wood_surface - 8, MaterialId::Wood);
        }

        // Sand dune (right of center)
        let sand_x = 160;
        let sand_surface = (h as i32 - 3) - ((sand_x as f32 * 0.1).sin() * 5.0) as i32;
        let sand_surface = sand_surface.max(10).min(h as i32 - 3);
        for dx in -8..=8 {
            let pile_h = (8.0 - (dx as f32).abs()) as i32;
            for dy in 0..pile_h {
                let y = sand_surface - 1 - dy;
                if self.grid.get(sand_x + dx, y).is_empty() {
                    self.grid.set_material(sand_x + dx, y, MaterialId::Sand);
                }
            }
        }

        // Acid pool (far left)
        let acid_x = 15;
        for x in acid_x - 5..=acid_x + 5 {
            let s = (h as i32 - 3) - ((x as f32 * 0.1).sin() * 5.0) as i32;
            let s = s.max(10).min(h as i32 - 3);
            for y in s - 4..s {
                if self.grid.get(x as i32, y).is_empty() {
                    self.grid.set_material(x as i32, y, MaterialId::Acid);
                }
            }
        }

        // Stone wall obstacle (between player and water)
        let wall_x = 110;
        let wall_surface = (h as i32 - 3) - ((wall_x as f32 * 0.1).sin() * 5.0) as i32;
        let wall_surface = wall_surface.max(10).min(h as i32 - 3);
        for y in wall_surface - 6..wall_surface {
            self.grid.set_material(wall_x, y, MaterialId::Stone);
            self.grid.set_material(wall_x + 1, y, MaterialId::Stone);
        }

        self.grid.fill_border(MaterialId::Stone);

        let cx = (w / 2) as f32;
        let surface_x = cx as i32;
        let mut surface_y = h as i32 - 3;
        for y in 0..h as i32 {
            if self.grid.get(surface_x, y).is_solid() && self.grid.get(surface_x, y).material != MaterialId::Stone {
                surface_y = y;
                break;
            }
        }
        let cy = (surface_y as f32) - 5.0;
        self.player.spawn_at(&mut self.entities, cx, cy);

        let (px, py) = self.player.center(&self.entities);
        self.center_camera_on(px, py);
    }

    pub fn center_camera_on(&mut self, px: f32, py: f32) {
        self.cam_x = px as i32 - 60;
        self.cam_y = py as i32 - 20;
    }

    pub fn run<R: Renderer>(&mut self, renderer: &mut R) {
        if let Err(e) = renderer.init() {
            eprintln!("Renderer init failed: {}", e);
            return;
        }
        self.init_world();

        self.last_time = Instant::now();

        while self.running {
            let now = Instant::now();
            let frame_time = now.duration_since(self.last_time);
            self.last_time = now;
            self.accumulator += frame_time;

            while self.accumulator >= self.fixed_dt {
                self.fixed_update();
                self.accumulator -= self.fixed_dt;
            }

            let vw = renderer.viewport_w();
            let vh = renderer.viewport_h();
            let (px, py) = self.player.center(&self.entities);
            self.cam_x = px as i32 - (vw as i32 / 2);
            self.cam_y = py as i32 - (vh as i32 / 2);

            if let Err(e) = renderer.render(&self.grid, &self.entities, self.cam_x, self.cam_y) {
                eprintln!("Render error: {}", e);
                break;
            }

            self.handle_input(vw, vh);

            std::thread::sleep(std::time::Duration::from_millis(16));
        }

        if let Err(e) = renderer.shutdown() {
            eprintln!("Renderer shutdown failed: {}", e);
        }
    }

    pub fn handle_input(&mut self, vw: usize, vh: usize) {
        let one_shots = self.input.update();

        for action in one_shots {
            match action {
                Action::Quit => {
                    self.running = false;
                    return;
                }
                Action::Paint(brush) => {
                    let mat = brush.to_material();
                    let cx = self.cam_x + (vw as i32 / 2);
                    let cy = self.cam_y + (vh as i32 / 2);
                    let r = 2;
                    for dy in -r..=r {
                        for dx in -r..=r {
                            if dx * dx + dy * dy <= r * r + 1 {
                                if let Some(m) = mat {
                                    self.grid.set_material(cx + dx, cy + dy, m);
                                } else {
                                    self.grid.set(cx + dx, cy + dy, crate::world::cell::Cell::empty());
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if !self.running {
            return;
        }

        // Jump: only on press, not held
        if self.input.jump_requested() {
            let on_ground = self.check_on_ground();
            self.player.jump(&mut self.entities, on_ground);
        }

        // Movement: applied every tick while held
        for action in self.input.held_actions() {
            match action {
                Action::MoveLeft => self.player.move_left(&mut self.entities),
                Action::MoveRight => self.player.move_right(&mut self.entities),
                Action::MoveCameraLeft => self.cam_x -= 2,
                Action::MoveCameraRight => self.cam_x += 2,
                Action::MoveCameraUp => self.cam_y -= 2,
                Action::MoveCameraDown => self.cam_y += 2,
                _ => {}
            }
        }
    }

    pub fn check_on_ground(&self) -> bool {
        if let Some(e) = self.player.entity(&self.entities) {
            let bottom_y = e.cy + e.half_h;
            let bottom_cell = bottom_y.floor() as i32;
            let frac = bottom_y - bottom_cell as f32;
            if frac > 0.05 {
                return false;
            }
            let left = (e.cx - e.half_w) as i32;
            let right = (e.cx + e.half_w) as i32;
            for x in left..=right {
                if self.grid.in_bounds(x, bottom_cell) && self.grid.get(x, bottom_cell).is_solid() {
                    return true;
                }
            }
        }
        false
    }

    pub fn fixed_update(&mut self) {
        self.tick += 1;

        self.ca.step(&mut self.grid);

        self.update_entities();

        self.apply_world_damage();

        if self.tick % 30 == 0 {
            self.try_spawn_goblin();
        }
    }

    fn update_entities(&mut self) {
        let solver = self.verlet.clone();
        let substeps = solver.substeps;
        let gravity = self.verlet.gravity;
        let damping = self.verlet.damping;
        let max_vel = solver.max_vel;

        let entity_count = self.entities.all().len();

        for idx in 0..entity_count {
            let is_rigid = self.entities.all()[idx].rigid;

            if is_rigid {
                self.update_rigid_entity(idx, gravity, damping, max_vel);
            } else {
                self.update_ragdoll_entity(idx, &solver, substeps);
            }

            if let Some(e) = self.entities.all_mut().get_mut(idx) {
                let mut total_health = 0.0;
                let mut alive_count = 0;
                for b in &e.bodies {
                    if b.alive {
                        total_health += b.health;
                        alive_count += 1;
                    }
                }
                if alive_count > 0 {
                    let avg = total_health / alive_count as f32;
                    if avg < 0.0 && e.alive {
                        e.kill();
                    }
                }

                let any_on_fire = e.bodies.iter().any(|b| b.alive && b.on_fire);
                e.on_fire = any_on_fire;
                if e.on_fire {
                    e.apply_fire_damage();
                }
            }
        }
    }

    fn update_rigid_entity(&mut self, idx: usize, gravity: f32, damping: f32, max_vel: f32) {
        let (cx, cy, cvx, cvy, half_w, half_h) = {
            let e = &self.entities.all()[idx];
            (e.cx, e.cy, e.cvx, e.cvy, e.half_w, e.half_h)
        };

        let mut nx = cx;
        let mut ny = cy;
        let mut nvx = cvx * damping;
        let mut nvy = cvy * damping;
        nvy += gravity;

        let v_mag = (nvx * nvx + nvy * nvy).sqrt();
        if v_mag > max_vel {
            nvx = nvx / v_mag * max_vel;
            nvy = nvy / v_mag * max_vel;
        }

        // Move X then resolve X collisions
        nx += nvx;
        let (resolved_x, hit_wall_x) = self.resolve_aabb_x(idx, nx, ny, half_w, half_h, nvx);
        nx = resolved_x;
        if hit_wall_x {
            if nvx > 0.0 { nvx = 0.0; }
            else if nvx < 0.0 { nvx = 0.0; }
        }

        // Move Y then resolve Y collisions
        ny += nvy;
        let (resolved_y, hit_floor, hit_ceiling) = self.resolve_aabb_y(idx, nx, ny, half_w, half_h, nvy > 0.0);
        ny = resolved_y;
        if hit_floor {
            nvy = 0.0;
        }
        if hit_ceiling {
            nvy = 0.0;
        }

        // Check material contacts
        let (touching_lava, touching_fire, touching_acid, in_liquid) = {
            let grid = &self.grid;
            let mut tl = false;
            let mut tf = false;
            let mut ta = false;
            let mut il = false;
            let min_x = (nx - half_w).floor() as i32;
            let max_x = (nx + half_w).ceil() as i32;
            let min_y = (ny - half_h).floor() as i32;
            let max_y = (ny + half_h).ceil() as i32;
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    if !grid.in_bounds(x, y) { continue; }
                    let cell = grid.get(x, y);
                    if cell.material == MaterialId::Lava { tl = true; }
                    if cell.material == MaterialId::Fire { tf = true; }
                    if cell.material == MaterialId::Acid { ta = true; }
                    if cell.is_liquid() { il = true; }
                }
            }
            (tl, tf, ta, il)
        };

        if let Some(e) = self.entities.all_mut().get_mut(idx) {
            e.cx = nx;
            e.cy = ny;
            e.cvx = nvx;
            e.cvy = nvy;
            if in_liquid {
                e.cvy *= 0.6;
                e.cvx *= 0.8;
            }
            e.sync_bodies_to_center();

            if touching_lava {
                for b in &mut e.bodies {
                    if b.alive {
                        b.health -= 0.5;
                        if !b.on_fire { b.on_fire = true; }
                    }
                }
            }
            if touching_fire {
                for b in &mut e.bodies {
                    if b.alive {
                        b.health -= 0.15;
                        if !b.on_fire && b.health < 80.0 { b.on_fire = true; }
                    }
                }
            }
            if touching_acid {
                for b in &mut e.bodies {
                    if b.alive { b.health -= 0.25; }
                }
            }
        }
    }

    fn resolve_aabb_x(&self, _idx: usize, cx: f32, cy: f32, hw: f32, hh: f32, vx: f32) -> (f32, bool) {
        let grid = &self.grid;
        let left = cx - hw;
        let right = cx + hw;
        let top = cy - hh;
        let bottom = cy + hh;

        let min_x = left.floor() as i32;
        let max_x = right.ceil() as i32;
        let min_y = top.floor() as i32;
        let max_y = bottom.ceil() as i32;

        let mut new_cx = cx;
        let mut hit = false;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if !grid.in_bounds(x, y) { continue; }
                let cell = grid.get(x, y);
                if !cell.is_solid() { continue; }

                let cell_left = x as f32;
                let cell_right = (x + 1) as f32;
                let cell_top = y as f32;
                let cell_bottom = (y + 1) as f32;

                if bottom <= cell_top || top >= cell_bottom {
                    continue;
                }

                if vx > 0.0 {
                    let pen = right - cell_left;
                    if pen > 0.0 && pen < 1.5 {
                        new_cx -= pen;
                        hit = true;
                    }
                } else if vx < 0.0 {
                    let pen = cell_right - left;
                    if pen > 0.0 && pen < 1.5 {
                        new_cx += pen;
                        hit = true;
                    }
                } else {
                    let pen_left = right - cell_left;
                    let pen_right = cell_right - left;
                    if pen_left < pen_right && pen_left > 0.0 && pen_left < 1.5 {
                        new_cx -= pen_left;
                        hit = true;
                    } else if pen_right > 0.0 && pen_right < 1.5 {
                        new_cx += pen_right;
                        hit = true;
                    }
                }
            }
        }

        (new_cx, hit)
    }

    fn resolve_aabb_y(&self, _idx: usize, cx: f32, cy: f32, hw: f32, hh: f32, moving_down: bool) -> (f32, bool, bool) {
        let grid = &self.grid;
        let left = cx - hw;
        let right = cx + hw;
        let top = cy - hh;
        let bottom = cy + hh;

        let min_x = left.floor() as i32;
        let max_x = right.ceil() as i32;
        let min_y = top.floor() as i32;
        let max_y = bottom.ceil() as i32;

        let mut max_pen = 0.0f32;
        let mut hit_floor = false;
        let mut hit_ceiling = false;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if !grid.in_bounds(x, y) { continue; }
                let cell = grid.get(x, y);
                if !cell.is_solid() { continue; }

                let cell_left = x as f32;
                let cell_right = (x + 1) as f32;
                let cell_top = y as f32;
                let cell_bottom = (y + 1) as f32;

                if right <= cell_left || left >= cell_right {
                    continue;
                }

                if bottom <= cell_top || top >= cell_bottom {
                    continue;
                }

                if moving_down {
                    let pen = bottom - cell_top;
                    if pen > max_pen {
                        max_pen = pen;
                        hit_floor = true;
                    }
                } else {
                    let pen = cell_bottom - top;
                    if pen > max_pen {
                        max_pen = pen;
                        hit_ceiling = true;
                    }
                }
            }
        }

        let new_cy = if hit_floor {
            cy - max_pen
        } else if hit_ceiling {
            cy + max_pen
        } else {
            cy
        };
        (new_cy, hit_floor, hit_ceiling)
    }

    fn update_ragdoll_entity(&mut self, idx: usize, solver: &crate::physics::verlet::VerletSolver, substeps: u32) {
        let grid = &self.grid;
        let mut bodies = self.entities.all()[idx].bodies.clone();
        let constraints = self.entities.all()[idx].constraints.clone();

        for b in &mut bodies {
            if !b.alive {
                continue;
            }
            if b.on_fire {
                b.fire_timer += 1;
                b.health -= 0.3;
                if b.fire_timer > 120 {
                    b.on_fire = false;
                    b.fire_timer = 0;
                }
            }
        }

        for _ in 0..substeps {
            solver.integrate(&mut bodies);

            for b in &mut bodies {
                if !b.alive {
                    continue;
                }
                let result = resolve_grid_collision(grid, b);
                if result.touching_lava {
                    b.health -= 0.5;
                    if !b.on_fire { b.on_fire = true; }
                }
                if result.touching_fire {
                    b.health -= 0.15;
                    if !b.on_fire && b.health < 80.0 { b.on_fire = true; }
                }
                if result.touching_acid {
                    b.health -= 0.25;
                }
            }

            for _ci in 0..4 {
                solver.solve_constraints(&mut bodies, &constraints, 1);
                for b in &mut bodies {
                    if !b.alive {
                        continue;
                    }
                    resolve_grid_collision(grid, b);
                }
            }
        }

        if let Some(e) = self.entities.all_mut().get_mut(idx) {
            e.bodies = bodies;
        }
    }

    fn apply_world_damage(&mut self) {
        let mut to_kill: Vec<usize> = Vec::new();
        for (i, e) in self.entities.all().iter().enumerate() {
            if !e.alive {
                continue;
            }
            let mut dead_parts = 0;
            for b in &e.bodies {
                if !b.alive || b.health <= 0.0 {
                    dead_parts += 1;
                }
            }
            if dead_parts == e.bodies.len() {
                to_kill.push(i);
            }
        }
        for i in to_kill {
            if let Some(e) = self.entities.all_mut().get_mut(i) {
                e.kill();
            }
        }
    }

    fn try_spawn_goblin(&mut self) {
        let alive_goblins = self.entities.all().iter().filter(|e| e.alive && e.kind == EntityKind::Goblin).count();
        if alive_goblins >= 3 {
            return;
        }

        let (px, _py) = self.player.center(&self.entities);
        let spawn_x = px as i32 + if px as i32 % 2 == 0 { 15 } else { -15 };
        if !self.grid.in_bounds(spawn_x, 0) {
            return;
        }

        let mut surface_y = self.grid.height as i32 - 3;
        for y in 0..self.grid.height as i32 {
            let cell = self.grid.get(spawn_x, y);
            if cell.is_solid() && cell.material != MaterialId::Stone {
                surface_y = y;
                break;
            }
        }
        let spawn_y = surface_y - 5;

        if !self.grid.in_bounds(spawn_x, spawn_y) {
            return;
        }

        let id = self.entities.spawn(EntityKind::Goblin);
        if let Some(g) = self.entities.get_mut(id) {
            g.build_humanoid(spawn_x as f32, spawn_y as f32);
        }
    }
}
