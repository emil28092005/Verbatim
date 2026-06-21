use crate::entity::item::ItemManager;
use crate::entity::EntityManager;
use crate::render::lighting;
use crate::ui::UiLayer;
use crate::world::cell::MaterialId;
use crate::world::chunked_grid::ChunkedGrid;
use image::{ImageBuffer, RgbImage};

pub const CELL_SIZE: u32 = 8;
pub const UI_CELL_SIZE: u32 = 2;

pub fn capture_frame(
    grid: &ChunkedGrid,
    entities: &EntityManager,
    items: &ItemManager,
    ui: &UiLayer,
    cam_x: i32,
    cam_y: i32,
    view_w: u32,
    view_h: u32,
    lighting: Option<&lighting::LightGrid>,
) -> RgbImage {
    let width = view_w * CELL_SIZE;
    let height = view_h * CELL_SIZE;
    let mut img: RgbImage = ImageBuffer::new(width, height);

    let entity_positions = entity_positions(entities, cam_x, cam_y, view_w, view_h);
    let shadow_positions = shadow_positions(&entity_positions, grid, cam_x, cam_y, view_w, view_h);

    for vy in 0..view_h as i32 {
        for vx in 0..view_w as i32 {
            let wx = cam_x + vx;
            let wy = cam_y + vy;
            let light = lighting
                .map(|l| l.get(vx, vy))
                .unwrap_or_else(lighting::ambient_light);

            let color = if let Some(c) = entity_positions.get(&(vx, vy)) {
                lighting::apply_light(*c, light)
            } else if let Some(c) = item_color_at(items, wx, wy) {
                lighting::apply_light(c, light)
            } else if shadow_positions.contains(&(vx, vy)) {
                [0, 0, 0]
            } else if !grid.in_bounds(wx, wy) {
                lighting::apply_light([40, 40, 40], light)
            } else {
                let cell = grid.get(wx, wy);
                if cell.is_empty() {
                    lighting::apply_light(background_color(wx, wy, vy, view_h as i32), light)
                } else if cell.material == MaterialId::Lava {
                    let r = 200u8.saturating_add(cell.variant / 2);
                    lighting::apply_light([r, 60, 20], light)
                } else {
                    lighting::apply_light([cell.fg[0], cell.fg[1], cell.fg[2]], light)
                }
            };

            draw_cell(&mut img, vx as u32, vy as u32, color);
        }
    }

    for (x, y) in ui.keys() {
        let cell = ui.get(*x, *y).unwrap();
        let px = (*x as u32) * UI_CELL_SIZE;
        let py = (*y as u32) * UI_CELL_SIZE;
        if px + UI_CELL_SIZE <= img.width() && py + UI_CELL_SIZE <= img.height() {
            let alpha = cell.alpha as f32 / 255.0;
            for dy in 0..UI_CELL_SIZE {
                for dx in 0..UI_CELL_SIZE {
                    let p = img.get_pixel(px + dx, py + dy);
                    let r = (cell.fg[0] as f32 * alpha + p[0] as f32 * (1.0 - alpha)) as u8;
                    let g = (cell.fg[1] as f32 * alpha + p[1] as f32 * (1.0 - alpha)) as u8;
                    let b = (cell.fg[2] as f32 * alpha + p[2] as f32 * (1.0 - alpha)) as u8;
                    img.put_pixel(px + dx, py + dy, image::Rgb([r, g, b]));
                }
            }
        }
    }

    img
}

fn background_color(wx: i32, wy: i32, vy: i32, view_h: i32) -> [u8; 3] {
    let t = (vy as f32 / view_h as f32).clamp(0.0, 1.0);
    let base_r = (10.0 + t * 15.0) as u8;
    let base_g = (10.0 + t * 25.0) as u8;
    let base_b = (25.0 + t * 35.0) as u8;

    let hash = ((wx.wrapping_mul(73856093)) ^ (wy.wrapping_mul(19349663))).wrapping_abs();
    if hash % 80 == 0 {
        let brightness = (60 + (hash % 120) as u8).min(255);
        return [brightness, brightness, brightness + 20];
    }

    [base_r, base_g, base_b]
}

fn entity_priority(kind: crate::entity::EntityKind) -> u32 {
    use crate::entity::EntityKind;
    match kind {
        EntityKind::Player => 3,
        EntityKind::Goblin => 2,
        EntityKind::Slime => 1,
        EntityKind::Corpse => 0,
    }
}

