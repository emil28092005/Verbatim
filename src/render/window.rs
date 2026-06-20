use minifb::{Key, Window, WindowOptions};
use fontdue::{Font, FontSettings};
use crate::entity::{EntityManager, EntityKind};
use crate::world::cell::MaterialId;
use crate::world::grid::Grid;
use crate::world::material::MaterialRegistry;

const CHAR_W: usize = 8;
const CHAR_H: usize = 16;
const ATLAS_COLS: usize = 16;
const ATLAS_ROWS: usize = 16;
const ATLAS_W: usize = ATLAS_COLS * CHAR_W;
const ATLAS_H: usize = ATLAS_ROWS * CHAR_H;

pub struct WindowRenderer {
    window: Window,
    width: usize,
    height: usize,
    pixels: Vec<u32>,
    atlas: Vec<u8>,
    atlas_map: std::collections::HashMap<char, (usize, usize)>,
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

        let mut window = Window::new(
            "Verbatim",
            width * CHAR_W,
            height * CHAR_H,
            WindowOptions {
                resize: true,
                ..WindowOptions::default()
            },
        ).expect("Failed to create window");
        window.set_target_fps(60);

        let mut atlas = vec![0u8; ATLAS_W * ATLAS_H];
        let mut atlas_map = std::collections::HashMap::new();

        let chars: Vec<char> = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~".chars().collect();

        for (i, &ch) in chars.iter().enumerate() {
            let col = i % ATLAS_COLS;
            let row = i / ATLAS_COLS;
            atlas_map.insert(ch, (col, row));

            let (metrics, bitmap) = font.rasterize(ch, CHAR_H as f32);
            let gw = metrics.width;
            let gh = metrics.height;

            for y in 0..gh.min(CHAR_H) {
                for x in 0..gw.min(CHAR_W) {
                    let alpha = bitmap[y * gw + x];
                    if alpha > 0 {
                        let px = (x as i32 + metrics.xmin).max(0) as usize;
                        let py = (y as i32 + CHAR_H as i32 - gh as i32 - metrics.ymin).max(0) as usize;
                        if px < CHAR_W && py < CHAR_H {
                            let ax = col * CHAR_W + px;
                            let ay = row * CHAR_H + py;
                            atlas[ay * ATLAS_W + ax] = alpha;
                        }
                    }
                }
            }
        }

        Self {
            window,
            width,
            height,
            pixels: vec![0u32; width * CHAR_W * height * CHAR_H],
            atlas,
            atlas_map,
        }
    }

    #[inline]
    fn blend(fg: [u8; 3], bg: [u8; 3], alpha: u8) -> u32 {
        if alpha == 0 {
            return ((bg[0] as u32) << 16) | ((bg[1] as u32) << 8) | (bg[2] as u32);
        }
        if alpha == 255 {
            return ((fg[0] as u32) << 16) | ((fg[1] as u32) << 8) | (fg[2] as u32);
        }
        let a = alpha as u32;
        let inv = 255 - a;
        let r = (fg[0] as u32 * a + bg[0] as u32 * inv) / 255;
        let g = (fg[1] as u32 * a + bg[1] as u32 * inv) / 255;
        let b = (fg[2] as u32 * a + bg[2] as u32 * inv) / 255;
        (r << 16) | (g << 8) | b
    }

    #[inline]
    fn draw_cell(&mut self, col: usize, row: usize, ch: char, fg: [u8; 3], bg: [u8; 3]) {
        let (ac, ar) = match self.atlas_map.get(&ch) {
            Some(&(c, r)) => (c, r),
            None => return,
        };

        let base_x = col * CHAR_W;
        let base_y = row * CHAR_H;
        let screen_w = self.width * CHAR_W;

        for y in 0..CHAR_H {
            let ay = ar * CHAR_H + y;
            let py = base_y + y;
            if py >= self.height * CHAR_H { break; }

            let atlas_row = &self.atlas[ay * ATLAS_W + ac * CHAR_W..ay * ATLAS_W + ac * CHAR_W + CHAR_W];
            let pixel_row = &mut self.pixels[py * screen_w + base_x..py * screen_w + base_x + CHAR_W.min(screen_w - base_x)];

            for x in 0..pixel_row.len() {
                pixel_row[x] = Self::blend(fg, bg, atlas_row[x]);
            }
        }
    }

    pub fn render(&mut self, grid: &Grid, entities: &EntityManager, cam_x: i32, cam_y: i32) {
        let reg = MaterialRegistry::instance();
        let screen_w = self.width * CHAR_W;
        let screen_h = self.height * CHAR_H;

        for py in (0..screen_h).step_by(CHAR_H) {
            for px in (0..screen_w).step_by(CHAR_W) {
                let base = py * screen_w + px;
                let end = base + CHAR_W.min(screen_w - px);
                for p in base..end {
                    self.pixels[p] = 0x000A0A0F;
                }
            }
        }

        let mut entity_map: std::collections::HashMap<(i32, i32), (char, [u8; 3])> = std::collections::HashMap::new();
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
                    self.draw_cell(dx, dy, ' ', [10, 10, 15], [10, 10, 15]);
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

        let _ = self.window.update_with_buffer(&self.pixels, screen_w, screen_h);
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }

    pub fn get_keys_down(&self) -> Vec<Key> {
        self.window.get_keys()
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
}
