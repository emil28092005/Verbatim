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
        let _ = renderer.init();
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

            let _ = renderer.render(&self.grid, &self.entities, self.cam_x, self.cam_y);

            self.handle_input(vw, vh);
        }

        let _ = renderer.shutdown();
    }

    pub fn handle_input(&mut self, vw: usize, vh: usize) {
        let action = self.input.poll();
        match action {
            Action::Quit => self.running = false,
            Action::MoveLeft => self.player.move_left(&mut self.entities),
            Action::MoveRight => self.player.move_right(&mut self.entities),
            Action::Jump => {
                let on_ground = self.check_on_ground();
                self.player.jump(&mut self.entities, on_ground);
            }
            Action::MoveCameraLeft => self.cam_x -= 5,
            Action::MoveCameraRight => self.cam_x += 5,
            Action::MoveCameraUp => self.cam_y -= 5,
            Action::MoveCameraDown => self.cam_y += 5,
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
            Action::None => {}
        }
    }

    pub fn check_on_ground(&self) -> bool {
        if let Some(e) = self.player.entity(&self.entities) {
            for b in &e.bodies {
                if !b.alive {
                    continue;
                }
                let bx = b.x as i32;
                let by = b.y as i32;
                let below = self.grid.get(bx, by + 1);
                if below.is_solid() {
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
        let gravity = solver.gravity;
        let damping = solver.damping;
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
        let grid = &self.grid;
        let (cx, cy, cvx, cvy) = {
            let e = &self.entities.all()[idx];
            (e.cx, e.cy, e.cvx, e.cvy)
        };

        let mut new_cx = cx;
        let mut new_cy = cy;
        let mut new_cvx = cvx * damping;
        let mut new_cvy = cvy * damping;

        new_cvy += gravity;

        let v_mag = (new_cvx * new_cvx + new_cvy * new_cvy).sqrt();
        if v_mag > max_vel {
            new_cvx = new_cvx / v_mag * max_vel;
            new_cvy = new_cvy / v_mag * max_vel;
        }

        new_cx += new_cvx;
        new_cy += new_cvy;

        let offsets = self.entities.all()[idx].rest_offsets.clone();
        let radii: Vec<f32> = self.entities.all()[idx].bodies.iter().map(|b| b.radius).collect();

        for substep in 0..4 {
            let mut total_push_x = 0.0;
            let mut total_push_y = 0.0;
            let mut push_count = 0;

            for (i, &(ox, oy)) in offsets.iter().enumerate() {
                let bx = new_cx + ox;
                let by = new_cy + oy;
                let r = radii[i];

                let min_x = (bx - r).floor() as i32;
                let max_x = (bx + r).ceil() as i32;
                let min_y = (by - r).floor() as i32;
                let max_y = (by + r).ceil() as i32;

                for cy_cell in min_y..=max_y {
                    for cx_cell in min_x..=max_x {
                        if !grid.in_bounds(cx_cell, cy_cell) {
                            continue;
                        }
                        let cell = grid.get(cx_cell, cy_cell);
                        if cell.is_empty() || cell.is_liquid() {
                            continue;
                        }
                        if cell.material == MaterialId::Fire {
                            continue;
                        }
                        if !cell.is_solid() {
                            continue;
                        }

                        let cell_min_x = cx_cell as f32;
                        let cell_max_x = (cx_cell + 1) as f32;
                        let cell_min_y = cy_cell as f32;
                        let cell_max_y = (cy_cell + 1) as f32;

                        let inside_x = bx >= cell_min_x && bx < cell_max_x;
                        let inside_y = by >= cell_min_y && by < cell_max_y;

                        if inside_x && inside_y {
                            let dl = bx - cell_min_x;
                            let dr = cell_max_x - bx;
                            let dt = by - cell_min_y;
                            let db = cell_max_y - by;
                            let md = dl.min(dr).min(dt).min(db);
                            if md == dt {
                                total_push_y -= r + md;
                                push_count += 1;
                                if new_cvy > 0.0 { new_cvy = 0.0; }
                            } else if md == db {
                                total_push_y += r + md;
                                push_count += 1;
                                if new_cvy < 0.0 { new_cvy = 0.0; }
                            } else if md == dl {
                                total_push_x -= r + md;
                                push_count += 1;
                                if new_cvx > 0.0 { new_cvx = 0.0; }
                            } else {
                                total_push_x += r + md;
                                push_count += 1;
                                if new_cvx < 0.0 { new_cvx = 0.0; }
                            }
                        } else {
                            let closest_x = bx.max(cell_min_x).min(cell_max_x);
                            let closest_y = by.max(cell_min_y).min(cell_max_y);
                            let dx = bx - closest_x;
                            let dy = by - closest_y;
                            let dist_sq = dx * dx + dy * dy;
                            if dist_sq < r * r && dist_sq > 0.0001 {
                                let dist = dist_sq.sqrt();
                                let overlap = r - dist;
                                total_push_x += dx / dist * overlap;
                                total_push_y += dy / dist * overlap;
                                push_count += 1;
                                if dy < -0.5 && new_cvy > 0.0 { new_cvy = 0.0; }
                            }
                        }
                    }
                }
            }

            if push_count > 0 {
                let inv = 1.0 / push_count as f32;
                let px = total_push_x * inv;
                let py = total_push_y * inv;
                let mag = (px * px + py * py).sqrt();
                if mag > 0.001 {
                    new_cx += px;
                    new_cy += py;
                }
            }
        }

        let mut touching_lava = false;
        let mut touching_fire = false;
        let mut touching_acid = false;
        let mut in_liquid = false;

        for (i, &(ox, oy)) in offsets.iter().enumerate() {
            let bx = (new_cx + ox) as i32;
            let by = (new_cy + oy) as i32;
            if !grid.in_bounds(bx, by) {
                continue;
            }
            let cell = grid.get(bx, by);
            if cell.material == MaterialId::Lava { touching_lava = true; }
            if cell.material == MaterialId::Fire { touching_fire = true; }
            if cell.material == MaterialId::Acid { touching_acid = true; }
            if cell.is_liquid() { in_liquid = true; }
        }

        if let Some(e) = self.entities.all_mut().get_mut(idx) {
            e.cx = new_cx;
            e.cy = new_cy;
            e.cvx = new_cvx;
            e.cvy = new_cvy;
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
            if in_liquid {
                new_cvy *= 0.5;
                e.cvy = new_cvy;
            }
        }
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
