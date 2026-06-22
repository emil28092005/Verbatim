use std::time::{Duration, Instant};

use crate::entity::player::Player;
use crate::entity::{EntityKind, EntityManager, ItemManager};
use crate::input::{Action, InputHandler};
use crate::physics::collision::resolve_grid_collision;
use crate::physics::projectile::{ProjectileManager, ProjectileType};
use crate::physics::verlet::VerletSolver;
use crate::render::lighting;
use crate::render::Renderer;
use crate::ui::UiLayer;
use crate::world::cache::WorldCache;
use crate::world::cell::MaterialId;
use crate::world::cellular::CellularAutomaton;
use crate::world::chunked_grid::ChunkedGrid;
use crate::world::grid::{WORLD_H, WORLD_W};
use crate::world::worldgen::WorldGenerator;

use crate::audio::AudioEngine;

pub struct Game {
    pub grid: ChunkedGrid,
    pub ca: CellularAutomaton,
    pub verlet: VerletSolver,
    pub entities: EntityManager,
    pub projectiles: ProjectileManager,
    pub items: ItemManager,
    pub player: Player,
    pub input: InputHandler,
    pub ui: UiLayer,
    pub audio: AudioEngine,
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
    pub inventory_open: bool,
    pub inventory_mouse_x: i32,
    pub inventory_mouse_y: i32,
    pub seed: u64,
    pub cache_dir: Option<String>,
}

impl Game {
    pub fn new() -> Self {
        Self::new_with_size(WORLD_W, WORLD_H)
    }

    fn new_with_size(width: usize, height: usize) -> Self {
        let mut entities = EntityManager::new();
        let player = Player::new(&mut entities);
        Self {
            grid: ChunkedGrid::with_size(width, height),
            ca: CellularAutomaton::new(),
            verlet: VerletSolver::new(),
            entities,
            projectiles: ProjectileManager::new(),
            items: ItemManager::new(),
            player,
            input: InputHandler::new(),
            ui: UiLayer::new(),
            audio: AudioEngine::new(),
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
            inventory_open: false,
            inventory_mouse_x: 0,
            inventory_mouse_y: 0,
            seed: 0x1234567890ABCDEF,
            cache_dir: None,
        }
    }

    pub fn new_random() -> Self {
        let seed = crate::world::cellular::random_seed();
        let cache_dir = Some("cache/worlds".to_string());
        let mut entities = EntityManager::new();
        let player = Player::new(&mut entities);
        let mut ca = CellularAutomaton::new();
        ca.seed(seed);
        Self {
            grid: ChunkedGrid::infinite(seed, cache_dir.clone()),
            ca,
            verlet: VerletSolver::new(),
            entities,
            projectiles: ProjectileManager::new(),
            items: ItemManager::new(),
            player,
            input: InputHandler::new(),
            ui: UiLayer::new(),
            audio: AudioEngine::new(),
            cam_x: 0,
            cam_y: 0,
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
            inventory_open: false,
            inventory_mouse_x: 0,
            inventory_mouse_y: 0,
            seed,
            cache_dir,
        }
    }

