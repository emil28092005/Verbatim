use crate::ai::action::AiAction;
use crate::ai::replay::ReplayRecorder;
use crate::ai::state::{build_game_state, CellInfo, EntityInfo, GameState, render_view};
use crate::game::Game;
use crate::world::cell::MaterialId;
use crate::world::grid::Grid;

pub struct GameSession {
    pub game: Game,
    pub seed: u64,
    pub recorder: Option<ReplayRecorder>,
    pub view_width: usize,
    pub view_height: usize,
}

impl GameSession {
    pub fn new() -> Self {
        Self {
            game: Game::new(),
            seed: 42,
            recorder: None,
            view_width: 80,
            view_height: 25,
        }
    }

    pub fn new_seeded(seed: u64) -> Self {
        let mut s = Self::new();
        s.seed = seed;
        s
    }

    pub fn init(&mut self) {
        self.game.init_world();
        if let Some(ref mut r) = self.recorder {
            r.set_seed(self.seed);
        }
    }

    pub fn init_empty(&mut self) {
        self.game.grid.fill_border(MaterialId::Stone);
        let cx = (self.game.grid.width / 2) as f32;
        let cy = (self.game.grid.height / 2) as f32;
        self.game.player.spawn_at(&mut self.game.entities, cx, cy);
        let (px, py) = self.game.player.center(&self.game.entities);
        self.game.center_camera_on(px, py);
    }

    pub fn step(&mut self, n: u32) {
        for _ in 0..n {
            self.game.fixed_update();
        }
        if let Some(ref mut r) = self.recorder {
            r.record_step(n);
        }
    }

    pub fn perform_action(&mut self, action: &AiAction) {
        action.execute(&mut self.game);
        if let Some(ref mut r) = self.recorder {
            r.record_action(self.game.tick, action.clone());
        }
    }

    pub fn perform_action_and_step(&mut self, action: &AiAction, steps: u32) {
        self.perform_action(action);
        self.step(steps);
    }

    pub fn get_state(&self) -> GameState {
        build_game_state(&self.game, self.view_width, self.view_height)
    }

    pub fn get_view(&self, w: usize, h: usize) -> String {
        let (px, py) = self.game.player.center(&self.game.entities);
        let cam_x = px as i32 - (w as i32 / 2);
        let cam_y = py as i32 - (h as i32 / 2);
        render_view(&self.game.grid, &self.game.entities, cam_x, cam_y, w, h)
    }

    pub fn get_view_at(&self, cam_x: i32, cam_y: i32, w: usize, h: usize) -> String {
        render_view(&self.game.grid, &self.game.entities, cam_x, cam_y, w, h)
    }

    pub fn get_cell(&self, x: i32, y: i32) -> CellInfo {
        CellInfo::from_grid(&self.game.grid, x, y)
    }

    pub fn get_region(&self, x: i32, y: i32, w: i32, h: i32) -> Vec<CellInfo> {
        let mut cells = Vec::with_capacity((w * h) as usize);
        for dy in 0..h {
            for dx in 0..w {
                cells.push(CellInfo::from_grid(&self.game.grid, x + dx, y + dy));
            }
        }
        cells
    }

    pub fn get_entities(&self) -> Vec<EntityInfo> {
        self.game.entities.all().iter().map(|e| {
            crate::ai::state::entity_info(e)
        }).collect()
    }

    pub fn get_player(&self) -> Option<EntityInfo> {
        self.game.player.entity(&self.game.entities).map(|e| crate::ai::state::entity_info(e))
    }

    pub fn count_material_in_region(&self, x: i32, y: i32, w: i32, h: i32, material: &str) -> usize {
        let target = crate::ai::state::material_from_name(material);
        if target.is_none() {
            return 0;
        }
        let target = target.unwrap();
        let mut count = 0;
        for dy in 0..h {
            for dx in 0..w {
                let cell = self.game.grid.get(x + dx, y + dy);
                if cell.material == target {
                    count += 1;
                }
            }
        }
        count
    }

    pub fn find_material(&self, material: &str) -> Option<(i32, i32)> {
        let target = crate::ai::state::material_from_name(material)?;
        for y in 0..self.game.grid.height as i32 {
            for x in 0..self.game.grid.width as i32 {
                if self.game.grid.get(x, y).material == target {
                    return Some((x, y));
                }
            }
        }
        None
    }

    pub fn set_recording(&mut self, on: bool) {
        if on {
            if self.recorder.is_none() {
                self.recorder = Some(ReplayRecorder::new(self.seed));
            }
        } else {
            self.recorder = None;
        }
    }

    pub fn is_recording(&self) -> bool {
        self.recorder.is_some()
    }

    pub fn save_replay(&self, path: &str) -> std::io::Result<()> {
        if let Some(ref r) = self.recorder {
            r.save(path)?;
        }
        Ok(())
    }

    pub fn clear_area(&mut self, x: i32, y: i32, w: i32, h: i32) {
        for dy in 0..h {
            for dx in 0..w {
                self.game.grid.set(x + dx, y + dy, crate::world::cell::Cell::empty());
            }
        }
    }

    pub fn grid(&self) -> &Grid {
        &self.game.grid
    }

    pub fn grid_mut(&mut self) -> &mut Grid {
        &mut self.game.grid
    }

    pub fn tick(&self) -> u64 {
        self.game.tick
    }
}

impl Default for GameSession {
    fn default() -> Self {
        Self::new()
    }
}
