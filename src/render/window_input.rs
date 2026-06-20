use minifb::Key;

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
}

impl WindowInput {
    pub fn new() -> Self {
        Self {
            left: false, right: false, jump: false,
            cam_left: false, cam_right: false, cam_up: false, cam_down: false,
            quit: false, paint: None,
            jump_was_down: false,
        }
    }

    pub fn update(&mut self, keys: &[Key]) {
        let now_left = keys.contains(&Key::A) || keys.contains(&Key::Left);
        let now_right = keys.contains(&Key::D) || keys.contains(&Key::Right);
        let now_jump = keys.contains(&Key::W) || keys.contains(&Key::Space) || keys.contains(&Key::Up);

        self.left = now_left && !now_right;
        self.right = now_right && !now_left;

        self.jump = now_jump && !self.jump_was_down;
        self.jump_was_down = now_jump;

        self.cam_left = keys.contains(&Key::H);
        self.cam_right = keys.contains(&Key::L);
        self.cam_up = keys.contains(&Key::K);
        self.cam_down = keys.contains(&Key::J);
        self.quit = keys.contains(&Key::Q) || keys.contains(&Key::Escape);

        self.paint = None;
        if keys.contains(&Key::Key1) { self.paint = Some(1); }
        else if keys.contains(&Key::Key2) { self.paint = Some(2); }
        else if keys.contains(&Key::Key3) { self.paint = Some(3); }
        else if keys.contains(&Key::Key4) { self.paint = Some(4); }
        else if keys.contains(&Key::Key5) { self.paint = Some(5); }
        else if keys.contains(&Key::Key6) { self.paint = Some(6); }
        else if keys.contains(&Key::Key7) { self.paint = Some(7); }
        else if keys.contains(&Key::Key8) { self.paint = Some(8); }
        else if keys.contains(&Key::Key9) { self.paint = Some(9); }
        else if keys.contains(&Key::Key0) { self.paint = Some(0); }
        else if keys.contains(&Key::X) { self.paint = Some(99); }
    }
}
