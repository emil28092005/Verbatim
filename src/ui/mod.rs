use std::collections::HashMap;

use crate::entity::entity::{Entity, EntityKind};
use crate::world::cell::MaterialId;
use crate::world::chunked_grid::ChunkedGrid;

pub const UI_SCALE: i32 = 4;
pub const FONT_W: i32 = 3;
pub const FONT_H: i32 = 5;

#[derive(Clone, Debug)]
pub struct UiCell {
    pub ch: char,
    pub fg: [u8; 3],
    pub bg: [u8; 3],
    pub alpha: u8,
}

pub struct UiLayer {
    cells: HashMap<(i32, i32), UiCell>,
    messages: Vec<(String, u32)>,
    damage_numbers: Vec<DamageNumber>,
    font_scale: i32,
}

#[derive(Clone)]
pub struct DamageNumber {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub life: u32,
    pub max_life: u32,
}

impl UiLayer {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            messages: Vec::new(),
            damage_numbers: Vec::new(),
            font_scale: 2,
        }
    }

    pub fn set_font_scale(&mut self, scale: i32) {
        self.font_scale = scale.max(1);
    }

    pub fn font_scale(&self) -> i32 {
        self.font_scale
    }

    #[inline]
    fn fw(&self) -> i32 {
        FONT_W * self.font_scale
    }

    #[inline]
    fn fh(&self) -> i32 {
        FONT_H * self.font_scale
    }

    pub fn clear(&mut self) {
        self.cells.clear();
        self.damage_numbers.retain(|d| d.life > 0);
        for d in &mut self.damage_numbers {
            d.y -= 0.1;
            d.life -= 1;
        }
    }

    pub fn set(&mut self, x: i32, y: i32, ch: char, fg: [u8; 3], bg: [u8; 3]) {
        self.cells.insert(
            (x, y),
            UiCell {
                ch,
                fg,
                bg,
                alpha: 255,
            },
        );
    }

    pub fn set_alpha(&mut self, x: i32, y: i32, ch: char, fg: [u8; 3], bg: [u8; 3], alpha: u8) {
        self.cells.insert((x, y), UiCell { ch, fg, bg, alpha });
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&UiCell> {
        self.cells.get(&(x, y))
    }

    pub fn keys(&self) -> impl Iterator<Item = &(i32, i32)> {
        self.cells.keys()
    }

    pub fn add_message(&mut self, text: &str) {
        self.messages.push((text.to_string(), 300));
    }

    pub fn add_damage_number(&mut self, x: f32, y: f32, text: &str) {
        self.damage_numbers.push(DamageNumber {
            x,
            y,
            text: text.to_string(),
            life: 50,
            max_life: 50,
        });
    }

    pub fn draw_damage_numbers(&mut self, cam_x: i32, cam_y: i32) {
        let numbers: Vec<DamageNumber> = self.damage_numbers.clone();
        for d in numbers {
            let sx = (d.x as i32 - cam_x) * UI_SCALE;
            let sy = (d.y as i32 - cam_y) * UI_SCALE;
            let fg = if d.text.starts_with('+') {
                [80, 240, 80]
            } else {
                [255, 80, 80]
            };
            self.draw_text(sx, sy, &d.text, fg, 255);
        }
    }

    pub fn draw_edge_indicators(
        &mut self,
        screen_w: usize,
        screen_h: usize,
        entities: &[Entity],
        cam_x: i32,
        cam_y: i32,
    ) {
        let sw = screen_w as i32;
        let sh = screen_h as i32;
        for e in entities {
            if !e.alive {
                continue;
            }
            let (sx, sy) = entity_screen_pos_ui(e, cam_x, cam_y);
            if sx >= 0 && sx < sw && sy >= 0 && sy < sh {
                continue;
            }
            let dx = sx - sw / 2;
            let dy = sy - sh / 2;
            let dist = ((dx * dx + dy * dy) as f32).sqrt().max(1.0);
            let ix = (sw / 2) as f32 + (dx as f32 / dist) * (sw as f32 / 2.0 - 2.0);
            let iy = (sh / 2) as f32 + (dy as f32 / dist) * (sh as f32 / 2.0 - 2.0);
            let (ix, iy) = (
                ix.clamp(1.0, sw as f32 - 2.0) as i32,
                iy.clamp(1.0, sh as f32 - 2.0) as i32,
            );
            let ch = match e.kind {
                EntityKind::Goblin => 'g',
                EntityKind::Slime => 's',
                EntityKind::Player => '@',
                EntityKind::Corpse => '%',
            };
            let fg = match e.kind {
                EntityKind::Goblin => [160, 240, 120],
                EntityKind::Slime => [120, 240, 160],
                EntityKind::Player => [200, 180, 255],
                EntityKind::Corpse => [160, 160, 160],
            };
            self.set(ix, iy, ch, fg, [20, 20, 30]);
        }
    }

    pub fn tick_messages(&mut self) {
        for (_, life) in &mut self.messages {
            if *life > 0 {
                *life -= 1;
            }
        }
        self.messages.retain(|(_, life)| *life > 0);
        while self.messages.len() > 8 {
            self.messages.remove(0);
        }
    }

    pub fn messages(&self) -> &[(String, u32)] {
        &self.messages
    }

    pub fn damage_numbers(&self) -> &[DamageNumber] {
        &self.damage_numbers
    }

    pub fn draw_health_bar(
        &mut self,
        screen_x: i32,
        screen_y: i32,
        health: f32,
        max_health: f32,
        width: i32,
    ) {
        if max_health <= 0.0 {
            return;
        }
        let ratio = (health / max_health).clamp(0.0, 1.0);
        let scaled_width = width * self.font_scale;
        let filled = (ratio * scaled_width as f32).round() as i32;
        let color = if ratio > 0.6 {
            [60, 220, 60]
        } else if ratio > 0.3 {
            [240, 200, 40]
        } else {
            [240, 50, 50]
        };
        for i in 0..scaled_width {
            let x = screen_x + i;
            let ch = if i < filled { '#' } else { '-' };
            let fg = if i < filled { color } else { [80, 80, 80] };
            self.set(x, screen_y, ch, fg, [0, 0, 0]);
        }
    }

    pub fn draw_entity_labels(&mut self, entities: &[Entity], cam_x: i32, cam_y: i32) {
        for e in entities {
            if !e.alive || e.kind == EntityKind::Corpse {
                continue;
            }
            let (sx, sy) = entity_screen_pos_ui(e, cam_x, cam_y);
            let label = e.name().to_uppercase();
            let top = sy - (e.half_h as i32 * UI_SCALE) - 7;
            let x = sx - (self.text_width(&label) / 2);
            let fg = entity_kind_color(e);
            self.draw_text(x, top, &label, fg, 255);
        }
    }

    pub fn draw_status_icons(&mut self, entities: &[Entity], cam_x: i32, cam_y: i32) {
        for e in entities {
            if !e.alive {
                continue;
            }
            let (sx, sy) = entity_screen_pos_ui(e, cam_x, cam_y);
            let icon_x = sx + (e.half_w as i32 * UI_SCALE) + 1;
            let mut icon_y = sy - (e.half_h as i32 * UI_SCALE) - 7;
            if e.on_fire {
                self.set(icon_x, icon_y, 'F', [255, 100, 20], [0, 0, 0]);
                icon_y += 1;
            }
            if e.poisoned {
                self.set(icon_x, icon_y, 'P', [80, 255, 80], [0, 0, 0]);
                icon_y += 1;
            }
            if e.frozen {
                self.set(icon_x, icon_y, 'I', [120, 220, 255], [0, 0, 0]);
                icon_y += 1;
            }
            if e.bleeding {
                self.set(icon_x, icon_y, 'B', [255, 40, 40], [0, 0, 0]);
            }
        }
    }

    pub fn draw_items(&mut self, items: &[crate::entity::item::Item], cam_x: i32, cam_y: i32) {
        for item in items {
            let sx = (item.x - cam_x) * UI_SCALE;
            let sy = (item.y - cam_y) * UI_SCALE;
            self.set(sx, sy, item.display_char(), item.color(), [0, 0, 0]);
        }
    }

    pub fn draw_minimap(
        &mut self,
        screen_w: usize,
        screen_h: usize,
        grid: &ChunkedGrid,
        entities: &[Entity],
        cam_x: i32,
        cam_y: i32,
    ) {
        let size = 16;
        let scale = 16;
        let start_x = screen_w as i32 - size - 2;
        let start_y = 1;
        let view_w = screen_w as i32 / UI_SCALE;
        let view_h = screen_h as i32 / UI_SCALE;
        let cx = (cam_x + view_w / 2) / scale;
        let cy = (cam_y + view_h / 2) / scale;
        for dy in 0..size {
            for dx in 0..size {
                let mx = cx - size / 2 + dx;
                let my = cy - size / 2 + dy;
                let wx = mx * scale;
                let wy = my * scale;
                if !grid.in_bounds(wx, wy) {
                    self.set(start_x + dx, start_y + dy, ' ', [0, 0, 0], [20, 20, 30]);
                    continue;
                }
                let mut r = 0u32;
                let mut g = 0u32;
                let mut b = 0u32;
                let mut n = 0u32;
                for yy in 0..scale {
                    for xx in 0..scale {
                        let cell = grid.get(wx + xx, wy + yy);
                        if cell.material != MaterialId::Empty {
                            let c = cell.fg;
                            r += c[0] as u32;
                            g += c[1] as u32;
                            b += c[2] as u32;
                            n += 1;
                        }
                    }
                }
                let fg = if n > 0 {
                    [(r / n) as u8, (g / n) as u8, (b / n) as u8]
                } else {
                    [40, 40, 50]
                };
                let bg = [20, 20, 30];
                self.set(start_x + dx, start_y + dy, '.', fg, bg);
            }
        }
        for e in entities {
            if !e.alive {
                continue;
            }
            let (ex, ey) = e.center();
            let mx = (ex as i32 / scale) - cx + size / 2;
            let my = (ey as i32 / scale) - cy + size / 2;
            if mx >= 0 && mx < size && my >= 0 && my < size {
                let ch = match e.kind {
                    EntityKind::Player => '@',
                    EntityKind::Goblin => 'g',
                    EntityKind::Slime => 's',
                    EntityKind::Corpse => '%',
                };
                let fg = entity_kind_color(e);
                self.set(start_x + mx, start_y + my, ch, fg, [0, 0, 0]);
            }
        }
        for dx in 0..size {
            self.set(start_x + dx, start_y - 1, '-', [80, 80, 100], [0, 0, 0]);
            self.set(start_x + dx, start_y + size, '-', [80, 80, 100], [0, 0, 0]);
        }
        for dy in 0..size {
            self.set(start_x - 1, start_y + dy, '|', [80, 80, 100], [0, 0, 0]);
            self.set(start_x + size, start_y + dy, '|', [80, 80, 100], [0, 0, 0]);
        }
        self.set(start_x - 1, start_y - 1, '+', [80, 80, 100], [0, 0, 0]);
        self.set(start_x + size, start_y - 1, '+', [80, 80, 100], [0, 0, 0]);
        self.set(start_x - 1, start_y + size, '+', [80, 80, 100], [0, 0, 0]);
        self.set(
            start_x + size,
            start_y + size,
            '+',
            [80, 80, 100],
            [0, 0, 0],
        );
    }

    pub fn draw_character_panel(
        &mut self,
        start_x: i32,
        start_y: i32,
        player: Option<&Entity>,
        player_state: &crate::entity::player::Player,
    ) {
        let fs = self.font_scale;
        let fh = self.fh();
        let fw = self.fw();
        let w = 52 * fs / 2;
        let h = 62 * fs / 2;
        let bg = [22u8, 26, 38];
        let border = [160u8, 180, 255];
        let title = [220u8, 230, 255];
        let dim = [150u8, 160, 190];
        let fg = [240u8, 240, 250];
        let fill_alpha = 140u8;
        let border_alpha = 220u8;

        for y in 0..h {
            for x in 0..w {
                self.set_alpha(start_x + x, start_y + y, ' ', [0, 0, 0], bg, fill_alpha);
            }
        }
        for x in 0..w {
            if (start_x + x) % 2 == 0 {
                self.set_alpha(start_x + x, start_y, '.', border, bg, border_alpha);
                self.set_alpha(start_x + x, start_y + h - 1, '.', border, bg, border_alpha);
            }
        }
        for y in 0..h {
            if (start_y + y) % 2 == 0 {
                self.set_alpha(start_x, start_y + y, '.', border, bg, border_alpha);
                self.set_alpha(start_x + w - 1, start_y + y, '.', border, bg, border_alpha);
            }
        }
        self.set_alpha(start_x, start_y, '*', border, bg, border_alpha);
        self.set_alpha(start_x + w - 1, start_y, '*', border, bg, border_alpha);
        self.set_alpha(start_x, start_y + h - 1, '*', border, bg, border_alpha);
        self.set_alpha(
            start_x + w - 1,
            start_y + h - 1,
            '*',
            border,
            bg,
            border_alpha,
        );

        let title_text = "STATUS";
        let tx = start_x + (w - self.text_width(title_text)) / 2;
        self.draw_text(tx, start_y + 2, title_text, title, 255);

        let row1 = start_y + 10 * fs / 2;
        let row2 = start_y + 18 * fs / 2;
        let row3 = start_y + 26 * fs / 2;
        let row4 = start_y + 36 * fs / 2;
        let row5 = start_y + 44 * fs / 2;
        let row_inv = start_y + 52 * fs / 2;
        let row_items = start_y + 58 * fs / 2;

        if let Some(p) = player {
            let hp_ratio = (p.health / p.max_health).clamp(0.0, 1.0);
            let hp_filled = (hp_ratio * 38.0 * fs as f32 / 2.0).round() as i32;
            let bar_color = if hp_ratio > 0.6 {
                [80, 240, 80]
            } else if hp_ratio > 0.3 {
                [240, 220, 60]
            } else {
                [255, 60, 60]
            };
            self.draw_text(start_x + 3, row1, "HP", fg, 255);
            for i in 0..(38 * fs / 2) {
                let ch = if i < hp_filled { '#' } else { '-' };
                let c = if i < hp_filled {
                    bar_color
                } else {
                    [80, 80, 100]
                };
                self.set(start_x + 10 * fs / 2 + i, row1, ch, c, bg);
            }
            let hp_text = format!(
                "{}/{}  LV:{}",
                p.health as i32, p.max_health as i32, p.level
            );
            self.draw_text(start_x + 3, row2, &hp_text, fg, 255);

            let xp_ratio = (p.xp as f32 / p.xp_to_level() as f32).clamp(0.0, 1.0);
            let xp_filled = (xp_ratio * 38.0 * fs as f32 / 2.0).round() as i32;
            self.draw_text(start_x + 3, row3, "XP", fg, 255);
            for i in 0..(38 * fs / 2) {
                let ch = if i < xp_filled { '#' } else { '-' };
                let c = if i < xp_filled {
                    [80, 160, 240]
                } else {
                    [60, 60, 80]
                };
                self.set(start_x + 10 * fs / 2 + i, row3, ch, c, bg);
            }

            let stats = format!("STR:{}  AGI:{}", p.strength, p.agility);
            self.draw_text(start_x + 3, row4, &stats, dim, 255);
            let stats2 = format!("TOU:{}  WIL:{}", p.toughness, p.willpower);
            self.draw_text(start_x + 3, row5, &stats2, dim, 255);
        }

        let inv_title = "INVENTORY";
        self.draw_text(start_x + 3, row_inv, inv_title, fg, 255);
        let mut ix = start_x + 3;
        for item in player_state.inventory.iter().take(8) {
            if ix + 2 >= start_x + w - 2 {
                break;
            }
            let [ch1, ch2] = item.display_glyph();
            let col = item.color();
            self.set(ix, row_items, ch1, col, bg);
            self.set(
                ix + 1,
                row_items,
                ch2,
                [col[0] / 2, col[1] / 2, col[2] / 2],
                bg,
            );
            ix += 4;
        }

        let status = if let Some(p) = player {
            let mut s = String::new();
            if p.on_fire {
                s.push_str("[FIRE] ");
            }
            if p.poisoned {
                s.push_str("[PSN] ");
            }
            if p.frozen {
                s.push_str("[ICE] ");
            }
            if p.bleeding {
                s.push_str("[BLD] ");
            }
            s
        } else {
            String::new()
        };
        if !status.is_empty() {
            self.draw_text(start_x + 30 * fs / 2, row4, &status, [255, 80, 80], 255);
        }
    }

    pub fn draw_inventory_overlay(
        &mut self,
        screen_w: usize,
        screen_h: usize,
        player_state: &crate::entity::player::Player,
        mouse_ui_x: i32,
        mouse_ui_y: i32,
    ) -> Vec<(usize, i32, i32)> {
        let bg = [15u8, 18, 28];
        let border = [160u8, 180, 255];
        let title_col = [220u8, 230, 255];
        let dim = [120u8, 130, 160];
        let slot_bg = [30u8, 36, 50];
        let slot_border = [80u8, 100, 140];
        let hover_border = [255u8, 220, 100];

        let cols = 4i32;
        let rows = 2i32;
        let slot_w = 12i32 * self.font_scale / 2;
        let slot_h = 10i32 * self.font_scale / 2;
        let gap = 3i32 * self.font_scale / 2;
        let panel_w = cols * slot_w + (cols + 1) * gap + 4;
        let panel_h = rows * slot_h + (rows + 1) * gap + 28;

        let px_start = (screen_w as i32 - panel_w) / 2;
        let py_start = (screen_h as i32 - panel_h) / 2;

        for y in 0..panel_h {
            for x in 0..panel_w {
                self.set_alpha(px_start + x, py_start + y, ' ', [0, 0, 0], bg, 200);
            }
        }
        for x in 0..panel_w {
            if x % 2 == 0 {
                self.set_alpha(px_start + x, py_start, '.', border, bg, 240);
                self.set_alpha(px_start + x, py_start + panel_h - 1, '.', border, bg, 240);
            }
        }
        for y in 0..panel_h {
            if y % 2 == 0 {
                self.set_alpha(px_start, py_start + y, '.', border, bg, 240);
                self.set_alpha(px_start + panel_w - 1, py_start + y, '.', border, bg, 240);
            }
        }
        self.set_alpha(px_start, py_start, '*', border, bg, 240);
        self.set_alpha(px_start + panel_w - 1, py_start, '*', border, bg, 240);
        self.set_alpha(px_start, py_start + panel_h - 1, '*', border, bg, 240);
        self.set_alpha(
            px_start + panel_w - 1,
            py_start + panel_h - 1,
            '*',
            border,
            bg,
            240,
        );

        let title = "INVENTORY";
        self.draw_text(px_start + 4, py_start + 3, title, title_col, 255);
        let count_text = format!("({}/{} items)", player_state.inventory.len(), cols * rows);
        self.draw_text(
            px_start + 4 + self.text_width(title) + 2,
            py_start + 3,
            &count_text,
            dim,
            200,
        );

        let mut hover_slot: Option<usize> = None;
        let mut slots: Vec<(usize, i32, i32)> = Vec::new();

        for row in 0..rows {
            for col in 0..cols {
                let idx = (row * cols + col) as usize;
                let sx = px_start + gap + col * (slot_w + gap) + 2;
                let sy = py_start + 14 * self.font_scale / 2 + row * (slot_h + gap) + gap;

                for y in 0..slot_h {
                    for x in 0..slot_w {
                        self.set_alpha(sx + x, sy + y, ' ', [0, 0, 0], slot_bg, 200);
                    }
                }
                let sb = if mouse_ui_x >= sx
                    && mouse_ui_x < sx + slot_w
                    && mouse_ui_y >= sy
                    && mouse_ui_y < sy + slot_h
                {
                    hover_slot = Some(idx);
                    hover_border
                } else {
                    slot_border
                };
                for x in 0..slot_w {
                    self.set_alpha(sx + x, sy, '.', sb, slot_bg, 240);
                    self.set_alpha(sx + x, sy + slot_h - 1, '.', sb, slot_bg, 240);
                }
                for y in 0..slot_h {
                    self.set_alpha(sx, sy + y, '.', sb, slot_bg, 240);
                    self.set_alpha(sx + slot_w - 1, sy + y, '.', sb, slot_bg, 240);
                }

                if idx < player_state.inventory.len() {
                    let item = &player_state.inventory[idx];
                    let [ch1, ch2] = item.display_glyph();
                    let col = item.color();
                    let cx = sx + slot_w / 2 - 1;
                    let cy = sy + slot_h / 2 - 2;
                    self.set(cx, cy, ch1, col, slot_bg);
                    self.set(
                        cx + 1,
                        cy,
                        ch2,
                        [col[0] / 2, col[1] / 2, col[2] / 2],
                        slot_bg,
                    );

                    let label = item.name();
                    let label_w = self.text_width(label) as i32;
                    let lx = sx + (slot_w - label_w) / 2;
                    let ly = sy + slot_h - 1;
                    if lx >= sx && lx + label_w <= sx + slot_w {
                        self.draw_text(lx, ly, label, dim, 200);
                    }

                    if hover_slot == Some(idx) {
                        let action = if item.is_consumable() {
                            "L-click: Use"
                        } else if item.is_weapon() {
                            if player_state
                                .weapon
                                .as_ref()
                                .map(|w| w.typ == item.typ)
                                .unwrap_or(false)
                            {
                                "Equipped"
                            } else {
                                "L-click: Equip"
                            }
                        } else if item.is_armor() {
                            if player_state
                                .armor
                                .as_ref()
                                .map(|a| a.typ == item.typ)
                                .unwrap_or(false)
                            {
                                "Equipped"
                            } else {
                                "L-click: Equip"
                            }
                        } else {
                            "L-click: Use"
                        };
                        self.draw_text(sx, sy + slot_h, action, hover_border, 240);
                    }
                } else if hover_slot == Some(idx) {
                    self.draw_text(sx + 1, sy + slot_h / 2, "empty", dim, 200);
                }

                slots.push((idx, sx, sy));
            }
        }

        if player_state.weapon.is_some() || player_state.armor.is_some() {
            let eq_y = py_start + panel_h - 6;
            self.draw_text(px_start + 4, eq_y, "EQ:", dim, 200);
            let mut eq_x = px_start + 8;
            if let Some(ref w) = player_state.weapon {
                let [c1, c2] = w.display_glyph();
                let col = w.color();
                self.set(eq_x, eq_y, c1, col, bg);
                self.set(eq_x + 1, eq_y, c2, [col[0] / 2, col[1] / 2, col[2] / 2], bg);
                eq_x += 4;
            }
            if let Some(ref a) = player_state.armor {
                let [c1, c2] = a.display_glyph();
                let col = a.color();
                self.set(eq_x, eq_y, c1, col, bg);
                self.set(eq_x + 1, eq_y, c2, [col[0] / 2, col[1] / 2, col[2] / 2], bg);
            }
        }

        self.draw_text(
            px_start + 4,
            py_start + panel_h - 3,
            "Tab: close  L-click: use  R-click: drop",
            dim,
            200,
        );

        slots
    }

    pub fn draw_hud(
        &mut self,
        screen_w: usize,
        screen_h: usize,
        player: Option<&Entity>,
        tick: u64,
        brush: MaterialId,
        kills: u32,
        score: u32,
        depth: u32,
        player_state: &crate::entity::player::Player,
        fps: f32,
    ) {
        let fs = self.font_scale;
        let fh = self.fh();
        let bg = [18u8, 20, 30];
        let border = [160u8, 170, 210];
        let fill_alpha = 140u8;
        let brush_name = material_name(brush);
        let weapon_glyph = player_state
            .weapon
            .as_ref()
            .map(|i| i.display_string())
            .unwrap_or("  ".to_string());
        let armor_glyph = player_state
            .armor
            .as_ref()
            .map(|i| i.display_string())
            .unwrap_or("  ".to_string());

        let bar_h = (28 * fs / 2).max(fh * 3);
        let y_top = screen_h as i32 - bar_h;
        let y_row1 = y_top + 2;
        let y_row2 = y_top + 2 + fh + 2;
        let y_row3 = y_top + 2 + (fh + 2) * 2;

        for row in y_top..screen_h as i32 {
            for x in 0..screen_w {
                self.set_alpha(x as i32, row, ' ', [200, 200, 200], bg, fill_alpha);
            }
        }
        for x in 0..screen_w {
            if x % 2 == 0 {
                self.set_alpha(x as i32, y_top - 1, '.', border, bg, 220);
            }
        }

        self.draw_text(0, y_row1, "HP", [220, 220, 220], 255);
        if let Some(p) = player {
            let hp_ratio = (p.health / p.max_health).clamp(0.0, 1.0);
            let hp_bar_w = 28 * fs / 2;
            let hp_filled = (hp_ratio * hp_bar_w as f32).round() as i32;
            let bar_fg = if hp_ratio > 0.6 {
                [80, 240, 80]
            } else if hp_ratio > 0.3 {
                [240, 220, 60]
            } else {
                [255, 60, 60]
            };
            for i in 0..hp_bar_w {
                let ch = if i < hp_filled { '#' } else { '-' };
                let c = if i < hp_filled { bar_fg } else { [70, 70, 90] };
                self.set(10 * fs / 2 + i, y_row1, ch, c, bg);
            }
            let hp_text = format!("{} / {}", p.health as i32, p.max_health as i32);
            self.draw_text(
                (40 * fs / 2).max(10 * fs / 2 + hp_bar_w + 2),
                y_row1,
                &hp_text,
                [220, 220, 220],
                255,
            );
        }

        let brush_color = brush_color(brush);
        self.set(0, y_row2, '#', brush_color, bg);
        let brush_text = format!(" {}", brush_name);
        self.draw_text(1, y_row2, &brush_text, [200, 200, 140], 255);

        let gear = format!(
            "W:[{}] A:[{}] INV:{} FPS:{}",
            weapon_glyph,
            armor_glyph,
            player_state.inventory.len(),
            fps as i32
        );
        let gear_w = self.text_width(&gear);
        let gear_x = (screen_w as i32 - gear_w).max(0);
        self.draw_text(gear_x, y_row2, &gear, [160, 180, 220], 255);

        let stats = format!(
            "LV:{} XP:{} K:{} S:{} D:{} T:{}",
            if let Some(p) = player { p.level } else { 0 },
            if let Some(p) = player { p.xp } else { 0 },
            kills,
            score,
            depth,
            tick
        );
        let stats_w = self.text_width(&stats);
        let stats_x = (screen_w as i32 - stats_w).max(0);
        self.draw_text(stats_x, y_row3, &stats, [180, 190, 220], 255);

        if let Some(p) = player {
            if p.on_fire {
                let msg = "ON FIRE!";
                self.draw_text(40 * fs / 2, y_top - fh - 2, msg, [255, 100, 20], 255);
            }
        }
    }

    pub fn draw_messages(&mut self, x: i32, y: i32) {
        let mut yy = y;
        let messages: Vec<(String, u32)> = self.messages.iter().rev().take(8).cloned().collect();
        let line_h = self.fh() + 1;
        for (msg, life) in messages {
            if yy < 0 {
                break;
            }
            let fade = (life as f32 / 300.0).clamp(0.3, 1.0);
            let fg = [
                (200.0 * fade) as u8,
                (200.0 * fade) as u8,
                (200.0 * fade) as u8,
            ];
            self.draw_text(x, yy, &msg, fg, 255);
            yy -= line_h;
        }
    }

    pub fn draw_death_screen(&mut self, screen_w: usize, screen_h: usize, kills: u32, score: u32) {
        let cx = screen_w as i32 / 2;
        let cy = screen_h as i32 / 2;
        let msg = "YOU DIED";
        let x = cx - (self.text_width(msg) / 2);
        self.draw_text(x, cy - 3, msg, [255, 50, 50], 255);
        let stats = format!("KILLS: {}  SCORE: {}", kills, score);
        let x2 = cx - (self.text_width(&stats) / 2);
        self.draw_text(x2, cy + 3, &stats, [200, 200, 200], 255);
    }

    pub fn text_width(&self, text: &str) -> i32 {
        text.chars()
            .map(|c| if c == '\n' { 0 } else { self.fw() as usize })
            .sum::<usize>() as i32
    }

    pub fn draw_text(&mut self, x: i32, y: i32, text: &str, fg: [u8; 3], alpha: u8) {
        let fs = self.font_scale;
        let mut cx = x;
        for c in text.chars() {
            if c == '\n' {
                cx = x;
                continue;
            }
            if let Some(bitmap) = char_bitmap(c) {
                for row in 0..FONT_H {
                    for col in 0..FONT_W {
                        let filled = (bitmap[row as usize] >> (FONT_W - 1 - col)) & 1 == 1;
                        let a = if filled { alpha } else { 0 };
                        for dy in 0..fs {
                            for dx in 0..fs {
                                let px = cx + col * fs + dx;
                                let py = y + row * fs + dy;
                                self.set_alpha(px, py, c, fg, [0, 0, 0], a);
                            }
                        }
                    }
                }
            } else {
                for row in 0..FONT_H {
                    for col in 0..FONT_W {
                        for dy in 0..fs {
                            for dx in 0..fs {
                                let px = cx + col * fs + dx;
                                let py = y + row * fs + dy;
                                self.set_alpha(px, py, c, fg, [0, 0, 0], alpha);
                            }
                        }
                    }
                }
            }
            cx += self.fw();
        }
    }
}

