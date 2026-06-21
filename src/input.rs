use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    MoveLeft,
    MoveRight,
    Jump,
    ShootLeft,
    ShootRight,
    ShootUp,
    ShootDown,
    ToggleFireball,
    MoveCameraUp,
    MoveCameraDown,
    MoveCameraLeft,
    MoveCameraRight,
    Paint(MaterialBrush),
    Descend,
    UseItem,
    DropItem,
    Quit,
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
    ShootLeft,
    ShootRight,
    ShootUp,
    ShootDown,
}

struct HeldState {
    last_seen: Instant,
    got_release: bool,
}

const FALLBACK_TIMEOUT: Duration = Duration::from_millis(150);

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
        self.input_thread = Some(thread::spawn(move || loop {
            match event::read() {
                Ok(ev) => {
                    if tx.send(ev).is_err() {
                        break;
                    }
                }
                Err(_) => break,
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
            KeyCode::Char('y') => Some(HeldKey::CamLeft),
            KeyCode::Char('u') => Some(HeldKey::CamRight),
            KeyCode::Char('i') => Some(HeldKey::CamUp),
            KeyCode::Char('o') => Some(HeldKey::CamDown),
            KeyCode::Char('h') => Some(HeldKey::ShootLeft),
            KeyCode::Char('l') => Some(HeldKey::ShootRight),
            KeyCode::Char('k') => Some(HeldKey::ShootUp),
            KeyCode::Char('j') => Some(HeldKey::ShootDown),
            _ => None,
        }
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

        let events: Vec<Event> = match &self.receiver {
            Some(rx) => rx.try_iter().collect(),
            None => return one_shots,
        };

        for ev in events {
            if let Event::Key(KeyEvent {
                code,
                modifiers,
                kind,
                ..
            }) = ev
            {
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
                        KeyCode::Char('1') => {
                            self.paint_brush = MaterialBrush::Sand;
                            one_shots.push(Action::Paint(MaterialBrush::Sand));
                            continue;
                        }
                        KeyCode::Char('2') => {
                            self.paint_brush = MaterialBrush::Water;
                            one_shots.push(Action::Paint(MaterialBrush::Water));
                            continue;
                        }
                        KeyCode::Char('3') => {
                            self.paint_brush = MaterialBrush::Stone;
                            one_shots.push(Action::Paint(MaterialBrush::Stone));
                            continue;
                        }
                        KeyCode::Char('4') => {
                            self.paint_brush = MaterialBrush::Lava;
                            one_shots.push(Action::Paint(MaterialBrush::Lava));
                            continue;
                        }
                        KeyCode::Char('5') => {
                            self.paint_brush = MaterialBrush::Wood;
                            one_shots.push(Action::Paint(MaterialBrush::Wood));
                            continue;
                        }
                        KeyCode::Char('6') => {
                            self.paint_brush = MaterialBrush::Acid;
                            one_shots.push(Action::Paint(MaterialBrush::Acid));
                            continue;
                        }
                        KeyCode::Char('7') => {
                            self.paint_brush = MaterialBrush::Grass;
                            one_shots.push(Action::Paint(MaterialBrush::Grass));
                            continue;
                        }
                        KeyCode::Char('8') => {
                            self.paint_brush = MaterialBrush::Dirt;
                            one_shots.push(Action::Paint(MaterialBrush::Dirt));
                            continue;
                        }
                        KeyCode::Char('9') => {
                            self.paint_brush = MaterialBrush::Fire;
                            one_shots.push(Action::Paint(MaterialBrush::Fire));
                            continue;
                        }
                        KeyCode::Char('0') => {
                            self.paint_brush = MaterialBrush::Flesh;
                            one_shots.push(Action::Paint(MaterialBrush::Flesh));
                            continue;
                        }
                        KeyCode::Char('x') => {
                            self.paint_brush = MaterialBrush::Erase;
                            one_shots.push(Action::Paint(MaterialBrush::Erase));
                            continue;
                        }
                        KeyCode::Char('f') => {
                            one_shots.push(Action::ToggleFireball);
                            continue;
                        }
                        KeyCode::Char('>') => {
                            one_shots.push(Action::Descend);
                            continue;
                        }
                        KeyCode::Char('e') => {
                            one_shots.push(Action::UseItem);
                            continue;
                        }
                        KeyCode::Char('r') => {
                            one_shots.push(Action::DropItem);
                            continue;
                        }
                        KeyCode::Char('k') => {
                            one_shots.push(Action::ShootUp);
                            continue;
                        }
                        KeyCode::Char('j') => {
                            one_shots.push(Action::ShootDown);
                            continue;
                        }
                        KeyCode::Char('h') => {
                            one_shots.push(Action::ShootLeft);
                            continue;
                        }
                        KeyCode::Char('l') => {
                            one_shots.push(Action::ShootRight);
                            continue;
                        }
                        _ => {}
                    }
                }

                if let Some(held_key) = Self::key_to_held(code) {
                    if is_press || is_repeat {
                        let prev_got_release = self
                            .held
                            .get(&held_key)
                            .map(|s| s.got_release)
                            .unwrap_or(false);
                        self.held.insert(
                            held_key,
                            HeldState {
                                last_seen: Instant::now(),
                                got_release: prev_got_release,
                            },
                        );
                    } else if is_release {
                        self.release(held_key);
                        for state in self.held.values_mut() {
                            state.got_release = true;
                        }
                    }
                }
            }
        }

        self.held.retain(|_, state| {
            if state.got_release {
                true
            } else {
                now.duration_since(state.last_seen) < FALLBACK_TIMEOUT
            }
        });

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

        if self.is_held(HeldKey::CamLeft) {
            actions.push(Action::MoveCameraLeft);
        }
        if self.is_held(HeldKey::CamRight) {
            actions.push(Action::MoveCameraRight);
        }
        if self.is_held(HeldKey::CamUp) {
            actions.push(Action::MoveCameraUp);
        }
        if self.is_held(HeldKey::CamDown) {
            actions.push(Action::MoveCameraDown);
        }

        if self.is_held(HeldKey::ShootLeft) {
            actions.push(Action::ShootLeft);
        }
        if self.is_held(HeldKey::ShootRight) {
            actions.push(Action::ShootRight);
        }
        if self.is_held(HeldKey::ShootUp) {
            actions.push(Action::ShootUp);
        }
        if self.is_held(HeldKey::ShootDown) {
            actions.push(Action::ShootDown);
        }

        actions
    }

    pub fn jump_requested(&self) -> bool {
        self.jump_pressed
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
}
