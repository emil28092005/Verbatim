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

        let cx = (w / 2) as f32;
        let cy = (h as f32 / 2.0) - 20.0;
        self.player.spawn_at(&mut self.entities, cx, cy);

        let (px, py) = self.player.center(&self.entities);
        self.center_camera_on(px, py);

        self.grid.fill_border(MaterialId::Stone);
    }

    fn center_camera_on(&mut self, px: f32, py: f32) {
        self.cam_x = px as i32 - 40;
        self.cam_y = py as i32 - 12;
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

            let (px, py) = self.player.center(&self.entities);
            self.center_camera_on(px, py);

            let _ = renderer.render(&self.grid, &self.entities, self.cam_x, self.cam_y);

            self.handle_input(renderer.viewport_w(), renderer.viewport_h());
        }

        let _ = renderer.shutdown();
    }

    fn handle_input(&mut self, vw: usize, vh: usize) {
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

    fn check_on_ground(&self) -> bool {
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

    fn fixed_update(&mut self) {
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
        let grid = &self.grid;

        let entities_data: Vec<(usize, Vec<crate::physics::verlet::SubBody>, Vec<crate::physics::verlet::Constraint>)>;
        entities_data = {
            let mut data = Vec::new();
            for (i, e) in self.entities.all().iter().enumerate() {
                data.push((i, e.bodies.clone(), e.constraints.clone()));
            }
            data
        };

        for (idx, mut bodies, constraints) in entities_data {
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

            solver.integrate(&mut bodies);

            for b in &mut bodies {
                if !b.alive {
                    continue;
                }
                let result = resolve_grid_collision(grid, b);
                if result.touching_lava {
                    b.health -= 2.0;
                    if !b.on_fire {
                        b.on_fire = true;
                    }
                }
                if result.touching_fire {
                    b.health -= 0.5;
                    if !b.on_fire && b.health < 80.0 {
                        b.on_fire = true;
                    }
                }
                if result.touching_acid {
                    b.health -= 1.0;
                }
                if result.in_liquid {
                    let body_density = crate::world::material::MaterialRegistry::instance()
                        .get(b.material).density;
                    if body_density > result.liquid_density {
                        b.y += 0.02;
                    }
                }
            }

            solver.solve_constraints(&mut bodies, &constraints, 3);

            if let Some(e) = self.entities.all_mut().get_mut(idx) {
                e.bodies = bodies;

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

        let (px, py) = self.player.center(&self.entities);
        let spawn_x = px as i32 + if px as i32 % 2 == 0 { 15 } else { -15 };
        let spawn_y = py as i32 - 5;

        if !self.grid.in_bounds(spawn_x, spawn_y) {
            return;
        }

        let id = self.entities.spawn(EntityKind::Goblin);
        if let Some(g) = self.entities.get_mut(id) {
            g.build_humanoid(spawn_x as f32, spawn_y as f32);
        }
    }
}
