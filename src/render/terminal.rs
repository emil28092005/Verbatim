use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{
        DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{
        self, size as term_size, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use std::io::{self, stdout, Write};

use crate::entity::EntityManager;
use crate::render::lighting::{self, apply_light_tuple, LightGrid};
use crate::render::Renderer;
use crate::world::cell::MaterialId;
use crate::world::grid::Grid;

fn entity_priority(kind: crate::entity::EntityKind) -> u32 {
    match kind {
        crate::entity::EntityKind::Player => 3,
        crate::entity::EntityKind::Goblin => 2,
        crate::entity::EntityKind::Slime => 1,
        crate::entity::EntityKind::Corpse => 0,
    }
}

fn background_color(wx: i32, wy: i32, vy: i32, view_h: i32) -> (u8, u8, u8) {
    let t = (vy as f32 / view_h as f32).clamp(0.0, 1.0);
    let base_r = (10.0 + t * 15.0) as u8;
    let base_g = (10.0 + t * 25.0) as u8;
    let base_b = (25.0 + t * 35.0) as u8;

    let hash = ((wx.wrapping_mul(73856093)) ^ (wy.wrapping_mul(19349663))).abs();
    if hash % 80 == 0 {
        let brightness = (60 + (hash % 120) as u8).min(255);
        return (brightness, brightness, (brightness + 20).min(255));
    }

    (base_r, base_g, base_b)
}

pub struct TerminalRenderer {
    width: usize,
    height: usize,
    prev_frame: Vec<(char, (u8, u8, u8), (u8, u8, u8))>,
    initialized: bool,
}

impl TerminalRenderer {
    pub fn new() -> Self {
        Self {
            width: 80,
            height: 25,
            prev_frame: Vec::new(),
            initialized: false,
        }
    }

    fn detect_size(&mut self) -> io::Result<()> {
        let (w, h) = term_size().unwrap_or((200, 60));
        self.width = (w as usize).min(250).max(80);
        self.height = (h as usize).min(120).max(25);
        Ok(())
    }

    fn empty_cell() -> (char, (u8, u8, u8), (u8, u8, u8)) {
        (' ', (0, 0, 0), (10, 10, 15))
    }
}

impl Renderer for TerminalRenderer {
    fn init(&mut self) -> io::Result<()> {
        self.detect_size()?;
        let total = self.width * self.height;
        self.prev_frame = vec![Self::empty_cell(); total];

        terminal::enable_raw_mode().map_err(|e| io::Error::other(e))?;
        execute!(
            stdout(),
            EnterAlternateScreen,
            Hide,
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
            ),
            EnableMouseCapture,
            Clear(ClearType::All),
        )?;

        self.initialized = true;
        Ok(())
    }

    fn render(
        &mut self,
        grid: &Grid,
        entities: &EntityManager,
        items: &crate::entity::item::ItemManager,
        ui: &crate::ui::UiLayer,
        cam_x: i32,
        cam_y: i32,
        lighting: Option<&LightGrid>,
    ) -> io::Result<()> {
        if !self.initialized {
            return Ok(());
        }

        let mut out = stdout();
        let mut frame: Vec<(char, (u8, u8, u8), (u8, u8, u8))>;
        let total = self.width * self.height;
        frame = vec![Self::empty_cell(); total];

        let h = self.height as i32;

        let mut entity_map: std::collections::HashMap<(i32, i32), (u32, char, (u8, u8, u8))> =
            std::collections::HashMap::new();
        for e in entities.all() {
            for b in &e.bodies {
                if !b.alive {
                    continue;
                }
                let sx = b.x as i32 - cam_x;
                let sy = b.y as i32 - cam_y;
                if sx >= 0 && sx < self.width as i32 && sy >= 0 && sy < self.height as i32 {
                    let ch = match e.kind {
                        crate::entity::EntityKind::Player if e.alive => '@',
                        crate::entity::EntityKind::Goblin if e.alive => 'g',
                        crate::entity::EntityKind::Slime if e.alive => 's',
                        _ => '%',
                    };
                    let fg = if e.on_fire {
                        (255, 160, 40)
                    } else {
                        (b.color[0], b.color[1], b.color[2])
                    };
                    let priority = entity_priority(e.kind);
                    let key = (sx, sy);
                    if entity_map
                        .get(&key)
                        .map(|(p, _, _)| priority > *p)
                        .unwrap_or(true)
                    {
                        entity_map.insert(key, (priority, ch, fg));
                    }
                }
            }
        }

        let mut shadow_map: std::collections::HashSet<(i32, i32)> =
            std::collections::HashSet::new();
        for (&(ex, ey), _) in entity_map.iter() {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let sx = ex + dx;
                    let sy = ey + dy;
                    if sx < 0 || sx >= self.width as i32 || sy < 0 || sy >= self.height as i32 {
                        continue;
                    }
                    if entity_map.contains_key(&(sx, sy)) {
                        continue;
                    }
                    let wx = cam_x + sx;
                    let wy = cam_y + sy;
                    if grid.in_bounds(wx, wy) && grid.get(wx, wy).is_empty() {
                        shadow_map.insert((sx, sy));
                    }
                }
            }
        }

        for dy in 0..self.height {
            for dx in 0..self.width {
                let wx = cam_x + dx as i32;
                let wy = cam_y + dy as i32;
                let idx = dy * self.width + dx;
                let light = lighting
                    .map(|l| l.get(dx as i32, dy as i32))
                    .unwrap_or_else(lighting::ambient_light);
                if !grid.in_bounds(wx, wy) {
                    let bg = background_color(wx, wy, dy as i32, h);
                    let bg = apply_light_tuple(bg, light);
                    frame[idx] = ('?', apply_light_tuple((80, 80, 80), light), bg);
                    continue;
                }
                let cell = grid.get(wx, wy);
                if cell.is_empty() {
                    let bg = background_color(wx, wy, dy as i32, h);
                    let bg = if shadow_map.contains(&(dx as i32, dy as i32)) {
                        (
                            (bg.0 as f32 * 0.4) as u8,
                            (bg.1 as f32 * 0.4) as u8,
                            (bg.2 as f32 * 0.4) as u8,
                        )
                    } else {
                        bg
                    };
                    let bg = apply_light_tuple(bg, light);
                    let fg = apply_light_tuple((cell.fg[0], cell.fg[1], cell.fg[2]), light);
                    frame[idx] = (' ', fg, bg);
                } else {
                    let ch = cell.material.display_char();
                    let fg = if cell.material == MaterialId::Lava {
                        let r = 200u8.saturating_add(cell.variant / 2);
                        (r, 60, 20)
                    } else {
                        (cell.fg[0], cell.fg[1], cell.fg[2])
                    };
                    let fg = apply_light_tuple(fg, light);
                    let bg = apply_light_tuple((cell.bg[0], cell.bg[1], cell.bg[2]), light);
                    frame[idx] = (ch, fg, bg);
                }
            }
        }

        for ((sx, sy), (_, ch, fg)) in entity_map {
            let idx = sy as usize * self.width + sx as usize;
            let light = lighting
                .map(|l| l.get(sx, sy))
                .unwrap_or_else(lighting::ambient_light);
            let fg = apply_light_tuple(fg, light);
            frame[idx] = (ch, fg, apply_light_tuple((20, 10, 10), light));
        }

        for item in items.all() {
            let sx = item.x - cam_x;
            let sy = item.y - cam_y;
            if sx >= 0 && sx < self.width as i32 && sy >= 0 && sy < self.height as i32 {
                let idx = sy as usize * self.width + sx as usize;
                let color = item.color();
                let light = lighting
                    .map(|l| l.get(sx, sy))
                    .unwrap_or_else(lighting::ambient_light);
                frame[idx] = (
                    item.display_char(),
                    apply_light_tuple((color[0], color[1], color[2]), light),
                    apply_light_tuple((20, 10, 10), light),
                );
            }
        }

        for (x, y) in ui.keys() {
            let tx = *x / crate::ui::UI_SCALE;
            let ty = *y / crate::ui::UI_SCALE;
            if tx < 0 || tx >= self.width as i32 || ty < 0 || ty >= self.height as i32 {
                continue;
            }
            let cell = ui.get(*x, *y).unwrap();
            let idx = ty as usize * self.width + tx as usize;
            let a = cell.alpha as f32 / 255.0;
            let (_, old_fg, old_bg) = frame[idx];
            let blend = |c: u8, o: u8| (c as f32 * a + o as f32 * (1.0 - a)).min(255.0) as u8;
            frame[idx] = (
                cell.ch,
                (
                    blend(cell.fg[0], old_fg.0),
                    blend(cell.fg[1], old_fg.1),
                    blend(cell.fg[2], old_fg.2),
                ),
                (
                    blend(cell.bg[0], old_bg.0),
                    blend(cell.bg[1], old_bg.1),
                    blend(cell.bg[2], old_bg.2),
                ),
            );
        }

        let mut prev_color: Option<(Color, Color)> = None;
        for i in 0..total {
            if frame[i] == self.prev_frame[i] {
                continue;
            }
            let dx = i % self.width;
            let dy = i / self.width;
            let (ch, fg, bg) = frame[i];

            queue!(out, MoveTo(dx as u16, dy as u16))?;

            let new_fg = Color::Rgb {
                r: fg.0,
                g: fg.1,
                b: fg.2,
            };
            let new_bg = Color::Rgb {
                r: bg.0,
                g: bg.1,
                b: bg.2,
            };
            if prev_color != Some((new_fg, new_bg)) {
                queue!(out, SetForegroundColor(new_fg), SetBackgroundColor(new_bg))?;
                prev_color = Some((new_fg, new_bg));
            }
            queue!(out, Print(ch))?;
        }

        out.flush()?;
        self.prev_frame = frame;
        Ok(())
    }

    fn shutdown(&mut self) -> io::Result<()> {
        if !self.initialized {
            return Ok(());
        }
        execute!(
            stdout(),
            ResetColor,
            Show,
            PopKeyboardEnhancementFlags,
            LeaveAlternateScreen,
            DisableMouseCapture,
        )?;
        terminal::disable_raw_mode().map_err(|e| io::Error::other(e))?;
        self.initialized = false;
        Ok(())
    }

    fn viewport_w(&self) -> usize {
        self.width
    }

    fn viewport_h(&self) -> usize {
        self.height
    }
}

impl Drop for TerminalRenderer {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}
