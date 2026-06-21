use std::time::{Duration, Instant};

use crate::entity::player::Player;
use crate::entity::{EntityKind, EntityManager, ItemManager, ItemType};
use crate::input::{Action, InputHandler};
use crate::physics::collision::resolve_grid_collision;
use crate::physics::projectile::{ProjectileManager, ProjectileType};
use crate::physics::verlet::VerletSolver;
use crate::render::lighting;
use crate::render::Renderer;
use crate::ui::UiLayer;
use crate::world::cell::MaterialId;
use crate::world::cellular::CellularAutomaton;
use crate::world::grid::Grid;

pub struct Game {
    pub grid: Grid,
    pub ca: CellularAutomaton,
    pub verlet: VerletSolver,
    pub entities: EntityManager,
    pub projectiles: ProjectileManager,
    pub items: ItemManager,
    pub player: Player,
    pub input: InputHandler,
    pub ui: UiLayer,
    pub cam_x: i32,
    pub cam_y: i32,
    pub cam_offset_x: i32,
    pub cam_offset_y: i32,
    pub running: bool,
    pub tick: u64,
    pub fixed_dt: Duration,
    pub accumulator: Duration,
    pub last_time: Instant,
    pub last_shot_tick: u64,
    pub shot_cooldown: u64,
    pub fireball_mode: bool,
    pub corpse_decomp_timer: u64,
    pub kills: u32,
    pub score: u32,
    pub depth: u32,
    pub fps: f32,
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
            projectiles: ProjectileManager::new(),
            items: ItemManager::new(),
            player,
            input: InputHandler::new(),
            ui: UiLayer::new(),
            cam_x: 100,
            cam_y: 100,
            cam_offset_x: 0,
            cam_offset_y: 0,
            running: true,
            tick: 0,
            fixed_dt: Duration::from_millis(16),
            accumulator: Duration::ZERO,
            last_time: Instant::now(),
            last_shot_tick: 0,
            shot_cooldown: 8,
            fireball_mode: false,
            corpse_decomp_timer: 0,
            kills: 0,
            score: 0,
            depth: 1,
            fps: 0.0,
        }
    }

    pub fn init_world(&mut self) {
        let w = self.grid.width;
        let h = self.grid.height;

        for x in 0..w {
            self.grid
                .set_material(x as i32, (h - 1) as i32, MaterialId::Stone);
            self.grid
                .set_material(x as i32, (h - 2) as i32, MaterialId::Dirt);
        }

        let surface_noise = |x: i32| -> i32 {
            let base = (h as i32 - 3) - ((x as f32 * 0.08).sin() * 4.0) as i32;
            let detail = ((x as f32 * 0.23).sin() * 2.0) as i32;
            (base + detail).max(10).min(h as i32 - 3)
        };

        for x in 0..w {
            let surface = surface_noise(x as i32);
            let biome = x / (w / 4);

            for y in surface..(h as i32 - 2) {
                if y == surface {
                    let mat = match biome {
                        0 => MaterialId::Grass,
                        1 => MaterialId::Grass,
                        2 => MaterialId::Dirt,
                        _ => MaterialId::Stone,
                    };
                    self.grid.set_material(x as i32, y, mat);
                } else if y > surface + 8 {
                    self.grid.set_material(x as i32, y, MaterialId::Stone);
                } else {
                    self.grid.set_material(x as i32, y, MaterialId::Dirt);
                }
            }
        }

        for _ in 0..8 {
            let cave_x = (self.ca.random_u32() % (w as u32 - 20) + 10) as i32;
            let cave_y = (self.ca.random_u32() % (h as u32 / 3) + (h as u32 / 3) * 2) as i32;
            let cave_r = (self.ca.random_u32() % 4 + 3) as i32;
            for dy in -cave_r..=cave_r {
                for dx in -cave_r..=cave_r {
                    if dx * dx + dy * dy <= cave_r * cave_r {
                        let cx = cave_x + dx;
                        let cy = cave_y + dy;
                        if cx > 1 && cx < w as i32 - 2 && cy > 1 && cy < h as i32 - 2 {
                            self.grid.set(cx, cy, crate::world::cell::Cell::empty());
                        }
                    }
                }
            }
        }

        for tree_x in [60, 75, 130, 145, 220] {
            let s = surface_noise(tree_x);
            for y in (s - 6)..s {
                if y > 5 {
                    self.grid.set_material(tree_x, y, MaterialId::Wood);
                }
            }
            for dy in -2..=0 {
                for dx in -2..=2 {
                    if dx * dx + dy * dy <= 5 {
                        let cx = tree_x + dx;
                        let cy = s - 6 + dy;
                        if cx > 1 && cx < w as i32 - 2 && cy > 1 {
                            if self.grid.get(cx, cy).is_empty() {
                                self.grid.set_material(cx, cy, MaterialId::Grass);
                            }
                        }
                    }
                }
            }
        }

        let water_x = 40;
        for x in water_x - 10..=water_x + 10 {
            let s = surface_noise(x);
            for y in s - 6..s {
                if self.grid.get(x, y).is_empty() {
                    self.grid.set_material(x, y, MaterialId::Water);
                }
            }
        }

        let lava_x = 200;
        for x in lava_x - 8..=lava_x + 8 {
            let s = surface_noise(x);
            for y in s - 4..s {
                if self.grid.get(x, y).is_empty() {
                    self.grid.set_material(x, y, MaterialId::Lava);
                }
            }
        }

        let sand_x = 160;
        for dx in -10..=10 {
            let s = surface_noise(sand_x + dx);
            let pile_h = (10.0 - (dx as f32).abs() * 0.8) as i32;
            for dy in 0..pile_h {
                let y = s - 1 - dy;
                if y > 5 && self.grid.get(sand_x + dx, y).is_empty() {
                    self.grid.set_material(sand_x + dx, y, MaterialId::Sand);
                }
            }
        }

        let acid_x = 20;
        for x in acid_x - 4..=acid_x + 4 {
            let s = surface_noise(x);
            for y in s - 3..s {
                if self.grid.get(x, y).is_empty() {
                    self.grid.set_material(x, y, MaterialId::Acid);
                }
            }
        }

        let wall_x = 110;
        let wall_s = surface_noise(wall_x);
        for y in (wall_s - 5)..wall_s {
            self.grid.set_material(wall_x, y, MaterialId::Stone);
            self.grid.set_material(wall_x + 1, y, MaterialId::Stone);
        }

        for _ in 0..6 {
            let px = (self.ca.random_u32() % (w as u32 - 20) + 10) as i32;
            let py = (self.ca.random_u32() % (h as u32 / 3) + (h as u32 / 3) * 2) as i32;
            for dy in 0..8 {
                for dx in -1..=1 {
                    let cx = px + dx;
                    let cy = py + dy;
                    if cx > 1 && cx < w as i32 - 2 && cy < h as i32 - 3 {
                        self.grid.set_material(cx, cy, MaterialId::Stone);
                    }
                }
            }
        }

        self.grid.fill_border(MaterialId::Stone);

        let cx = (w / 2) as f32;
        let surface_x = cx as i32;
        let mut surface_y = h as i32 - 3;
        for y in 0..h as i32 {
            if self.grid.get(surface_x, y).is_solid()
                && self.grid.get(surface_x, y).material != MaterialId::Stone
            {
                surface_y = y;
                break;
            }
        }
        let stair_y = (h as i32 - 2).max(surface_y + 2);
        self.grid
            .set_material(surface_x, stair_y, MaterialId::Stairs);

        let cy = (surface_y as f32) - 5.0;
        self.player.spawn_at(&mut self.entities, cx, cy);

        let (px, py) = self.player.center(&self.entities);
        self.center_camera_on(px, py);

        self.items
            .spawn(ItemType::Sword, px as i32 - 6, py as i32 + 1);
        self.items
            .spawn(ItemType::HealthPotion, px as i32 + 6, py as i32 + 1);
        self.items
            .spawn(ItemType::LeatherArmor, px as i32 - 3, py as i32 - 8);
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
        self.input.start();

        self.last_time = Instant::now();
        let mut frame_count = 0u32;
        let mut frame_time_acc = Duration::ZERO;
        let mut last_fps_print = Instant::now();

        while self.running {
            let now = Instant::now();
            let frame_time = now.duration_since(self.last_time);
            self.last_time = now;
            self.accumulator += frame_time;
            frame_time_acc += frame_time;
            frame_count += 1;
            if last_fps_print.elapsed() >= Duration::from_secs(1) {
                let avg_ms = frame_time_acc.as_secs_f32() * 1000.0 / frame_count as f32;
                self.fps = 1000.0 / avg_ms;
                frame_count = 0;
                frame_time_acc = Duration::ZERO;
                last_fps_print = Instant::now();
            }

            while self.accumulator >= self.fixed_dt {
                self.fixed_update();
                self.accumulator -= self.fixed_dt;
            }

            let vw = renderer.viewport_w();
            let vh = renderer.viewport_h();
            let (px, py) = self.player.center(&self.entities);
            self.cam_x = px as i32 - (vw as i32 / 2) + self.cam_offset_x;
            self.cam_y = py as i32 - (vh as i32 / 2) + self.cam_offset_y;

            self.build_ui(vw, vh);

            let light = lighting::compute_lighting(
                &self.grid,
                self.cam_x,
                self.cam_y,
                vw,
                vh,
                lighting::ambient_light(),
            );

            if let Err(e) = renderer.render(
                &self.grid,
                &self.entities,
                &self.items,
                &self.ui,
                self.cam_x,
                self.cam_y,
                Some(&light),
            ) {
                eprintln!("Render error: {}", e);
                break;
            }

            self.handle_input(vw, vh);
        }

        self.input.stop();

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
                Action::ShootLeft => self.player_shoot(-1.0, 0.0),
                Action::ShootRight => self.player_shoot(1.0, 0.0),
                Action::ShootUp => self.player_shoot(0.0, -1.0),
                Action::ShootDown => self.player_shoot(0.0, 1.0),
                Action::ToggleFireball => self.fireball_mode = !self.fireball_mode,
                Action::Descend => self.descend(),
                Action::UseItem => self.use_item(0),
                Action::DropItem => self.drop_item(0),
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
                                    self.grid.set(
                                        cx + dx,
                                        cy + dy,
                                        crate::world::cell::Cell::empty(),
                                    );
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

        // Movement: applied every tick while held (vector-style, direct velocity)
        let held = self.input.held_actions();
        let moving_left = held.iter().any(|a| *a == Action::MoveLeft);
        let moving_right = held.iter().any(|a| *a == Action::MoveRight);

        if moving_left && !moving_right {
            self.player.move_left(&mut self.entities);
        } else if moving_right && !moving_left {
            self.player.move_right(&mut self.entities);
        } else {
            self.player.stop_horizontal(&mut self.entities);
        }

        let mut held = held;
        for action in &mut held {
            match action {
                Action::MoveCameraLeft => self.cam_x -= 2,
                Action::MoveCameraRight => self.cam_x += 2,
                Action::MoveCameraUp => self.cam_y -= 2,
                Action::MoveCameraDown => self.cam_y += 2,
                Action::ShootLeft => self.player_shoot(-1.0, 0.0),
                Action::ShootRight => self.player_shoot(1.0, 0.0),
                Action::ShootUp => self.player_shoot(0.0, -1.0),
                Action::ShootDown => self.player_shoot(0.0, 1.0),
                _ => {}
            }
        }
    }

    pub fn build_ui(&mut self, vw: usize, vh: usize) {
        self.ui.clear();
        self.ui.tick_messages();

        let player = self.player.entity(&self.entities).cloned();
        let brush = self
            .input
            .paint_brush
            .to_material()
            .unwrap_or(crate::world::cell::MaterialId::Empty);
        let player_alive = player.as_ref().map(|e| e.alive).unwrap_or(false);
        let ui_w = (vw as i32) * crate::ui::UI_SCALE;
        let ui_h = (vh as i32) * crate::ui::UI_SCALE;
        self.ui
            .draw_character_panel(1, 1, player.as_ref(), &self.player);
        self.ui.draw_hud(
            ui_w as usize,
            ui_h as usize,
            player.as_ref(),
            self.tick,
            brush,
            self.kills,
            self.score,
            self.depth,
            &self.player,
            self.fps,
        );
        let msg_x = (ui_w / 2) - 24;
        self.ui.draw_messages(msg_x, 4);
        self.ui.draw_damage_numbers(self.cam_x, self.cam_y);
        self.ui.draw_edge_indicators(
            ui_w as usize,
            ui_h as usize,
            self.entities.all(),
            self.cam_x,
            self.cam_y,
        );
        self.ui
            .draw_entity_labels(self.entities.all(), self.cam_x, self.cam_y);
        self.ui
            .draw_status_icons(self.entities.all(), self.cam_x, self.cam_y);
        self.ui.draw_minimap(
            ui_w as usize,
            ui_h as usize,
            &self.grid,
            self.entities.all(),
            self.cam_x,
            self.cam_y,
        );
        if !player_alive {
            self.ui
                .draw_death_screen(ui_w as usize, ui_h as usize, self.kills, self.score);
        }

        for e in self.entities.all() {
            if !e.alive || e.kind == EntityKind::Corpse {
                continue;
            }
            let (sx, sy) = crate::ui::entity_screen_pos_ui(e, self.cam_x, self.cam_y);
            let top = sy - (e.half_h as i32 * crate::ui::UI_SCALE) - 2;
            self.ui
                .draw_health_bar(sx - 6, top, e.health, e.max_health, 12);
        }

        for p in self.projectiles.all() {
            let sx = (p.x as i32 - self.cam_x) * crate::ui::UI_SCALE;
            let sy = (p.y as i32 - self.cam_y) * crate::ui::UI_SCALE;
            if sx >= 0 && sx < ui_w && sy >= 0 && sy < ui_h {
                self.ui
                    .set(sx, sy, p.draw_char(), p.draw_color(), [0, 0, 0]);
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

        self.update_active_chunks();
        self.ca.step(&mut self.grid);

        self.update_entities();
        self.update_slime_ai();
        self.update_goblin_ai();
        self.update_combat();
        self.update_projectiles();
        self.apply_world_damage();
        self.decompose_corpses();
        self.update_status_effects();
        self.update_score();
        self.update_item_pickup();

        if self.tick % 30 == 0 {
            self.try_spawn_goblin();
        }
        if self.tick % 45 == 0 {
            self.try_spawn_slime();
        }

        self.grid.swap_modified_flags();
    }

    fn update_score(&mut self) {
        let mut new_kills = 0;
        for e in self.entities.all_mut() {
            if !e.alive && e.kind == EntityKind::Corpse && !e.counted_for_score {
                e.counted_for_score = true;
                new_kills += 1;
            }
        }
        if new_kills > 0 {
            self.kills += new_kills;
            self.score += new_kills * 10;
            if let Some(player) = self.player.entity_mut(&mut self.entities) {
                player.add_xp(new_kills * 25);
            }
        }
    }

    fn update_status_effects(&mut self) {
        for e in self.entities.all_mut() {
            if e.alive {
                e.apply_status_effects();
            }
        }
    }

    fn update_item_pickup(&mut self) {
        let (px, py) = self.player.center(&self.entities);
        let ix = px as i32;
        let iy = py as i32;
        let mut picked = Vec::new();
        for (idx, item) in self.items.all().iter().enumerate() {
            if (item.x - ix).abs() <= 1 && (item.y - iy).abs() <= 1 {
                picked.push(idx);
            }
        }
        for idx in picked.into_iter().rev() {
            let item = self.items.all_mut().remove(idx);
            self.ui.add_message(&format!("Picked up {}", item.name()));
            self.player.inventory.push(item);
        }
    }

    pub fn descend(&mut self) {
        let can_descend = match self.player.entity(&self.entities) {
            Some(e) => {
                let foot_x = e.cx as i32;
                let foot_y = (e.cy + e.half_h).ceil() as i32;
                self.grid.get(foot_x, foot_y).material == MaterialId::Stairs
            }
            None => false,
        };
        if !can_descend {
            return;
        }
        self.depth += 1;
        self.grid = Grid::new();
        self.entities = EntityManager::new();
        self.player = Player::new(&mut self.entities);
        self.projectiles = ProjectileManager::new();
        self.items = ItemManager::new();
        self.corpse_decomp_timer = 0;
        self.init_world();
        self.ui
            .add_message(&format!("Descended to depth {}", self.depth));
    }

    pub fn use_item(&mut self, index: usize) {
        if index >= self.player.inventory.len() {
            return;
        }
        let item = self.player.inventory[index].clone();
        let name = item.name();
        if item.is_weapon() {
            if let Some(old) = self.player.weapon.take() {
                self.player.inventory.push(old);
            }
            self.player.weapon = Some(item);
            self.player.inventory.remove(index);
            self.ui.add_message(&format!("Equipped {}", name));
        } else if item.is_armor() {
            if let Some(old) = self.player.armor.take() {
                self.player.inventory.push(old);
            }
            self.player.armor = Some(item);
            self.player.inventory.remove(index);
            self.ui.add_message(&format!("Equipped {}", name));
        } else if item.is_consumable() {
            let heal = item.heal_amount();
            if let Some(p) = self.player.entity_mut(&mut self.entities) {
                p.health = (p.health + heal).min(p.max_health);
            }
            self.player.inventory.remove(index);
            self.ui.add_message(&format!("Consumed {}", name));
        }
    }

    pub fn drop_item(&mut self, index: usize) {
        if index >= self.player.inventory.len() {
            return;
        }
        let item = self.player.inventory.remove(index);
        let (px, py) = self.player.center(&self.entities);
        self.items.spawn(item.typ, px as i32, py as i32 + 1);
        self.ui.add_message(&format!("Dropped {}", item.name()));
    }

    fn update_active_chunks(&mut self) {
        self.grid.deactivate_all();

        for e in self.entities.all() {
            let (cx, cy) = e.center();
            self.grid.activate_around(cx as i32, cy as i32, 2);
        }

        for p in self.projectiles.all() {
            self.grid.activate_around(p.x as i32, p.y as i32, 1);
        }

        for item in self.items.all() {
            self.grid.activate_around(item.x, item.y, 1);
        }

        let chunk_size = self.grid.chunk_size as i32;
        for cy in 0..self.grid.chunks_y as i32 {
            for cx in 0..self.grid.chunks_x as i32 {
                let idx = self.grid.chunk_index(cx, cy);
                if self.grid.chunks[idx].modified || self.grid.chunks[idx].was_modified {
                    self.grid
                        .activate_around(cx * chunk_size, cy * chunk_size, 1);
                }
            }
        }
    }

    pub fn update_projectiles(&mut self) {
        self.projectiles.update(&self.grid);
        self.projectiles
            .resolve_hits(&mut self.grid, self.entities.all_mut(), &mut self.ui);
        self.projectiles.cull_dead();
    }

    pub fn player_shoot(&mut self, dir_x: f32, dir_y: f32) {
        if self.tick > self.last_shot_tick && self.tick - self.last_shot_tick < self.shot_cooldown {
            return;
        }
        let (px, py) = self.player.center(&self.entities);
        let owner = self.player.entity_id;
        let speed = if self.fireball_mode { 2.2 } else { 3.0 };
        let vx = dir_x * speed;
        let vy = dir_y * speed;
        let typ = if self.fireball_mode {
            ProjectileType::Fireball
        } else if self.tick % 4 == 0 {
            ProjectileType::MagicBolt
        } else {
            ProjectileType::Arrow
        };
        let spawn_x = px + dir_x * 3.0;
        let spawn_y = py + dir_y * 3.0;
        let damage_bonus = self
            .player
            .weapon
            .as_ref()
            .map(|w| w.damage_bonus())
            .unwrap_or(0.0);
        self.projectiles
            .spawn(typ, spawn_x, spawn_y, vx, vy, owner, damage_bonus);
        self.last_shot_tick = self.tick;
    }

    fn update_goblin_ai(&mut self) {
        let (px, py) = self.player.center(&self.entities);
        let tick = self.tick;
        let goblin_data: Vec<(usize, f32, f32, f32, u32)> = self
            .entities
            .all()
            .iter()
            .enumerate()
            .filter(|(_, e)| e.alive && e.kind == EntityKind::Goblin)
            .map(|(i, e)| (i, e.cx, e.cy, e.health, e.id))
            .collect();

        for (idx, gx, gy, health, goblin_id) in goblin_data {
            if !self.entities.all()[idx].rigid {
                continue;
            }
            let dx = px - gx;
            let dy = py - gy;
            let dist_sq = dx * dx + dy * dy;
            let dist = dist_sq.sqrt();
            if dist < 0.5 {
                continue;
            }
            let dir_x = dx / dist;

            if dist > 20.0 && dist < 60.0 && tick % 80 == 0 {
                let vx = dir_x * 1.2;
                let vy = (dy / dist) * 0.8;
                self.projectiles
                    .spawn_arrow(gx, gy - 2.0, vx, vy, goblin_id);
            }

            if health < 10.0 {
                let flee = self.entities.all()[idx].cx < px;
                let move_dir = if flee { -1.0 } else { 1.0 };
                if let Some(e) = self.entities.all_mut().get_mut(idx) {
                    e.set_horizontal_vel(move_dir * 0.6);
                }
            } else if dist > 6.0 {
                if let Some(e) = self.entities.all_mut().get_mut(idx) {
                    e.set_horizontal_vel(dir_x * 0.45);
                }
            } else if dist < 4.0 {
                if let Some(e) = self.entities.all_mut().get_mut(idx) {
                    e.set_horizontal_vel(-dir_x * 0.45);
                }
            }
        }
    }

    fn decompose_corpses(&mut self) {
        self.corpse_decomp_timer += 1;
        if self.corpse_decomp_timer < 40 {
            return;
        }
        self.corpse_decomp_timer = 0;
        let mut drops: Vec<(i32, i32, usize, usize)> = Vec::new();
        for (idx, e) in self.entities.all().iter().enumerate() {
            if e.alive || e.kind != EntityKind::Corpse {
                continue;
            }
            if e.bodies.is_empty() {
                continue;
            }
            let alive_indices: Vec<usize> = e
                .bodies
                .iter()
                .enumerate()
                .filter(|(_, b)| b.alive)
                .map(|(i, _)| i)
                .collect();
            if alive_indices.is_empty() {
                continue;
            }
            let pick = self.ca.random_usize(alive_indices.len());
            let bi = alive_indices[pick];
            let b = &e.bodies[bi];
            let gx = b.x.floor() as i32;
            let gy = b.y.floor() as i32;
            if self.grid.in_bounds(gx, gy) && self.grid.get(gx, gy).is_empty() {
                drops.push((gx, gy, idx, bi));
            }
        }
        for (gx, gy, eidx, bidx) in drops {
            self.grid.set_material(gx, gy, MaterialId::Flesh);
            if let Some(e) = self.entities.all_mut().get_mut(eidx) {
                if let Some(b) = e.bodies.get_mut(bidx) {
                    b.alive = false;
                    b.health = 0.0;
                }
                let alive_count = e.bodies.iter().filter(|b| b.alive).count();
                if alive_count == 0 {
                    e.health = 0.0;
                }
            }
        }
    }

    fn update_slime_ai(&mut self) {
        let (px, py) = self.player.center(&self.entities);
        let tick = self.tick;

        let slime_data: Vec<(usize, f32, f32, bool, f32)> = self
            .entities
            .all()
            .iter()
            .enumerate()
            .filter(|(_, e)| e.alive && e.kind == EntityKind::Slime)
            .map(|(i, e)| (i, e.cx, e.cy, e.rigid, e.health))
            .collect();

        for (idx, sx, sy, rigid, _health) in slime_data {
            if !rigid {
                continue;
            }
            let dx = px - sx;
            let dy = py - sy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < 0.5 {
                continue;
            }

            let jump_phase = tick % 60;
            if jump_phase == 0 && dist < 40.0 {
                let dir_x = dx / dist;
                let dir_y = dy / dist;
                let jump_power = 0.8 + (1.0 - dist / 40.0).min(0.5) * 0.5;
                if let Some(e) = self.entities.all_mut().get_mut(idx) {
                    e.set_horizontal_vel(dir_x * jump_power);
                    let vy = if dy < -1.0 {
                        -jump_power * 0.8
                    } else {
                        -jump_power * 0.6
                    };
                    e.set_vertical_vel(vy + dir_y * jump_power * 0.3);
                }
            } else if jump_phase == 30 {
                if let Some(e) = self.entities.all_mut().get_mut(idx) {
                    e.set_horizontal_vel(0.0);
                }
            }
        }
    }

    fn update_combat(&mut self) {
        let player_id = self.player.entity_id;
        let player_alive = self
            .entities
            .get(player_id)
            .map(|e| e.alive)
            .unwrap_or(false);
        if !player_alive {
            return;
        }
        let player_center = self.player.center(&self.entities);
        let player_half_w = self
            .entities
            .get(player_id)
            .map(|e| e.half_w)
            .unwrap_or(3.0);
        let player_half_h = self
            .entities
            .get(player_id)
            .map(|e| e.half_h)
            .unwrap_or(6.0);

        let enemy_data: Vec<(usize, EntityKind, f32, f32, f32, f32)> = self
            .entities
            .all()
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                e.alive && e.kind != EntityKind::Player && e.kind != EntityKind::Corpse
            })
            .map(|(i, e)| (i, e.kind, e.cx, e.cy, e.half_w, e.half_h))
            .collect();

        for (_idx, kind, ex, ey, ew, eh) in enemy_data {
            let dx = (ex - player_center.0).abs();
            let dy = (ey - player_center.1).abs();
            if dx < ew + player_half_w && dy < eh + player_half_h {
                if self.tick % 20 == 0 {
                    let base = match kind {
                        EntityKind::Goblin => 8.0,
                        EntityKind::Slime => 5.0,
                        _ => 0.0,
                    };
                    let armor = self
                        .player
                        .armor
                        .as_ref()
                        .map(|a| a.armor_bonus())
                        .unwrap_or(0.0);
                    let damage = (base - armor).max(0.0);
                    if damage > 0.0 {
                        let player_before = self
                            .entities
                            .get(player_id)
                            .map(|e| e.health)
                            .unwrap_or(0.0);
                        if let Some(p) = self.entities.get_mut(player_id) {
                            p.take_damage(damage);
                        }
                        let player_after = self
                            .entities
                            .get(player_id)
                            .map(|e| e.health)
                            .unwrap_or(0.0);
                        self.ui.add_damage_number(
                            player_center.0,
                            player_center.1 - player_half_h - 2.0,
                            &format!("-{:.0}", player_before - player_after),
                        );
                        let knockback_dir = if player_center.0 < ex { -1.0 } else { 1.0 };
                        if let Some(p) = self.entities.get_mut(player_id) {
                            p.set_horizontal_vel(knockback_dir * 0.5);
                        }
                        let msg = match kind {
                            EntityKind::Goblin => "Goblin hits you!",
                            EntityKind::Slime => "Slime burns you!",
                            _ => "Enemy hits you!",
                        };
                        self.ui.add_message(msg);
                    }
                }
            }
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

    fn update_rigid_entity(&mut self, idx: usize, gravity: f32, _damping: f32, max_vel: f32) {
        let (cx, cy, cvx, cvy, half_w, half_h) = {
            let e = &self.entities.all()[idx];
            (e.cx, e.cy, e.cvx, e.cvy, e.half_w, e.half_h)
        };

        let mut nx = cx;
        let mut ny = cy;
        let mut nvx = cvx;
        let mut nvy = cvy * 0.99;
        nvy += gravity;

        let v_mag = (nvx * nvx + nvy * nvy).sqrt();
        if v_mag > max_vel {
            nvx = nvx / v_mag * max_vel;
            nvy = nvy / v_mag * max_vel;
        }

        // Step 1: Try horizontal movement with slope stepping
        nx += nvx;
        if self.aabb_overlaps_solid(nx, ny, half_w, half_h) {
            // Try stepping up 1 cell
            let step = 1.0;
            if !self.aabb_overlaps_solid(nx, ny - step, half_w, half_h) {
                // Can step up — snap to top of the obstacle
                ny -= step;
            } else {
                // Blocked — resolve X
                let (resolved_x, hit) = self.resolve_aabb_x(idx, nx, ny, half_w, half_h, nvx);
                nx = resolved_x;
                if hit {
                    nvx = 0.0;
                }
            }
        }

        // Step 2: Vertical movement
        ny += nvy;
        let (resolved_y, hit_floor, hit_ceiling) =
            self.resolve_aabb_y(idx, nx, ny, half_w, half_h, nvy > 0.0);
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
                    if !grid.in_bounds(x, y) {
                        continue;
                    }
                    let cell = grid.get(x, y);
                    if cell.material == MaterialId::Lava {
                        tl = true;
                    }
                    if cell.material == MaterialId::Fire {
                        tf = true;
                    }
                    if cell.material == MaterialId::Acid {
                        ta = true;
                    }
                    if cell.is_liquid() {
                        il = true;
                    }
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
                e.health -= 1.0;
                for b in &mut e.bodies {
                    if b.alive {
                        b.health -= 1.0;
                        if !b.on_fire {
                            b.on_fire = true;
                        }
                    }
                }
            }
            if touching_fire {
                e.health -= 0.15;
                for b in &mut e.bodies {
                    if b.alive {
                        b.health -= 0.15;
                        if !b.on_fire && b.health < 80.0 {
                            b.on_fire = true;
                        }
                    }
                }
            }
            if touching_acid {
                e.health -= 0.25;
                for b in &mut e.bodies {
                    if b.alive {
                        b.health -= 0.25;
                    }
                }
            }
        }
    }

    fn aabb_overlaps_solid(&self, cx: f32, cy: f32, hw: f32, hh: f32) -> bool {
        let grid = &self.grid;
        let left = cx - hw;
        let right = cx + hw;
        let top = cy - hh;
        let bottom = cy + hh;

        let min_x = left.floor() as i32;
        let max_x = right.ceil() as i32;
        let min_y = top.floor() as i32;
        let max_y = bottom.ceil() as i32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if !grid.in_bounds(x, y) {
                    continue;
                }
                let cell = grid.get(x, y);
                if !cell.is_solid() {
                    continue;
                }
                let cl = x as f32;
                let cr = (x + 1) as f32;
                let ct = y as f32;
                let cb = (y + 1) as f32;
                if right > cl && left < cr && bottom > ct && top < cb {
                    return true;
                }
            }
        }
        false
    }

    fn resolve_aabb_x(
        &self,
        _idx: usize,
        cx: f32,
        cy: f32,
        hw: f32,
        hh: f32,
        vx: f32,
    ) -> (f32, bool) {
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
                if !grid.in_bounds(x, y) {
                    continue;
                }
                let cell = grid.get(x, y);
                if !cell.is_solid() {
                    continue;
                }

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

    fn resolve_aabb_y(
        &self,
        _idx: usize,
        cx: f32,
        cy: f32,
        hw: f32,
        hh: f32,
        moving_down: bool,
    ) -> (f32, bool, bool) {
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
                if !grid.in_bounds(x, y) {
                    continue;
                }
                let cell = grid.get(x, y);
                if !cell.is_solid() {
                    continue;
                }

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

    fn update_ragdoll_entity(
        &mut self,
        idx: usize,
        solver: &crate::physics::verlet::VerletSolver,
        substeps: u32,
    ) {
        let grid = &self.grid;
        let e = match self.entities.all_mut().get_mut(idx) {
            Some(e) => e,
            None => return,
        };
        let alive = e.alive;
        let bodies = &mut e.bodies;
        let constraints = &e.constraints;

        let effective_substeps = if alive { substeps } else { 1 };
        let constraint_iters = if alive { 4 } else { 1 };

        for b in bodies.iter_mut() {
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

        for _ in 0..effective_substeps {
            solver.integrate(bodies);

            for b in bodies.iter_mut() {
                if !b.alive {
                    continue;
                }
                let result = resolve_grid_collision(grid, b);
                if result.touching_lava {
                    b.health -= 0.5;
                    if !b.on_fire {
                        b.on_fire = true;
                    }
                }
                if result.touching_fire {
                    b.health -= 0.15;
                    if !b.on_fire && b.health < 80.0 {
                        b.on_fire = true;
                    }
                }
                if result.touching_acid {
                    b.health -= 0.25;
                }
            }

            for _ci in 0..constraint_iters {
                solver.solve_constraints(bodies, constraints, 1);
                for b in bodies.iter_mut() {
                    if !b.alive {
                        continue;
                    }
                    resolve_grid_collision(grid, b);
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
        let alive_goblins = self
            .entities
            .all()
            .iter()
            .filter(|e| e.alive && e.kind == EntityKind::Goblin)
            .count();
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

    fn try_spawn_slime(&mut self) {
        let alive_slimes = self
            .entities
            .all()
            .iter()
            .filter(|e| e.alive && e.kind == EntityKind::Slime)
            .count();
        if alive_slimes >= 2 {
            return;
        }

        let (px, _py) = self.player.center(&self.entities);
        let spawn_x = px as i32 + if px as i32 % 2 == 0 { -18 } else { 18 };
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
        let spawn_y = surface_y - 3;

        if !self.grid.in_bounds(spawn_x, spawn_y) {
            return;
        }

        let id = self.entities.spawn(EntityKind::Slime);
        if let Some(s) = self.entities.get_mut(id) {
            s.build_humanoid(spawn_x as f32, spawn_y as f32);
        }
    }
}
