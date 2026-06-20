use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    MoveLeft,
    MoveRight,
    Jump,
    MoveCameraUp,
    MoveCameraDown,
    MoveCameraLeft,
    MoveCameraRight,
    Paint(MaterialBrush),
    Quit,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MaterialBrush {
    Sand,
    Water,
    Stone,
    Lava,
    Wood,
    Acid,
    Grass,
    Dirt,
    Fire,
    Flesh,
    Erase,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeldKey {
    Left,
    Right,
    CamLeft,
    CamRight,
    CamUp,
    CamDown,
}

struct HeldState {
    last_seen: Instant,
    just_pressed: bool,
}

const HOLD_TIMEOUT: Duration = Duration::from_millis(80);

pub struct InputHandler {
    pub paint_brush: MaterialBrush,
    held: std::collections::HashMap<HeldKey, HeldState>,
    jump_pressed: bool,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            paint_brush: MaterialBrush::Sand,
            held: std::collections::HashMap::new(),
            jump_pressed: false,
        }
    }

    fn key_to_held(code: KeyCode) -> Option<HeldKey> {
        match code {
            KeyCode::Left | KeyCode::Char('a') => Some(HeldKey::Left),
            KeyCode::Right | KeyCode::Char('d') => Some(HeldKey::Right),
            KeyCode::Char('h') => Some(HeldKey::CamLeft),
            KeyCode::Char('l') => Some(HeldKey::CamRight),
            KeyCode::Char('k') => Some(HeldKey::CamUp),
            KeyCode::Char('j') => Some(HeldKey::CamDown),
            _ => None,
        }
    }

    fn mark_held(&mut self, key: HeldKey) {
        let state = self.held.entry(key).or_insert(HeldState { last_seen: Instant::now(), just_pressed: false });
        state.last_seen = Instant::now();
    }

    fn mark_pressed(&mut self, key: HeldKey) {
        self.held.insert(key, HeldState { last_seen: Instant::now(), just_pressed: true });
    }

    fn release(&mut self, key: HeldKey) {
        self.held.remove(&key);
    }

    pub fn is_held(&self, key: HeldKey) -> bool {
        self.held.contains_key(&key)
    }

    /// Drain all pending input events, update held key state.
    /// Returns one-shot actions (quit, paint, jump-on-press).
    pub fn update(&mut self) -> Vec<Action> {
        let mut one_shots = Vec::new();
        self.jump_pressed = false;
        let now = Instant::now();

        // Expire stale held keys (no Repeat received within timeout)
        self.held.retain(|_, state| now.duration_since(state.last_seen) < HOLD_TIMEOUT);

        while event::poll(Duration::from_millis(0)).unwrap_or(false) {
            let ev = match event::read() {
                Ok(ev) => ev,
                Err(_) => continue,
            };

            if let Event::Key(KeyEvent { code, modifiers, kind, .. }) = ev {
                let is_press = kind == KeyEventKind::Press;
                let is_repeat = kind == KeyEventKind::Repeat;
                let is_release = kind == KeyEventKind::Release;

                if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                    one_shots.push(Action::Quit);
                    continue;
                }

                // Jump: only on initial press, not repeat
                if is_press {
                    match code {
                        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char(' ') => {
                            self.jump_pressed = true;
                            continue;
                        }
                        KeyCode::Char('q') => {
                            one_shots.push(Action::Quit);
                            continue;
                        }
                        KeyCode::Char('1') => { self.paint_brush = MaterialBrush::Sand; one_shots.push(Action::Paint(MaterialBrush::Sand)); continue; }
                        KeyCode::Char('2') => { self.paint_brush = MaterialBrush::Water; one_shots.push(Action::Paint(MaterialBrush::Water)); continue; }
                        KeyCode::Char('3') => { self.paint_brush = MaterialBrush::Stone; one_shots.push(Action::Paint(MaterialBrush::Stone)); continue; }
                        KeyCode::Char('4') => { self.paint_brush = MaterialBrush::Lava; one_shots.push(Action::Paint(MaterialBrush::Lava)); continue; }
                        KeyCode::Char('5') => { self.paint_brush = MaterialBrush::Wood; one_shots.push(Action::Paint(MaterialBrush::Wood)); continue; }
                        KeyCode::Char('6') => { self.paint_brush = MaterialBrush::Acid; one_shots.push(Action::Paint(MaterialBrush::Acid)); continue; }
                        KeyCode::Char('7') => { self.paint_brush = MaterialBrush::Grass; one_shots.push(Action::Paint(MaterialBrush::Grass)); continue; }
                        KeyCode::Char('8') => { self.paint_brush = MaterialBrush::Dirt; one_shots.push(Action::Paint(MaterialBrush::Dirt)); continue; }
                        KeyCode::Char('9') => { self.paint_brush = MaterialBrush::Fire; one_shots.push(Action::Paint(MaterialBrush::Fire)); continue; }
                        KeyCode::Char('0') => { self.paint_brush = MaterialBrush::Flesh; one_shots.push(Action::Paint(MaterialBrush::Flesh)); continue; }
                        KeyCode::Char('x') => { self.paint_brush = MaterialBrush::Erase; one_shots.push(Action::Paint(MaterialBrush::Erase)); continue; }
                        _ => {}
                    }
                }

                // Movement keys: track held state
                if let Some(held_key) = Self::key_to_held(code) {
                    if is_press {
                        self.mark_pressed(held_key);
                    } else if is_repeat {
                        self.mark_held(held_key);
                    } else if is_release {
                        self.release(held_key);
                    }
                }
            }
        }

        // Clear just_pressed flags
        for state in self.held.values_mut() {
            state.just_pressed = false;
        }

        one_shots
    }

    pub fn held_actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();

        // Left/Right: last pressed wins if both held
        let left = self.is_held(HeldKey::Left);
        let right = self.is_held(HeldKey::Right);

        if left && !right {
            actions.push(Action::MoveLeft);
        } else if right && !left {
            actions.push(Action::MoveRight);
        } else if left && right {
            // Both held — check which was pressed more recently
            let left_time = self.held.get(&HeldKey::Left).map(|s| s.last_seen);
            let right_time = self.held.get(&HeldKey::Right).map(|s| s.last_seen);
            match (left_time, right_time) {
                (Some(lt), Some(rt)) => {
                    if lt > rt {
                        actions.push(Action::MoveLeft);
                    } else {
                        actions.push(Action::MoveRight);
                    }
                }
                _ => {}
            }
        }

        if self.is_held(HeldKey::CamLeft) { actions.push(Action::MoveCameraLeft); }
        if self.is_held(HeldKey::CamRight) { actions.push(Action::MoveCameraRight); }
        if self.is_held(HeldKey::CamUp) { actions.push(Action::MoveCameraUp); }
        if self.is_held(HeldKey::CamDown) { actions.push(Action::MoveCameraDown); }

        actions
    }

    pub fn jump_requested(&self) -> bool {
        self.jump_pressed
    }

    pub fn release_all(&mut self) {
        self.held.clear();
        self.jump_pressed = false;
    }

    pub fn poll(&mut self) -> Action {
        let actions = self.update();
        actions.into_iter().next().unwrap_or(Action::None)
    }
}

