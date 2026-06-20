use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HeldKey {
    Left,
    Right,
    Jump,
    CamLeft,
    CamRight,
    CamUp,
    CamDown,
}

pub struct InputHandler {
    pub paint_brush: MaterialBrush,
    held: Vec<HeldKey>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            paint_brush: MaterialBrush::Sand,
            held: Vec::new(),
        }
    }

    fn key_to_held(code: KeyCode) -> Option<HeldKey> {
        match code {
            KeyCode::Left | KeyCode::Char('a') => Some(HeldKey::Left),
            KeyCode::Right | KeyCode::Char('d') => Some(HeldKey::Right),
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char(' ') => Some(HeldKey::Jump),
            KeyCode::Char('h') => Some(HeldKey::CamLeft),
            KeyCode::Char('l') => Some(HeldKey::CamRight),
            KeyCode::Char('k') => Some(HeldKey::CamUp),
            KeyCode::Char('j') => Some(HeldKey::CamDown),
            _ => None,
        }
    }

    fn hold(&mut self, key: HeldKey) {
        if !self.held.contains(&key) {
            self.held.push(key);
        }
    }

    fn release(&mut self, key: HeldKey) {
        self.held.retain(|&k| k != key);
    }

    pub fn is_held(&self, key: HeldKey) -> bool {
        self.held.contains(&key)
    }

    /// Drain all pending input events, update held key state,
    /// and return one-shot actions (quit, paint, etc).
    pub fn update(&mut self) -> Action {
        let mut one_shot = Action::None;

        while event::poll(Duration::from_millis(0)).unwrap_or(false) {
            let ev = match event::read() {
                Ok(ev) => ev,
                Err(_) => continue,
            };

            if let Event::Key(KeyEvent { code, modifiers, kind, .. }) = ev {
                let is_press = kind == KeyEventKind::Press || kind == KeyEventKind::Repeat;
                let is_release = kind == KeyEventKind::Release;

                if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                    return Action::Quit;
                }

                if let Some(held_key) = Self::key_to_held(code) {
                    if is_press {
                        self.hold(held_key);
                    } else if is_release {
                        self.release(held_key);
                    }
                    continue;
                }

                if is_press {
                    match code {
                        KeyCode::Char('q') => return Action::Quit,
                        KeyCode::Char('1') => { self.paint_brush = MaterialBrush::Sand; one_shot = Action::Paint(MaterialBrush::Sand); }
                        KeyCode::Char('2') => { self.paint_brush = MaterialBrush::Water; one_shot = Action::Paint(MaterialBrush::Water); }
                        KeyCode::Char('3') => { self.paint_brush = MaterialBrush::Stone; one_shot = Action::Paint(MaterialBrush::Stone); }
                        KeyCode::Char('4') => { self.paint_brush = MaterialBrush::Lava; one_shot = Action::Paint(MaterialBrush::Lava); }
                        KeyCode::Char('5') => { self.paint_brush = MaterialBrush::Wood; one_shot = Action::Paint(MaterialBrush::Wood); }
                        KeyCode::Char('6') => { self.paint_brush = MaterialBrush::Acid; one_shot = Action::Paint(MaterialBrush::Acid); }
                        KeyCode::Char('7') => { self.paint_brush = MaterialBrush::Grass; one_shot = Action::Paint(MaterialBrush::Grass); }
                        KeyCode::Char('8') => { self.paint_brush = MaterialBrush::Dirt; one_shot = Action::Paint(MaterialBrush::Dirt); }
                        KeyCode::Char('9') => { self.paint_brush = MaterialBrush::Fire; one_shot = Action::Paint(MaterialBrush::Fire); }
                        KeyCode::Char('0') => { self.paint_brush = MaterialBrush::Flesh; one_shot = Action::Paint(MaterialBrush::Flesh); }
                        KeyCode::Char('x') => { self.paint_brush = MaterialBrush::Erase; one_shot = Action::Paint(MaterialBrush::Erase); }
                        _ => {}
                    }
                }
            }
        }

        one_shot
    }

    pub fn held_actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();
        if self.is_held(HeldKey::Left) {
            actions.push(Action::MoveLeft);
        }
        if self.is_held(HeldKey::Right) {
            actions.push(Action::MoveRight);
        }
        if self.is_held(HeldKey::Jump) {
            actions.push(Action::Jump);
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
        actions
    }

    pub fn release_all(&mut self) {
        self.held.clear();
    }

    pub fn poll(&mut self) -> Action {
        self.update()
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
