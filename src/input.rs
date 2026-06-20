use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

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

pub struct InputHandler {
    pub paint_brush: MaterialBrush,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            paint_brush: MaterialBrush::Sand,
        }
    }

    pub fn poll(&mut self) -> Action {
        if !event::poll(Duration::from_millis(0)).unwrap_or(false) {
            return Action::None;
        }
        match event::read() {
            Ok(Event::Key(KeyEvent { code, modifiers, .. })) => {
                if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                    return Action::Quit;
                }
                match code {
                    KeyCode::Char('q') => Action::Quit,
                    KeyCode::Left | KeyCode::Char('a') => Action::MoveLeft,
                    KeyCode::Right | KeyCode::Char('d') => Action::MoveRight,
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char(' ') => Action::Jump,
                    KeyCode::Char('h') => Action::MoveCameraLeft,
                    KeyCode::Char('l') => Action::MoveCameraRight,
                    KeyCode::Char('k') => Action::MoveCameraUp,
                    KeyCode::Char('j') => Action::MoveCameraDown,
                    KeyCode::Char('1') => { self.paint_brush = MaterialBrush::Sand; Action::Paint(MaterialBrush::Sand) }
                    KeyCode::Char('2') => { self.paint_brush = MaterialBrush::Water; Action::Paint(MaterialBrush::Water) }
                    KeyCode::Char('3') => { self.paint_brush = MaterialBrush::Stone; Action::Paint(MaterialBrush::Stone) }
                    KeyCode::Char('4') => { self.paint_brush = MaterialBrush::Lava; Action::Paint(MaterialBrush::Lava) }
                    KeyCode::Char('5') => { self.paint_brush = MaterialBrush::Wood; Action::Paint(MaterialBrush::Wood) }
                    KeyCode::Char('6') => { self.paint_brush = MaterialBrush::Acid; Action::Paint(MaterialBrush::Acid) }
                    KeyCode::Char('7') => { self.paint_brush = MaterialBrush::Grass; Action::Paint(MaterialBrush::Grass) }
                    KeyCode::Char('8') => { self.paint_brush = MaterialBrush::Dirt; Action::Paint(MaterialBrush::Dirt) }
                    KeyCode::Char('9') => { self.paint_brush = MaterialBrush::Fire; Action::Paint(MaterialBrush::Fire) }
                    KeyCode::Char('0') => { self.paint_brush = MaterialBrush::Flesh; Action::Paint(MaterialBrush::Flesh) }
                    KeyCode::Char('x') => { self.paint_brush = MaterialBrush::Erase; Action::Paint(MaterialBrush::Erase) }
                    _ => Action::None,
                }
            }
            _ => Action::None,
        }
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