    pub fn init_world(&mut self) {
        let cache_dir = self.cache_dir.clone();
        let (px, py) = if let Some(ref root) = cache_dir {
            let has_meta = WorldCache::meta_exists(root, self.seed);
            if has_meta {
                if let Err(e) = WorldCache::load_meta(
                    root,
                    self.seed,
                    &mut self.player,
                    &mut self.entities,
                    &mut self.items,
                ) {
                    eprintln!("World cache meta load failed: {}", e);
                } else {
                    let (px, py) = self.player.center(&self.entities);
                    self.grid.ensure_loaded(px as i32, py as i32, 3);
                    self.center_camera_on(px, py);
                    return;
                }
            }
            let (px, py) = WorldGenerator::new(&mut self.ca).generate(
                &mut self.grid,
                &mut self.items,
                &mut self.player,
                &mut self.entities,
                self.depth,
            );
            self.center_camera_on(px, py);
            if let Err(e) = WorldCache::save_meta(root, self.seed, px, py, self.depth, &self.items)
            {
                eprintln!("World cache meta save failed: {}", e);
            }
            (px, py)
        } else {
            let (px, py) = WorldGenerator::new(&mut self.ca).generate(
                &mut self.grid,
                &mut self.items,
                &mut self.player,
                &mut self.entities,
                self.depth,
            );
            self.center_camera_on(px, py);
            (px, py)
        };
        let _ = (px, py);
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
            if on_ground {
                self.audio.play("jump");
            }
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

        let ui_w = (vw as i32) * crate::ui::UI_SCALE;
        let ui_h = (vh as i32) * crate::ui::UI_SCALE;
        let fs = ((ui_h / 200).max(2)).min(6) as i32;
        self.ui.set_font_scale(fs);
        self.ui.resize(ui_w as usize, ui_h as usize);

        let player = self.player.entity(&self.entities).cloned();
        let brush = self
            .input
            .paint_brush
            .to_material()
            .unwrap_or(crate::world::cell::MaterialId::Empty);
        let player_alive = player.as_ref().map(|e| e.alive).unwrap_or(false);
        if !self.inventory_open {
            self.ui
                .draw_character_panel(1, 1, player.as_ref(), &self.player);
        }
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
        let msg_x = (ui_w / 2) - 24 * fs;
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
        if self.inventory_open {
            self.ui.draw_inventory_overlay(
                ui_w as usize,
                ui_h as usize,
                &self.player,
                self.inventory_mouse_x,
                self.inventory_mouse_y,
            );
        }
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

        self.stream_chunks();
        self.update_active_chunks();
        self.apply_gas_damage();
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

        if self.tick % (30u64 - (self.depth as u64 * 3).min(20)) == 0 {
            self.try_spawn_goblin();
        }
        if self.tick % (45u64 - (self.depth as u64 * 4).min(30)) == 0 {
            self.try_spawn_slime();
        }

        self.play_ambient_sounds();
        self.grid.swap_modified_flags();
    }

