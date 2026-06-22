use std::collections::HashSet;
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct WindowInput {
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub shoot_left: bool,
    pub shoot_right: bool,
    pub shoot_up: bool,
    pub shoot_down: bool,
    pub toggle_fireball: bool,
    pub descend: bool,
    pub use_item: bool,
    pub drop_item: bool,
    pub cam_left: bool,
    pub cam_right: bool,
    pub cam_up: bool,
    pub cam_down: bool,
    pub quit: bool,
    pub toggle_audio: bool,
    pub paint: Option<u8>,
    pub mouse_x: f64,
    pub mouse_y: f64,
    pub mouse_left: bool,
    pub mouse_left_was_down: bool,
    pub mouse_right: bool,
    pub mouse_right_was_down: bool,
    pub shoot_mouse: bool,
    pub inventory_toggle: bool,
    pub inventory_click: Option<(f64, f64)>,
    pub inventory_right_click: Option<(f64, f64)>,
    jump_was_down: bool,
    inventory_tab_was_down: bool,
    shoot_left_was_down: bool,
    shoot_right_was_down: bool,
    shoot_up_was_down: bool,
    shoot_down_was_down: bool,
    fireball_was_down: bool,
    descend_was_down: bool,
    use_item_was_down: bool,
    drop_item_was_down: bool,
    audio_was_down: bool,
    down_keys: HashSet<KeyCode>,
}