fn brush_color(mat: MaterialId) -> [u8; 3] {
    let reg = crate::world::material::MaterialRegistry::instance();
    let m = reg.get(mat);
    [m.color_fg.0, m.color_fg.1, m.color_fg.2]
}

fn material_name(mat: MaterialId) -> &'static str {
    match mat {
        MaterialId::Empty => "Erase",
        MaterialId::Sand => "Sand",
        MaterialId::Water => "Water",
        MaterialId::Stone => "Stone",
        MaterialId::Lava => "Lava",
        MaterialId::Wood => "Wood",
        MaterialId::Flesh => "Flesh",
        MaterialId::Bone => "Bone",
        MaterialId::Steam => "Steam",
        MaterialId::Fire => "Fire",
        MaterialId::Acid => "Acid",
        MaterialId::Smoke => "Smoke",
        MaterialId::Grass => "Grass",
        MaterialId::Dirt => "Dirt",
        MaterialId::Stairs => "Stairs",
    }
}

pub fn entity_screen_pos(e: &Entity, cam_x: i32, cam_y: i32) -> (i32, i32) {
    let (cx, cy) = e.center();
    (cx as i32 - cam_x, cy as i32 - cam_y)
}

pub fn entity_screen_pos_ui(e: &Entity, cam_x: i32, cam_y: i32) -> (i32, i32) {
    let (sx, sy) = entity_screen_pos(e, cam_x, cam_y);
    (sx * UI_SCALE, sy * UI_SCALE)
}

