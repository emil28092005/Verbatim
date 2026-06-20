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
}

impl WindowInput {
    pub fn new() -> Self {
        Self {
            left: false, right: false, jump: false,
            cam_left: false, cam_right: false, cam_up: false, cam_down: false,
            quit: false, paint: None,
        }
    }

    pub fn update(&mut self, keys: &[Key]) {
        let was_jump = self.jump;
        let now_left = keys.contains(&Key::A) || keys.contains(&Key::Left);
        let now_right = keys.contains(&Key::D) || keys.contains(&Key::Right);
        let now_jump = keys.contains(&Key::W) || keys.contains(&Key::Space) || keys.contains(&Key::Up);
        let now_cam_left = keys.contains(&Key::H);
        let now_cam_right = keys.contains(&Key::L);
        let now_cam_up = keys.contains(&Key::K);
        let now_cam_down = keys.contains(&Key::J);

        self.left = now_left && !now_right;
        self.right = now_right && !now_left;
        if now_left && now_right {
            self.left = false;
            self.right = false;
        }
        self.jump = now_jump && !was_jump;
        self.cam_left = now_cam_left;
        self.cam_right = now_cam_right;
        self.cam_up = now_cam_up;
        self.cam_down = now_cam_down;
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