impl WindowInput {
    pub fn new() -> Self {
        Self {
            left: false,
            right: false,
            jump: false,
            shoot_left: false,
            shoot_right: false,
            shoot_up: false,
            shoot_down: false,
            toggle_fireball: false,
            descend: false,
            use_item: false,
            drop_item: false,
            cam_left: false,
            cam_right: false,
            cam_up: false,
            cam_down: false,
            quit: false,
            toggle_audio: false,
            paint: None,
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_left: false,
            mouse_left_was_down: false,
            mouse_right: false,
            mouse_right_was_down: false,
            shoot_mouse: false,
            inventory_toggle: false,
            inventory_click: None,
            inventory_right_click: None,
            jump_was_down: false,
            inventory_tab_was_down: false,
            shoot_left_was_down: false,
            shoot_right_was_down: false,
            shoot_up_was_down: false,
            shoot_down_was_down: false,
            fireball_was_down: false,
            descend_was_down: false,
            use_item_was_down: false,
            drop_item_was_down: false,
            audio_was_down: false,
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

    pub fn clear_keys(&mut self) {
        self.down_keys.clear();
        self.mouse_left = false;
        self.mouse_right = false;
    }

    pub fn on_mouse_move(&mut self, x: f64, y: f64) {
        self.mouse_x = x;
        self.mouse_y = y;
    }

    pub fn on_mouse_button(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
        match (button, state) {
            (winit::event::MouseButton::Left, winit::event::ElementState::Pressed) => {
                self.mouse_left = true;
            }
            (winit::event::MouseButton::Left, winit::event::ElementState::Released) => {
                self.mouse_left = false;
            }
            (winit::event::MouseButton::Right, winit::event::ElementState::Pressed) => {
                self.mouse_right = true;
            }
            (winit::event::MouseButton::Right, winit::event::ElementState::Released) => {
                self.mouse_right = false;
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {
        let keys = &self.down_keys;

        let now_left = keys.contains(&KeyCode::KeyA) || keys.contains(&KeyCode::ArrowLeft);
        let now_right = keys.contains(&KeyCode::KeyD) || keys.contains(&KeyCode::ArrowRight);
        let now_jump = keys.contains(&KeyCode::KeyW)
            || keys.contains(&KeyCode::Space)
            || keys.contains(&KeyCode::ArrowUp);

        self.left = now_left && !now_right;
        self.right = now_right && !now_left;

        self.jump = now_jump && !self.jump_was_down;
        self.jump_was_down = now_jump;

        let now_shoot_left = keys.contains(&KeyCode::KeyH);
        let now_shoot_right = keys.contains(&KeyCode::KeyL);
        let now_shoot_up = keys.contains(&KeyCode::KeyK);
        let now_shoot_down = keys.contains(&KeyCode::KeyJ);
        let now_fireball = keys.contains(&KeyCode::KeyF);

        self.shoot_left = now_shoot_left && !self.shoot_left_was_down;
        self.shoot_right = now_shoot_right && !self.shoot_right_was_down;
        self.shoot_up = now_shoot_up && !self.shoot_up_was_down;
        self.shoot_down = now_shoot_down && !self.shoot_down_was_down;
        self.shoot_left_was_down = now_shoot_left;
        self.shoot_right_was_down = now_shoot_right;
        self.shoot_up_was_down = now_shoot_up;
        self.shoot_down_was_down = now_shoot_down;

        self.toggle_fireball = now_fireball && !self.fireball_was_down;
        self.fireball_was_down = now_fireball;

        let now_descend = keys.contains(&KeyCode::Period);
        self.descend = now_descend && !self.descend_was_down;
        self.descend_was_down = now_descend;

        let now_use_item = keys.contains(&KeyCode::KeyE);
        self.use_item = now_use_item && !self.use_item_was_down;
        self.use_item_was_down = now_use_item;

        let now_drop_item = keys.contains(&KeyCode::KeyR);
        self.drop_item = now_drop_item && !self.drop_item_was_down;
        self.drop_item_was_down = now_drop_item;

        self.shoot_mouse = self.mouse_left && !self.mouse_left_was_down;
        self.mouse_left_was_down = self.mouse_left;

        let now_tab = keys.contains(&KeyCode::Tab);
        self.inventory_toggle = now_tab && !self.inventory_tab_was_down;
        self.inventory_tab_was_down = now_tab;

        self.inventory_click = None;
        self.inventory_right_click = None;
        if self.mouse_left && !self.mouse_left_was_down {
            self.inventory_click = Some((self.mouse_x, self.mouse_y));
        }
        if self.mouse_right && !self.mouse_right_was_down {
            self.inventory_right_click = Some((self.mouse_x, self.mouse_y));
        }
        self.mouse_right_was_down = self.mouse_right;

        self.cam_left = keys.contains(&KeyCode::KeyY);
        self.cam_right = keys.contains(&KeyCode::KeyU);
        self.cam_up = keys.contains(&KeyCode::KeyI);
        self.cam_down = keys.contains(&KeyCode::KeyO);
        self.quit = keys.contains(&KeyCode::KeyQ) || keys.contains(&KeyCode::Escape);
        let now_audio = keys.contains(&KeyCode::KeyM);
        self.toggle_audio = now_audio && !self.audio_was_down;
        self.audio_was_down = now_audio;

        self.paint = None;
        if keys.contains(&KeyCode::Digit1) {
            self.paint = Some(1);
        } else if keys.contains(&KeyCode::Digit2) {
            self.paint = Some(2);
        } else if keys.contains(&KeyCode::Digit3) {
            self.paint = Some(3);
        } else if keys.contains(&KeyCode::Digit4) {
            self.paint = Some(4);
        } else if keys.contains(&KeyCode::Digit5) {
            self.paint = Some(5);
        } else if keys.contains(&KeyCode::Digit6) {
            self.paint = Some(6);
        } else if keys.contains(&KeyCode::Digit7) {
            self.paint = Some(7);
        } else if keys.contains(&KeyCode::Digit8) {
            self.paint = Some(8);
        } else if keys.contains(&KeyCode::Digit9) {
            self.paint = Some(9);
        } else if keys.contains(&KeyCode::Digit0) {
            self.paint = Some(0);
        } else if keys.contains(&KeyCode::KeyX) {
            self.paint = Some(99);
        }
    }
}
