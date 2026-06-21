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
use crate::render::Renderer;
use crate::world::cell::MaterialId;
use crate::world::grid::Grid;

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
        cam_x: i32,
        cam_y: i32,
    ) -> io::Result<()> {
        if !self.initialized {
            return Ok(());
        }

        let mut out = stdout();
        let mut frame: Vec<(char, (u8, u8, u8), (u8, u8, u8))>;
        let total = self.width * self.height;
        frame = vec![Self::empty_cell(); total];

        for dy in 0..self.height {
            for dx in 0..self.width {
                let wx = cam_x + dx as i32;
                let wy = cam_y + dy as i32;
                let idx = dy * self.width + dx;
                if !grid.in_bounds(wx, wy) {
                    frame[idx] = ('?', (80, 80, 80), (10, 10, 15));
                    continue;
                }
                let cell = grid.get(wx, wy);
                if cell.is_empty() {
                    frame[idx] = (
                        ' ',
                        (cell.fg[0], cell.fg[1], cell.fg[2]),
                        (cell.bg[0], cell.bg[1], cell.bg[2]),
                    );
                } else {
                    let ch = cell.material.display_char();
                    let fg = if cell.material == MaterialId::Lava {
                        let r = 200u8.saturating_add(cell.variant / 2);
                        (r, 60, 20)
                    } else {
                        (cell.fg[0], cell.fg[1], cell.fg[2])
                    };
                    frame[idx] = (ch, fg, (cell.bg[0], cell.bg[1], cell.bg[2]));
                }
            }
        }

        for e in entities.all() {
            for b in &e.bodies {
                if !b.alive {
                    continue;
                }
                let sx = b.x as i32 - cam_x;
                let sy = b.y as i32 - cam_y;
                if sx >= 0 && sx < self.width as i32 && sy >= 0 && sy < self.height as i32 {
                    let idx = sy as usize * self.width + sx as usize;
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
                    frame[idx] = (ch, fg, (20, 10, 10));
                }
            }
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