pub fn entity_class_glyph(e: &Entity) -> char {
    match e.kind {
        EntityKind::Player if e.alive => '@',
        EntityKind::Goblin if e.alive => 'g',
        EntityKind::Slime if e.alive => 's',
        _ => '%',
    }
}

pub fn entity_kind_color(e: &Entity) -> [u8; 3] {
    match e.kind {
        EntityKind::Player => [200, 180, 255],
        EntityKind::Goblin => [160, 240, 120],
        EntityKind::Slime => [120, 240, 160],
        EntityKind::Corpse => [160, 160, 160],
    }
}

fn char_bitmap(c: char) -> Option<[u8; 5]> {
    let u = c.to_ascii_uppercase();
    match u {
        '0' => Some([0b111, 0b101, 0b101, 0b101, 0b111]),
        '1' => Some([0b010, 0b010, 0b010, 0b010, 0b010]),
        '2' => Some([0b111, 0b001, 0b111, 0b100, 0b111]),
        '3' => Some([0b111, 0b001, 0b111, 0b001, 0b111]),
        '4' => Some([0b101, 0b101, 0b111, 0b001, 0b001]),
        '5' => Some([0b111, 0b100, 0b111, 0b001, 0b111]),
        '6' => Some([0b111, 0b100, 0b111, 0b101, 0b111]),
        '7' => Some([0b111, 0b001, 0b001, 0b001, 0b001]),
        '8' => Some([0b111, 0b101, 0b111, 0b101, 0b111]),
        '9' => Some([0b111, 0b101, 0b111, 0b001, 0b111]),
        'A' => Some([0b111, 0b101, 0b111, 0b101, 0b101]),
        'B' => Some([0b110, 0b101, 0b110, 0b101, 0b110]),
        'C' => Some([0b111, 0b100, 0b100, 0b100, 0b111]),
        'D' => Some([0b110, 0b101, 0b101, 0b101, 0b110]),
        'E' => Some([0b111, 0b100, 0b111, 0b100, 0b111]),
        'F' => Some([0b111, 0b100, 0b111, 0b100, 0b100]),
        'G' => Some([0b111, 0b100, 0b101, 0b101, 0b111]),
        'H' => Some([0b101, 0b101, 0b111, 0b101, 0b101]),
        'I' => Some([0b111, 0b010, 0b010, 0b010, 0b111]),
        'J' => Some([0b001, 0b001, 0b001, 0b101, 0b111]),
        'K' => Some([0b101, 0b101, 0b110, 0b101, 0b101]),
        'L' => Some([0b100, 0b100, 0b100, 0b100, 0b111]),
        'M' => Some([0b101, 0b111, 0b101, 0b101, 0b101]),
        'N' => Some([0b111, 0b101, 0b101, 0b101, 0b101]),
        'O' => Some([0b111, 0b101, 0b101, 0b101, 0b111]),
        'P' => Some([0b111, 0b101, 0b111, 0b100, 0b100]),
        'Q' => Some([0b111, 0b101, 0b101, 0b111, 0b001]),
        'R' => Some([0b111, 0b101, 0b110, 0b101, 0b101]),
        'S' => Some([0b111, 0b100, 0b111, 0b001, 0b111]),
        'T' => Some([0b111, 0b010, 0b010, 0b010, 0b010]),
        'U' => Some([0b101, 0b101, 0b101, 0b101, 0b111]),
        'V' => Some([0b101, 0b101, 0b101, 0b101, 0b010]),
        'W' => Some([0b101, 0b101, 0b101, 0b111, 0b101]),
        'X' => Some([0b101, 0b101, 0b010, 0b101, 0b101]),
        'Y' => Some([0b101, 0b101, 0b010, 0b010, 0b010]),
        'Z' => Some([0b111, 0b001, 0b010, 0b100, 0b111]),
        ' ' => Some([0b000, 0b000, 0b000, 0b000, 0b000]),
        ':' => Some([0b000, 0b010, 0b000, 0b010, 0b000]),
        '/' => Some([0b001, 0b001, 0b010, 0b100, 0b100]),
        '(' => Some([0b001, 0b010, 0b010, 0b010, 0b001]),
        ')' => Some([0b100, 0b010, 0b010, 0b010, 0b100]),
        '[' => Some([0b011, 0b010, 0b010, 0b010, 0b011]),
        ']' => Some([0b110, 0b010, 0b010, 0b010, 0b110]),
        '-' => Some([0b000, 0b000, 0b111, 0b000, 0b000]),
        '?' => Some([0b111, 0b001, 0b011, 0b000, 0b010]),
        '!' => Some([0b010, 0b010, 0b010, 0b000, 0b010]),
        '.' => Some([0b000, 0b000, 0b000, 0b000, 0b010]),
        ',' => Some([0b000, 0b000, 0b000, 0b010, 0b100]),
        '\'' => Some([0b010, 0b010, 0b000, 0b000, 0b000]),
        '+' => Some([0b000, 0b010, 0b111, 0b010, 0b000]),
        '=' => Some([0b000, 0b111, 0b000, 0b111, 0b000]),
        '>' => Some([0b100, 0b010, 0b001, 0b010, 0b100]),
        '<' => Some([0b001, 0b010, 0b100, 0b010, 0b001]),
        '*' => Some([0b000, 0b101, 0b010, 0b101, 0b000]),
        '%' => Some([0b101, 0b001, 0b010, 0b100, 0b101]),
        '#' => Some([0b101, 0b111, 0b101, 0b111, 0b101]),
        '|' => Some([0b010, 0b010, 0b010, 0b010, 0b010]),
        '@' => Some([0b111, 0b101, 0b111, 0b101, 0b111]),
        '_' => Some([0b000, 0b000, 0b000, 0b000, 0b111]),
        _ => None,
    }
}
