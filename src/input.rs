use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::sync::mpsc;
use std::thread;
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
}

const HOLD_TIMEOUT: Duration = Duration::from_millis(60);

pub struct InputHandler {
    pub paint_brush: MaterialBrush,
    held: std::collections::HashMap<HeldKey, HeldState>,
    jump_pressed: bool,
    receiver: Option<mpsc::Receiver<Event>>,
    input_thread: Option<thread::JoinHandle<()>>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            paint_brush: MaterialBrush::Sand,
            held: std::collections::HashMap::new(),
            jump_pressed: false,
            receiver: None,
            input_thread: None,
        }
    }

    pub fn start(&mut self) {
        let (tx, rx) = mpsc::channel::<Event>();
        self.receiver = Some(rx);
        self.input_thread = Some(thread::spawn(move || {
            loop {
                match event::read() {
                    Ok(ev) => {
                        if tx.send(ev).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }));
    }

    pub fn stop(&mut self) {
        self.receiver = None;
        self.input_thread = None;
        self.held.clear();
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
        self.held.insert(key, HeldState { last_seen: Instant::now() });
    }

    fn release(&mut self, key: HeldKey) {
        self.held.remove(&key);
    }

    pub fn is_held(&self, key: HeldKey) -> bool {
        self.held.contains_key(&key)
    }

    pub fn update(&mut self) -> Vec<Action> {
        let mut one_shots = Vec::new();
        self.jump_pressed = false;
        let now = Instant::now();

        self.held.retain(|_, state| now.duration_since(state.last_seen) < HOLD_TIMEOUT);

        let events: Vec<Event> = match &self.receiver {
            Some(rx) => rx.try_iter().collect(),
            None => return one_shots,
        };

        for ev in events {
            if let Event::Key(KeyEvent { code, modifiers, kind, .. }) = ev {
                let is_press = kind == KeyEventKind::Press;
                let is_repeat = kind == KeyEventKind::Repeat;
                let is_release = kind == KeyEventKind::Release;

                if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                    one_shots.push(Action::Quit);
                    continue;
                }

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

                if let Some(held_key) = Self::key_to_held(code) {
                    if is_press || is_repeat {
                        self.mark_held(held_key);
                    } else if is_release {
                        self.release(held_key);
                    }
                }
            }
        }

        one_shots
    }

    pub fn held_actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();

        let left = self.is_held(HeldKey::Left);
        let right = self.is_held(HeldKey::Right);

        if left && !right {
            actions.push(Action::MoveLeft);
        } else if right && !left {
            actions.push(Action::MoveRight);
        } else if left && right {
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
