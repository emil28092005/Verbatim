use crate::world::cell::MaterialId;
use crate::world::chunked_grid::ChunkedGrid;

#[derive(Clone, Copy, Debug)]
pub struct LightSource {
    pub x: i32,
    pub y: i32,
    pub color: [u8; 3],
    pub intensity: f32,
    pub radius: u32,
}

pub struct LightGrid {
    pub width: usize,
    pub height: usize,
    pub data: Vec<[u8; 3]>,
}

impl LightGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let data = vec![[0; 3]; width * height];
        Self {
            width,
            height,
            data,
        }
    }

    pub fn get(&self, x: i32, y: i32) -> [u8; 3] {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return [0, 0, 0];
        }
        self.data[y as usize * self.width + x as usize]
    }

    pub fn set(&mut self, x: i32, y: i32, value: [u8; 3]) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        self.data[y as usize * self.width + x as usize] = value;
    }

    pub fn clear(&mut self, ambient: [u8; 3]) {
        for v in self.data.iter_mut() {
            *v = ambient;
        }
    }
}

pub fn material_light(material: MaterialId) -> Option<LightSource> {
    match material {
        MaterialId::Lava => Some(LightSource {
            x: 0,
            y: 0,
            color: [255, 80, 20],
            intensity: 1.0,
            radius: 24,
        }),
        MaterialId::Fire => Some(LightSource {
            x: 0,
            y: 0,
            color: [255, 160, 40],
            intensity: 1.0,
            radius: 18,
        }),
        _ => None,
    }
}

pub fn gather_sources(grid: &ChunkedGrid) -> Vec<LightSource> {
    let mut sources = Vec::new();
    let w = grid.width;
    let h = grid.height;
    for y in 0..h {
        for x in 0..w {
            let cell = grid.get(x as i32, y as i32);
            if let Some(mut src) = material_light(cell.material) {
                src.x = x as i32;
                src.y = y as i32;
                sources.push(src);
            }
        }
    }
    sources
}

pub fn gather_sources_in_range(
    grid: &ChunkedGrid,
    cam_x: i32,
    cam_y: i32,
    view_w: usize,
    view_h: usize,
    margin: i32,
) -> Vec<LightSource> {
    let mut sources = Vec::new();
    let min_x = (cam_x - margin).max(0);
    let max_x = if grid.is_infinite() {
        cam_x + view_w as i32 + margin
    } else {
        (cam_x + view_w as i32 + margin).min(grid.width as i32)
    };
    let min_y = (cam_y - margin).max(0);
    let max_y = if grid.is_infinite() {
        cam_y + view_h as i32 + margin
    } else {
        (cam_y + view_h as i32 + margin).min(grid.height as i32)
    };
    for y in min_y..max_y {
        for x in min_x..max_x {
            let cell = grid.get(x, y);
            if let Some(mut src) = material_light(cell.material) {
                src.x = x;
                src.y = y;
                sources.push(src);
            }
        }
    }
    sources
}

pub fn compute_lighting(
    grid: &ChunkedGrid,
    cam_x: i32,
    cam_y: i32,
    view_w: usize,
    view_h: usize,
    ambient: [u8; 3],
) -> LightGrid {
    let mut grid_light = LightGrid::new(view_w, view_h);
    grid_light.clear(ambient);

    let sources = gather_sources_in_range(grid, cam_x, cam_y, view_w, view_h, 30);
    let cap = sources.len().min(32);
    let radius_limit = 30u32;

    for src in sources.iter().take(cap) {
        let r = src.radius.min(radius_limit) as i32;
        let r2 = r * r;
        let sx = src.x;
        let sy = src.y;

        for dy in -r..=r {
            for dx in -r..=r {
                let d2 = dx * dx + dy * dy;
                if d2 > r2 {
                    continue;
                }
                let tx = sx + dx;
                let ty = sy + dy;
                if !grid.in_bounds(tx, ty) {
                    continue;
                }
                let vx = tx - cam_x;
                let vy = ty - cam_y;
                if vx < 0 || vx >= view_w as i32 || vy < 0 || vy >= view_h as i32 {
                    continue;
                }
                if !line_of_sight(grid, sx, sy, tx, ty) {
                    continue;
                }
                let dist = (d2 as f32).sqrt();
                let t = 1.0 - (dist / r as f32);
                if t <= 0.0 {
                    continue;
                }
                let attenuation = t * t;
                let contrib = [
                    (src.color[0] as f32 * src.intensity * attenuation),
                    (src.color[1] as f32 * src.intensity * attenuation),
                    (src.color[2] as f32 * src.intensity * attenuation),
                ];
                let idx = vy as usize * view_w + vx as usize;
                let cur = grid_light.data[idx];
                let next = [
                    (cur[0] as f32 + contrib[0]).min(255.0) as u8,
                    (cur[1] as f32 + contrib[1]).min(255.0) as u8,
                    (cur[2] as f32 + contrib[2]).min(255.0) as u8,
                ];
                grid_light.data[idx] = next;
            }
        }
    }

    grid_light
}

