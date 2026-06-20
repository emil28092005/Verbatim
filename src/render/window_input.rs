use std::collections::HashSet;
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct WindowInput {
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub cam_left: bool,
    pub cam_right: bool,
    pub cam_up: bool,
    pub cam_down: bool,
    pub quit: bool,
    pub paint: Option<u8>,
    jump_was_down: bool,
    down_keys: HashSet<KeyCode>,
}

impl WindowInput {
    pub fn new() -> Self {
        Self {
            left: false, right: false, jump: false,
            cam_left: false, cam_right: false, cam_up: false, cam_down: false,
            quit: false, paint: None,
            jump_was_down: false,
            down_keys: HashSet::new(),
        }
    }

    pub fn on_key_event(&mut self, key: PhysicalKey, state: winit::event::ElementState) {
        let code = match key {
            PhysicalKey::Code(c) => c,
            _ => return,
        };

        match state {
            winit::event::ElementState::Pressed => {
                self.down_keys.insert(code);
            }
            winit::event::ElementState::Released => {
                self.down_keys.remove(&code);
            }
        }
    }

    pub fn update(&mut self) {
        let keys = &self.down_keys;

        let now_left = keys.contains(&KeyCode::KeyA) || keys.contains(&KeyCode::ArrowLeft);
        let now_right = keys.contains(&KeyCode::KeyD) || keys.contains(&KeyCode::ArrowRight);
        let now_jump = keys.contains(&KeyCode::KeyW) || keys.contains(&KeyCode::Space) || keys.contains(&KeyCode::ArrowUp);

        self.left = now_left && !now_right;
        self.right = now_right && !now_left;

        self.jump = now_jump && !self.jump_was_down;
        self.jump_was_down = now_jump;

        self.cam_left = keys.contains(&KeyCode::KeyH);
        self.cam_right = keys.contains(&KeyCode::KeyL);
        self.cam_up = keys.contains(&KeyCode::KeyK);
        self.cam_down = keys.contains(&KeyCode::KeyJ);
        self.quit = keys.contains(&KeyCode::KeyQ) || keys.contains(&KeyCode::Escape);

        self.paint = None;
        if keys.contains(&KeyCode::Digit1) { self.paint = Some(1); }
        else if keys.contains(&KeyCode::Digit2) { self.paint = Some(2); }
        else if keys.contains(&KeyCode::Digit3) { self.paint = Some(3); }
        else if keys.contains(&KeyCode::Digit4) { self.paint = Some(4); }
        else if keys.contains(&KeyCode::Digit5) { self.paint = Some(5); }
        else if keys.contains(&KeyCode::Digit6) { self.paint = Some(6); }
        else if keys.contains(&KeyCode::Digit7) { self.paint = Some(7); }
        else if keys.contains(&KeyCode::Digit8) { self.paint = Some(8); }
        else if keys.contains(&KeyCode::Digit9) { self.paint = Some(9); }
        else if keys.contains(&KeyCode::Digit0) { self.paint = Some(0); }
        else if keys.contains(&KeyCode::KeyX) { self.paint = Some(99); }
    }
}