fn entity_positions(
    entities: &EntityManager,
    cam_x: i32,
    cam_y: i32,
    view_w: u32,
    view_h: u32,
) -> std::collections::HashMap<(i32, i32), [u8; 3]> {
    let mut map: std::collections::HashMap<(i32, i32), (u32, [u8; 3])> =
        std::collections::HashMap::new();
    for e in entities.all() {
        for b in &e.bodies {
            if !b.alive {
                continue;
            }
            let sx = b.x as i32 - cam_x;
            let sy = b.y as i32 - cam_y;
            if sx < 0 || sx >= view_w as i32 || sy < 0 || sy >= view_h as i32 {
                continue;
            }
            let color = if e.on_fire {
                let flicker = b.fire_timer % 4;
                [255, 120 + flicker as u8 * 20, 20 + flicker as u8 * 10]
            } else {
                [b.color[0], b.color[1], b.color[2]]
            };
            let priority = entity_priority(e.kind);
            if map
                .get(&(sx, sy))
                .map(|(p, _)| priority > *p)
                .unwrap_or(true)
            {
                map.insert((sx, sy), (priority, color));
            }
        }
    }
    map.into_iter().map(|(k, (_, c))| (k, c)).collect()
}

fn shadow_positions(
    entity_positions: &std::collections::HashMap<(i32, i32), [u8; 3]>,
    grid: &ChunkedGrid,
    cam_x: i32,
    cam_y: i32,
    view_w: u32,
    view_h: u32,
) -> std::collections::HashSet<(i32, i32)> {
    let mut shadows = std::collections::HashSet::new();
    for (vx, vy) in entity_positions.keys() {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let sx = vx + dx;
                let sy = vy + dy;
                if sx < 0 || sx >= view_w as i32 || sy < 0 || sy >= view_h as i32 {
                    continue;
                }
                if entity_positions.contains_key(&(sx, sy)) {
                    continue;
                }
                let wx = cam_x + sx;
                let wy = cam_y + sy;
                let empty = !grid.in_bounds(wx, wy) || grid.get(wx, wy).is_empty();
                if empty {
                    shadows.insert((sx, sy));
                }
            }
        }
    }
    shadows
}

fn item_color_at(items: &ItemManager, wx: i32, wy: i32) -> Option<[u8; 3]> {
    for item in items.all() {
        if item.x == wx && item.y == wy {
            return Some(item.color());
        }
    }
    None
}

fn draw_cell(img: &mut RgbImage, vx: u32, vy: u32, color: [u8; 3]) {
    let base_x = vx * CELL_SIZE;
    let base_y = vy * CELL_SIZE;
    for dy in 0..CELL_SIZE {
        for dx in 0..CELL_SIZE {
            let px = base_x + dx;
            let py = base_y + dy;
            if px < img.width() && py < img.height() {
                img.put_pixel(px, py, image::Rgb(color));
            }
        }
    }
}

pub fn save_capture(
    path: &str,
    grid: &ChunkedGrid,
    entities: &EntityManager,
    items: &ItemManager,
    ui: &UiLayer,
    cam_x: i32,
    cam_y: i32,
    view_w: u32,
    view_h: u32,
    lighting: Option<&lighting::LightGrid>,
) -> Result<(), String> {
    let img = capture_frame(
        grid, entities, items, ui, cam_x, cam_y, view_w, view_h, lighting,
    );
    img.save(path).map_err(|e| format!("save capture: {e}"))?;
    Ok(())
}

pub fn capture_from_state(
    grid: &ChunkedGrid,
    entities: &EntityManager,
    items: &ItemManager,
    ui: &UiLayer,
    cam_x: i32,
    cam_y: i32,
    path: &str,
    lighting: Option<&lighting::LightGrid>,
) -> Result<(), String> {
    let view_w = (grid.width as u32 / CELL_SIZE).min(256);
    let view_h = (grid.height as u32 / CELL_SIZE).min(256);
    save_capture(
        path, grid, entities, items, ui, cam_x, cam_y, view_w, view_h, lighting,
    )
}

pub fn capture_from_game(game: &crate::game::Game, path: &str) -> Result<(), String> {
    let (px, py) = game.player.center(&game.entities);
    let view_w = (game.grid.width as u32 / CELL_SIZE).min(256);
    let view_h = (game.grid.height as u32 / CELL_SIZE).min(256);
    let cam_x = px as i32 - (view_w as i32 / 2);
    let cam_y = py as i32 - (view_h as i32 / 2);
    let light = lighting::compute_lighting(
        &game.grid,
        cam_x,
        cam_y,
        view_w as usize,
        view_h as usize,
        lighting::ambient_light(),
    );
    save_capture(
        path,
        &game.grid,
        &game.entities,
        &game.items,
        &game.ui,
        cam_x,
        cam_y,
        view_w,
        view_h,
        Some(&light),
    )
}