pub fn line_of_sight(grid: &ChunkedGrid, x0: i32, y0: i32, x1: i32, y1: i32) -> bool {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        if x == x1 && y == y1 {
            return true;
        }
        if grid.in_bounds(x, y) && grid.get(x, y).is_solid() {
            return false;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

pub fn apply_light(color: [u8; 3], light: [u8; 3]) -> [u8; 3] {
    [
        ((color[0] as f32 * light[0] as f32 / 255.0).min(255.0) as u8),
        ((color[1] as f32 * light[1] as f32 / 255.0).min(255.0) as u8),
        ((color[2] as f32 * light[2] as f32 / 255.0).min(255.0) as u8),
    ]
}

pub fn apply_light_rgba(color: [u8; 4], light: [u8; 3]) -> [u8; 4] {
    [
        ((color[0] as f32 * light[0] as f32 / 255.0).min(255.0) as u8),
        ((color[1] as f32 * light[1] as f32 / 255.0).min(255.0) as u8),
        ((color[2] as f32 * light[2] as f32 / 255.0).min(255.0) as u8),
        color[3],
    ]
}

pub fn apply_light_tuple(color: (u8, u8, u8), light: [u8; 3]) -> (u8, u8, u8) {
    (
        ((color.0 as f32 * light[0] as f32 / 255.0).min(255.0) as u8),
        ((color.1 as f32 * light[1] as f32 / 255.0).min(255.0) as u8),
        ((color.2 as f32 * light[2] as f32 / 255.0).min(255.0) as u8),
    )
}

pub fn ambient_light() -> [u8; 3] {
    [160, 160, 180]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::chunked_grid::ChunkedGrid;

    fn grid_with_lava() -> (ChunkedGrid, i32, i32) {
        let mut grid = ChunkedGrid::with_size(250, 250);
        grid.set_material(10, 10, MaterialId::Lava);
        (grid, 10, 10)
    }

    #[test]
    fn lava_emits_light() {
        let (grid, x, y) = grid_with_lava();
        let sources = gather_sources(&grid);
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].x, x);
        assert_eq!(sources[0].y, y);
    }

    #[test]
    fn light_attenuates_with_distance() {
        let (grid, _, _) = grid_with_lava();
        let light = compute_lighting(&grid, 0, 0, 20, 20, ambient_light());
        let center = light.get(10, 10);
        let far = light.get(10, 0);
        assert!(
            center.iter().map(|&v| v as u32).sum::<u32>()
                > far.iter().map(|&v| v as u32).sum::<u32>()
        );
    }

    #[test]
    fn walls_block_light() {
        let mut grid = ChunkedGrid::with_size(250, 250);
        grid.set_material(5, 10, MaterialId::Lava);
        for y in 7..13 {
            grid.set_material(8, y, MaterialId::Stone);
        }
        let light = compute_lighting(&grid, 0, 0, 20, 20, ambient_light());
        let lit_side = light.get(6, 10);
        let shadow_side = light.get(10, 10);
        assert!(
            lit_side.iter().map(|&v| v as u32).sum::<u32>()
                > shadow_side.iter().map(|&v| v as u32).sum::<u32>()
        );
    }
}
