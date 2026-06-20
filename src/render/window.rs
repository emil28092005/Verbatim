use minifb::{Key, KeyRepeat, Window, WindowOptions};
use fontdue::{Font, FontSettings};
use crate::entity::{EntityManager, EntityKind};
use crate::world::cell::MaterialId;
use crate::world::grid::Grid;
use crate::world::material::MaterialRegistry;

const CHAR_W: usize = 8;
const CHAR_H: usize = 16;

pub struct WindowRenderer {
    window: Window,
    font: Font,
    glyph_cache: std::collections::HashMap<(char, [u8; 3]), Vec<u32>>,
    width: usize,
    height: usize,
    pixels: Vec<u32>,
}

impl WindowRenderer {
    pub fn new() -> Self {
        let font_bytes: &[u8] = include_bytes!("../../assets/DejaVuSansMono.ttf");
        let font = Font::from_bytes(font_bytes, FontSettings {
            collection_index: 0,
            scale: CHAR_H as f32,
            load_substitutions: false,
        }).expect("Failed to load font");

        let width = 160;
        let height = 50;

        let window = Window::new(
            "Verbatim",
            width * CHAR_W,
            height * CHAR_H,
            WindowOptions {
                resize: true,
                ..WindowOptions::default()
            },
        ).expect("Failed to create window");

        Self {
            window,
            font,
            glyph_cache: std::collections::HashMap::new(),
            width,
            height,
            pixels: vec![0u32; width * CHAR_W * height * CHAR_H],
        }
    }

    fn rgb_to_u32(r: u8, g: u8, b: u8) -> u32 {
        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }

    fn get_glyph(&mut self, ch: char, fg: [u8; 3]) -> Vec<u32> {
        let key = (ch, fg);
        if !self.glyph_cache.contains_key(&key) {
            let (metrics, bitmap) = self.font.rasterize(ch, CHAR_H as f32);
            let gw = metrics.width;
            let gh = metrics.height;
            let mut pixels = vec![0u32; CHAR_W * CHAR_H];
            for y in 0..gh.min(CHAR_H) {
                for x in 0..gw.min(CHAR_W) {
                    let alpha = bitmap[y * gw + x] as f32 / 255.0;
                    if alpha > 0.01 {
                        let px = (x as i32 + metrics.xmin).max(0) as usize;
                        let py = (y as i32 + CHAR_H as i32 - gh as i32 - metrics.ymin).max(0) as usize;
                        if px < CHAR_W && py < CHAR_H {
                            pixels[py * CHAR_W + px] = Self::rgb_to_u32(
                                (fg[0] as f32 * alpha) as u8,
                                (fg[1] as f32 * alpha) as u8,
                                (fg[2] as f32 * alpha) as u8,
                            );
                        }
                    }
                }
            }
            self.glyph_cache.insert(key, pixels);
        }
        self.glyph_cache[&key].clone()
    }

    fn draw_cell(&mut self, col: usize, row: usize, ch: char, fg: [u8; 3], bg: [u8; 3]) {
        let bg_u32 = Self::rgb_to_u32(bg[0], bg[1], bg[2]);
        let glyph = self.get_glyph(ch, fg);

        let base_x = col * CHAR_W;
        let base_y = row * CHAR_H;
        let screen_w = self.width * CHAR_W;
        let screen_h = self.height * CHAR_H;

        for y in 0..CHAR_H {
            for x in 0..CHAR_W {
                let px = base_x + x;
                let py = base_y + y;
                if px >= screen_w || py >= screen_h {
                    continue;
                }
                let gp = glyph[y * CHAR_W + x];
                if gp != 0 {
                    self.pixels[py * screen_w + px] = gp;
                } else {
                    self.pixels[py * screen_w + px] = bg_u32;
                }
            }
        }
    }

    pub fn render(&mut self, grid: &Grid, entities: &EntityManager, cam_x: i32, cam_y: i32) {
        let reg = MaterialRegistry::instance();

        let mut entity_map = std::collections::HashMap::new();
        for e in entities.all() {
            for b in &e.bodies {
                if !b.alive { continue; }
                let sx = b.x as i32 - cam_x;
                let sy = b.y as i32 - cam_y;
                if sx >= 0 && sx < self.width as i32 && sy >= 0 && sy < self.height as i32 {
                    let ch = match e.kind {
                        EntityKind::Player if e.alive => '@',
                        EntityKind::Goblin if e.alive => 'g',
                        _ => '%',
                    };
                    let fg = if e.on_fire {
                        [255, 160, 40]
                    } else if !e.alive {
                        [100, 60, 60]
                    } else {
                        match e.kind {
                            EntityKind::Player => [255, 255, 100],
                            EntityKind::Goblin => [100, 220, 100],
                            _ => [180, 50, 50],
                        }
                    };
                    entity_map.insert((sx, sy), (ch, fg));
                }
            }
        }

        for dy in 0..self.height {
            for dx in 0..self.width {
                let wx = cam_x + dx as i32;
                let wy = cam_y + dy as i32;

                if let Some(&(ch, fg)) = entity_map.get(&(dx as i32, dy as i32)) {
                    self.draw_cell(dx, dy, ch, fg, [10, 10, 15]);
                    continue;
                }

                if !grid.in_bounds(wx, wy) {
                    self.draw_cell(dx, dy, '?', [80, 80, 80], [10, 10, 15]);
                    continue;
                }

                let cell = grid.get(wx, wy);
                let mat = reg.get(cell.material);
                if cell.is_empty() {
                    self.draw_cell(dx, dy, ' ', [mat.color_fg.0, mat.color_fg.1, mat.color_fg.2], [10, 10, 15]);
                } else {
                    let fg = if cell.material == MaterialId::Lava {
                        let r = 200u8.saturating_add(cell.variant / 2);
                        [r, 60, 20]
                    } else {
                        [mat.color_fg.0, mat.color_fg.1, mat.color_fg.2]
                    };
                    let bg = [mat.color_bg.0, mat.color_bg.1, mat.color_bg.2];
                    self.draw_cell(dx, dy, mat.display_char, fg, bg);
                }
            }
        }

        self.window.update_with_buffer(&self.pixels, self.width * CHAR_W, self.height * CHAR_H)
            .expect("update_with_buffer failed");
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }

    pub fn get_keys(&self) -> Vec<Key> {
        self.window.get_keys()
    }

    pub fn get_pressed_keys(&self) -> Vec<Key> {
        self.window.get_keys_pressed(KeyRepeat::No)
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
}