impl MaterialBrush {
    pub fn to_material(&self) -> Option<crate::world::cell::MaterialId> {
        use crate::world::cell::MaterialId;
        match self {
            MaterialBrush::Sand => Some(MaterialId::Sand),
            MaterialBrush::Water => Some(MaterialId::Water),
            MaterialBrush::Stone => Some(MaterialId::Stone),
            MaterialBrush::Lava => Some(MaterialId::Lava),
            MaterialBrush::Wood => Some(MaterialId::Wood),
            MaterialBrush::Acid => Some(MaterialId::Acid),
            MaterialBrush::Grass => Some(MaterialId::Grass),
            MaterialBrush::Dirt => Some(MaterialId::Dirt),
            MaterialBrush::Fire => Some(MaterialId::Fire),
            MaterialBrush::Flesh => Some(MaterialId::Flesh),
            MaterialBrush::Erase => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            MaterialBrush::Sand => "Sand",
            MaterialBrush::Water => "Water",
            MaterialBrush::Stone => "Stone",
            MaterialBrush::Lava => "Lava",
            MaterialBrush::Wood => "Wood",
            MaterialBrush::Acid => "Acid",
            MaterialBrush::Grass => "Grass",
            MaterialBrush::Dirt => "Dirt",
            MaterialBrush::Fire => "Fire",
            MaterialBrush::Flesh => "Flesh",
            MaterialBrush::Erase => "Erase",
        }
    }
}
