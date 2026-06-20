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
    width: usize,
    height: usize,
    pixels: Vec<u32>,
    atlas: Vec<u8>,
    atlas_map: std::collections::HashMap<char, (usize, usize)>,
    font: Font,
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

        let mut atlas = vec![0u8; ATLAS_W * ATLAS_H];
        let mut atlas_map = std::collections::HashMap::new();

        let chars: Vec<char> = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~?".chars().collect();

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
            width,
            height,
            pixels: vec![0x000A0A0F; width * CHAR_W * height * CHAR_H],
            atlas,
            atlas_map,
            font,
        }
    }

    #[inline(always)]
    fn blend_fast(fg: [u8; 3], bg: u32, alpha: u8) -> u32 {
        if alpha == 0 { return bg; }
        if alpha == 255 {
            return ((fg[0] as u32) << 16) | ((fg[1] as u32) << 8) | (fg[2] as u32);
        }
        let a = alpha as u32;
        let inv = 255 - a;
        let br = (bg >> 16) & 0xFF;
        let bg_ = (bg >> 8) & 0xFF;
        let bb = bg & 0xFF;
        let r = (fg[0] as u32 * a + br * inv) >> 8;
        let g = (fg[1] as u32 * a + bg_ * inv) >> 8;
        let b = (fg[2] as u32 * a + bb * inv) >> 8;
        (r << 16) | (g << 8) | b
    }

    pub fn render_to_buffer(&mut self, grid: &Grid, entities: &EntityManager, cam_x: i32, cam_y: i32) {
        let reg = MaterialRegistry::instance();
        let screen_w = self.width * CHAR_W;
        let bg_default = 0x000A0A0F;

        // Fast clear: fill with background
        self.pixels.fill(bg_default);

        // Build entity overlay
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

        // Draw cells
        for dy in 0..self.height {
            let wy = cam_y + dy as i32;
            for dx in 0..self.width {
                let wx = cam_x + dx as i32;

                let (ch, fg, bg) = if let Some(&(ec, ef)) = entity_map.get(&(dx as i32, dy as i32)) {
                    (ec, ef, bg_default)
                } else if !grid.in_bounds(wx, wy) {
                    ('?', [80, 80, 80], bg_default)
                } else {
                    let cell = grid.get(wx, wy);
                    let mat = reg.get(cell.material);
                    if cell.is_empty() {
                        (' ', [10, 10, 15], bg_default)
                    } else {
                        let fg = if cell.material == MaterialId::Lava {
                            let r = 200u8.saturating_add(cell.variant / 2);
                            [r, 60, 20]
                        } else {
                            [mat.color_fg.0, mat.color_fg.1, mat.color_fg.2]
                        };
                        let bg = ((mat.color_bg.0 as u32) << 16) | ((mat.color_bg.1 as u32) << 8) | (mat.color_bg.2 as u32);
                        (mat.display_char, fg, bg)
                    }
                };

                // Skip drawing if it's a space on default background
                if ch == ' ' && bg == bg_default {
                    continue;
                }

                let (ac, ar) = match self.atlas_map.get(&ch) {
                    Some(&(c, r)) => (c, r),
                    None => continue,
                };

                let base_x = dx * CHAR_W;
                let base_y = dy * CHAR_H;

                for y in 0..CHAR_H {
                    let ay = ar * CHAR_H + y;
                    let py = base_y + y;
                    let atlas_off = ay * ATLAS_W + ac * CHAR_W;
                    let pix_off = py * screen_w + base_x;

                    for x in 0..CHAR_W {
                        let alpha = self.atlas[atlas_off + x];
                        if alpha > 0 {
                            self.pixels[pix_off + x] = Self::blend_fast(fg, bg, alpha);
                        } else {
                            self.pixels[pix_off + x] = bg;
                        }
                    }
                }
            }
        }
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
    pub fn pixels(&self) -> &[u32] { &self.pixels }
    pub fn pixel_w(&self) -> usize { self.width * CHAR_W }
    pub fn pixel_h(&self) -> usize { self.height * CHAR_H }
    pub fn font(&self) -> &Font { &self.font }
}