    fn play_ambient_sounds(&mut self) {
        if !self.audio.is_enabled() {
            return;
        }
        let (px, py) = self.player.center(&self.entities);
        let radius = 15i32;
        let mut has_lava = false;
        let mut has_fire = false;
        let mut has_acid = false;
        let mut has_water = false;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let x = px as i32 + dx;
                let y = py as i32 + dy;
                if !self.grid.in_bounds(x, y) {
                    continue;
                }
                let cell = self.grid.get(x, y);
                match cell.material {
                    MaterialId::Lava => has_lava = true,
                    MaterialId::Fire => has_fire = true,
                    MaterialId::Acid => has_acid = true,
                    MaterialId::Water => has_water = true,
                    _ => {}
                }
            }
        }
        if has_lava && self.tick % 30 == 0 {
            self.audio.play_throttled("lava_bubble", 500);
        }
        if has_fire && self.tick % 15 == 0 {
            self.audio.play_throttled("fire_crackle", 200);
        }
        if has_acid && self.tick % 20 == 0 {
            self.audio.play_throttled("acid_sizzle", 300);
        }
        if has_water && self.tick % 40 == 0 {
            self.audio.play_throttled("water_splash", 600);
        }
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

    fn apply_gas_damage(&mut self) {
        for e in self.entities.all_mut() {
            if e.alive {
                let (ex, ey) = e.center();
                let (gas_type, gas_density) = self.grid.get_gas(ex as i32, ey as i32);
                if gas_type == 2 && gas_density > 50 {
                    e.health -= (gas_density as f32 - 50.0) * 0.1;
                }
                if gas_type == 3 && gas_density > 100 {
                    e.health -= 2.0;
                }
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
            self.audio.play("pickup");
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
        self.grid = ChunkedGrid::with_size(self.grid.width, self.grid.height);
        self.entities = EntityManager::new();
        self.player = Player::new(&mut self.entities);
        self.projectiles = ProjectileManager::new();
        self.items = ItemManager::new();
        self.corpse_decomp_timer = 0;
        self.init_world();
        self.ui
            .add_message(&format!("Descended to depth {}", self.depth));
        self.audio.play("descend");
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
        self.audio.play("powerup");
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

    fn stream_chunks(&mut self) {
        if !self.grid.is_infinite() && self.grid.width <= 2048 {
            return;
        }
        let (px, py) = self.player.center(&self.entities);
        let px = px as i32;
        let py = py as i32;
        let (pcx, pcy, _, _) = self.grid.chunk_at(px, py);
        let radius = 3;
        let chunk_size = self.grid.chunk_size as i32;

        let mut to_generate: Vec<(i32, i32)> = Vec::new();
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let cx = pcx + dx;
                let cy = pcy + dy;
                let ox = cx * chunk_size;
                let oy = cy * chunk_size;
                if !self.grid.in_bounds(ox, oy) {
                    continue;
                }
                self.grid.ensure_chunk(cx, cy);
                if !self.grid.is_chunk_generated(cx, cy) {
                    to_generate.push((cx, cy));
                }
            }
        }

        let gen_budget = 2;
        for &(cx, cy) in to_generate.iter().take(gen_budget) {
            WorldGenerator::new(&mut self.ca).generate_chunk(&mut self.grid, cx, cy);
        }

        if let Some(ref dir) = self.cache_dir {
            let save_radius = radius + 2;
            if self.tick % 10 == 0 {
                let coords = self.grid.all_chunk_coords();
                for (cx, cy) in coords {
                    let dx = (cx - pcx).abs();
                    let dy = (cy - pcy).abs();
                    if (dx > save_radius || dy > save_radius) && self.grid.is_chunk_modified(cx, cy)
                    {
                        let path = crate::world::chunked_grid::chunk_path(dir, self.seed, cx, cy);
                        let _ = self.grid.save_chunk(path.to_str().unwrap(), cx, cy);
                        break;
                    }
                }
            }
        }

        if self.tick % 120 == 0 {
            let unload_radius = radius + 4;
            let to_unload: Vec<(i32, i32)> = self
                .grid
                .all_chunk_coords()
                .into_iter()
                .filter(|(cx, cy)| {
                    let dx = (cx - pcx).abs();
                    let dy = (cy - pcy).abs();
                    dx > unload_radius || dy > unload_radius
                })
                .collect();
            for (cx, cy) in to_unload {
                if self.grid.is_chunk_modified(cx, cy) {
                    if let Some(ref dir) = self.cache_dir {
                        let path = crate::world::chunked_grid::chunk_path(dir, self.seed, cx, cy);
                        let _ = self.grid.save_chunk(path.to_str().unwrap(), cx, cy);
                    }
                }
                self.grid.unload_chunk(cx, cy);
            }
        }
    }

    fn update_active_chunks(&mut self) {
        let chunk_size = self.grid.chunk_size as i32;
        let (px, py) = self.player.center(&self.entities);
        let pcx = px as i32 / chunk_size;
        let pcy = py as i32 / chunk_size;

        if !self.grid.is_infinite() {
            for e in self.entities.all() {
                let (cx, cy) = e.center();
                self.grid.activate_around(cx as i32, cy as i32, 1);
            }
            for (cx, cy) in self.grid.all_chunk_coords() {
                if self.grid.get_chunk_dirty(cx, cy).is_some() {
                    self.grid.set_chunk_active(cx, cy, true);
                }
            }
            return;
        }

        self.grid.deactivate_all();

        for dy in -2..=2 {
            for dx in -2..=2 {
                self.grid.set_chunk_active(pcx + dx, pcy + dy, true);
            }
        }
        for e in self.entities.all() {
            let (cx, cy) = e.center();
            self.grid.activate_around(cx as i32, cy as i32, 1);
        }
        for (cx, cy) in self.grid.all_chunk_coords() {
            if self.grid.get_chunk_dirty(cx, cy).is_some() {
                if (cx - pcx).abs() <= 1 && (cy - pcy).abs() <= 1 {
                    self.grid.set_chunk_active(cx, cy, true);
                }
            }
        }
    }

    pub fn update_projectiles(&mut self) {
        let count_before = self.projectiles.all().len();
        self.projectiles.update(&self.grid);
        self.projectiles
            .resolve_hits(&mut self.grid, self.entities.all_mut(), &mut self.ui);
        self.projectiles.cull_dead();
        let count_after = self.projectiles.all().len();
        if count_after < count_before {
            let any_fireball = self
                .projectiles
                .all()
                .iter()
                .any(|p| p.typ == ProjectileType::Fireball);
            if any_fireball || self.fireball_mode {
                self.audio.play("explosion");
            } else {
                self.audio.play("hit");
            }
        }
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
        self.audio.play("shoot");
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
                        self.audio.play("hit");
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

    fn find_spawn_location(&self, near_x: i32, near_y: i32, radius: i32) -> Option<(i32, i32)> {
        for r in 0..=radius {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() + dy.abs() != r {
                        continue;
                    }
                    let x = near_x + dx;
                    let y = near_y + dy;
                    let _sc = crate::world::worldgen::WORLD_SCALE;
                    let clear_h = 8;
                    if !self.grid.in_bounds(x, y) || !self.grid.in_bounds(x, y - clear_h) {
                        continue;
                    }
                    if !self.grid.get(x, y).is_empty() || !self.grid.get(x, y + 1).is_solid() {
                        continue;
                    }
                    let mut clear = true;
                    for k in -clear_h..=0 {
                        if !self.grid.get(x, y + k).is_empty() {
                            clear = false;
                            break;
                        }
                    }
                    if clear {
                        return Some((x, y - clear_h));
                    }
                }
            }
        }
        None
    }

    fn find_surface_spawn(
        &self,
        px: f32,
        py: f32,
        offset: i32,
        height_offset: i32,
    ) -> Option<(i32, i32)> {
        let spawn_x = px as i32 + offset;
        if !self.grid.in_bounds(spawn_x, 0) {
            return None;
        }
        let search_top = 0;
        let search_bottom = if self.grid.is_infinite() {
            py as i32 + crate::world::worldgen::WORLD_SCALE * 50
        } else {
            self.grid.height as i32 - 3
        };
        let mut surface_y = search_bottom;
        for y in search_top..=search_bottom {
            let cell = self.grid.get(spawn_x, y);
            if cell.is_solid() && cell.material != MaterialId::Stone {
                surface_y = y;
                break;
            }
        }
        let spawn_y = surface_y - height_offset;
        if !self.grid.in_bounds(spawn_x, spawn_y) {
            return None;
        }
        Some((spawn_x, spawn_y))
    }

    fn try_spawn_goblin(&mut self) {
        let max_goblins = 3usize + self.depth.min(5) as usize;
        let alive_goblins = self
            .entities
            .all()
            .iter()
            .filter(|e| e.alive && e.kind == EntityKind::Goblin)
            .count();
        if alive_goblins >= max_goblins {
            return;
        }

        let (px, py) = self.player.center(&self.entities);
        let offset = if px as i32 % 2 == 0 {
            crate::world::worldgen::WORLD_SCALE * 15
        } else {
            -(crate::world::worldgen::WORLD_SCALE * 15)
        };
        let spawn = if self.depth <= 3 {
            self.find_surface_spawn(px, py, offset, crate::world::worldgen::WORLD_SCALE * 5)
        } else {
            self.find_spawn_location(px as i32 + offset, py as i32, 3)
        };

        if let Some((spawn_x, spawn_y)) = spawn {
            let id = self.entities.spawn(EntityKind::Goblin);
            if let Some(g) = self.entities.get_mut(id) {
                g.build_humanoid(spawn_x as f32, spawn_y as f32);
                g.health += self.depth as f32 * 5.0;
                g.max_health += self.depth as f32 * 5.0;
                g.strength += self.depth;
            }
        }
    }

    fn try_spawn_slime(&mut self) {
        let max_slimes = 2usize + self.depth.min(3) as usize;
        let alive_slimes = self
            .entities
            .all()
            .iter()
            .filter(|e| e.alive && e.kind == EntityKind::Slime)
            .count();
        if alive_slimes >= max_slimes {
            return;
        }

        let (px, py) = self.player.center(&self.entities);
        let offset = if px as i32 % 2 == 0 {
            -(crate::world::worldgen::WORLD_SCALE * 18)
        } else {
            crate::world::worldgen::WORLD_SCALE * 18
        };
        let spawn = if self.depth <= 3 {
            self.find_surface_spawn(px, py, offset, crate::world::worldgen::WORLD_SCALE * 3)
        } else {
            self.find_spawn_location(px as i32 + offset, py as i32, 3)
        };

        if let Some((spawn_x, spawn_y)) = spawn {
            let id = self.entities.spawn(EntityKind::Slime);
            if let Some(s) = self.entities.get_mut(id) {
                s.build_humanoid(spawn_x as f32, spawn_y as f32);
                s.health += self.depth as f32 * 3.0;
                s.max_health += self.depth as f32 * 3.0;
            }
        }
    }
}
