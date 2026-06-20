use serde::{Deserialize, Serialize};
use crate::ai::action::AiAction;
use crate::ai::session::GameSession;
use std::io::Write;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReplayRecording {
    pub seed: u64,
    pub init_mode: String,
    pub events: Vec<ReplayEvent>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReplayEvent {
    Step { n: u32 },
    Action { tick: u64, action: AiAction },
}

pub struct ReplayRecorder {
    recording: ReplayRecording,
}

impl ReplayRecorder {
    pub fn new(seed: u64) -> Self {
        Self {
            recording: ReplayRecording {
                seed,
                init_mode: "world".to_string(),
                events: Vec::new(),
            },
        }
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.recording.seed = seed;
    }

    pub fn record_step(&mut self, n: u32) {
        self.recording.events.push(ReplayEvent::Step { n });
    }

    pub fn record_action(&mut self, tick: u64, action: AiAction) {
        self.recording.events.push(ReplayEvent::Action { tick, action });
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.recording)
            .map_err(|e| std::io::Error::other(e))?;
        std::fs::write(path, json)
    }

    pub fn recording(&self) -> &ReplayRecording {
        &self.recording
    }
}

pub struct ReplayPlayer {
    recording: ReplayRecording,
}

impl ReplayPlayer {
    pub fn load(path: &str) -> std::io::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let recording: ReplayRecording = serde_json::from_str(&data)
            .map_err(|e| std::io::Error::other(e))?;
        Ok(Self { recording })
    }

    pub fn from_recording(recording: ReplayRecording) -> Self {
        Self { recording }
    }

    pub fn play(&self) -> GameSession {
        let mut session = GameSession::new_seeded(self.recording.seed);
        session.init();
        session.set_recording(false);

        for event in &self.recording.events {
            match event {
                ReplayEvent::Step { n } => {
                    session.step(*n);
                }
                ReplayEvent::Action { tick: _, action } => {
                    session.perform_action(action);
                }
            }
        }

        session
    }

    pub fn play_until_tick(&self, target_tick: u64) -> GameSession {
        let mut session = GameSession::new_seeded(self.recording.seed);
        session.init();
        session.set_recording(false);

        for event in &self.recording.events {
            if session.tick() >= target_tick {
                break;
            }
            match event {
                ReplayEvent::Step { n } => {
                    let remaining = target_tick.saturating_sub(session.tick());
                    let steps = (*n).min(remaining as u32);
                    if steps > 0 {
                        session.step(steps);
                    }
                }
                ReplayEvent::Action { tick: _, action } => {
                    session.perform_action(action);
                }
            }
        }

        while session.tick() < target_tick {
            session.step(1);
        }

        session
    }

    pub fn recording(&self) -> &ReplayRecording {
        &self.recording
    }
}
