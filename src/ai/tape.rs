use crate::game::Game;
use crate::render::lighting;
use std::io::Write;

pub struct TapeRecorder {
    frames: Vec<TapeFrame>,
    interval: u32,
    last_recorded_tick: u64,
    view_w: usize,
    view_h: usize,
}

#[derive(Clone, Debug)]
pub struct TapeFrame {
    pub tick: u64,
    pub depth: u32,
    pub kills: u32,
    pub score: u32,
    pub cam_x: i32,
    pub cam_y: i32,
    pub player_hp: f32,
    pub player_max_hp: f32,
    pub player_pos: [f32; 2],
    pub entity_count: usize,
    pub spectrums: Vec<(String, String)>,
}

impl TapeRecorder {
    pub fn new(interval: u32, view_w: usize, view_h: usize) -> Self {
        Self {
            frames: Vec::new(),
            interval,
            last_recorded_tick: 0,
            view_w,
            view_h,
        }
    }

    pub fn should_record(&self, tick: u64) -> bool {
        tick - self.last_recorded_tick >= self.interval as u64
    }

    pub fn record(&mut self, game: &Game) {
        if !self.should_record(game.tick) {
            return;
        }
        self.last_recorded_tick = game.tick;

        let (px, py) = game.player.center(&game.entities);
        let cam_x = px as i32 - (self.view_w as i32 / 2);
        let cam_y = py as i32 - (self.view_h as i32 / 2);

        let light = lighting::compute_lighting(
            &game.grid,
            cam_x,
            cam_y,
            self.view_w,
            self.view_h,
            lighting::ambient_light(),
        );

        let spectrums = crate::ai::spectrum::render_all_spectrums(
            &game.grid,
            &game.entities,
            Some(&light),
            cam_x,
            cam_y,
            self.view_w,
            self.view_h,
        );

        let (hp, max_hp) = game
            .player
            .entity(&game.entities)
            .map(|e| (e.health, e.max_health))
            .unwrap_or((0.0, 0.0));

        let frame = TapeFrame {
            tick: game.tick,
            depth: game.depth,
            kills: game.kills,
            score: game.score,
            cam_x,
            cam_y,
            player_hp: hp,
            player_max_hp: max_hp,
            player_pos: [px, py],
            entity_count: game.entities.all().len(),
            spectrums,
        };
        self.frames.push(frame);
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let mut f = std::fs::File::create(path)?;
        writeln!(f, "=== Verbatim Tape Recording ===")?;
        writeln!(f, "frames: {}", self.frames.len())?;
        writeln!(f, "view: {}x{}", self.view_w, self.view_h)?;
        writeln!(f)?;

        for frame in &self.frames {
            writeln!(
                f,
                "==== FRAME tick={} depth={} kills={} score={} ====",
                frame.tick, frame.depth, frame.kills, frame.score
            )?;
            writeln!(f, "cam: ({}, {})", frame.cam_x, frame.cam_y)?;
            writeln!(
                f,
                "player: hp={:.1}/{:.1} pos=({:.1},{:.1}) entities={}",
                frame.player_hp,
                frame.player_max_hp,
                frame.player_pos[0],
                frame.player_pos[1],
                frame.entity_count
            )?;
            writeln!(f)?;
            for (name, view) in &frame.spectrums {
                writeln!(f, "--- {} ---", name)?;
                write!(f, "{}", view)?;
                writeln!(f)?;
            }
        }
        Ok(())
    }

    pub fn save_json_to_file(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(
            &self
                .frames
                .iter()
                .map(|f| {
                    serde_json::json!({
                        "tick": f.tick,
                        "depth": f.depth,
                        "kills": f.kills,
                        "score": f.score,
                        "cam": [f.cam_x, f.cam_y],
                        "player": {
                            "hp": f.player_hp,
                            "max_hp": f.player_max_hp,
                            "pos": f.player_pos,
                        },
                        "entity_count": f.entity_count,
                        "spectrums": f.spectrums.iter().map(|(name, view)| {
                            serde_json::json!({
                                "name": name,
                                "view": view,
                            })
                        }).collect::<Vec<_>>(),
                    })
                })
                .collect::<Vec<_>>(),
        )
        .unwrap_or_default();

        let mut f = std::fs::File::create(path)?;
        f.write_all(json.as_bytes())?;
        Ok(())
    }
}

pub fn run_tape_mode(ticks: u32, interval: u32, output: &str, json_output: Option<&str>) {
    let mut game = Game::new();
    game.init_world();

    let view_w = 80usize;
    let view_h = 25usize;
    let mut recorder = TapeRecorder::new(interval, view_w, view_h);

    recorder.record(&game);

    for _ in 0..ticks {
        game.fixed_update();
        recorder.record(&game);
    }

    let frame_count = recorder.frame_count();
    match recorder.save_to_file(output) {
        Ok(_) => eprintln!("Tape: {} frames recorded, saved to {}", frame_count, output),
        Err(e) => eprintln!("Tape save failed: {}", e),
    }

    if let Some(json_path) = json_output {
        match recorder.save_json_to_file(json_path) {
            Ok(_) => eprintln!("Tape JSON saved to {}", json_path),
            Err(e) => eprintln!("Tape JSON save failed: {}", e),
        }
    }
}
