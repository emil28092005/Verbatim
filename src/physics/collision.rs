use crate::world::cell::MaterialId;
use crate::world::grid::Grid;
use crate::physics::verlet::SubBody;

pub struct CollisionResult {
    pub on_ground: bool,
    pub in_liquid: bool,
    pub liquid_density: f32,
    pub touching_lava: bool,
    pub touching_fire: bool,
    pub touching_acid: bool,
}

impl CollisionResult {
    pub fn none() -> Self {
        Self {
            on_ground: false,
            in_liquid: false,
            liquid_density: 0.0,
            touching_lava: false,
            touching_fire: false,
            touching_acid: false,
        }
    }
}

pub fn resolve_grid_collision(grid: &Grid, body: &mut SubBody) -> CollisionResult {
    let mut result = CollisionResult::none();

    let r = body.radius;
    let min_x = (body.x - r).floor() as i32;
    let max_x = (body.x + r).ceil() as i32;
    let min_y = (body.y - r).floor() as i32;
    let max_y = (body.y + r).ceil() as i32;

    for cy in min_y..=max_y {
        for cx in min_x..=max_x {
            if !grid.in_bounds(cx, cy) {
                continue;
            }
            let cell = grid.get(cx, cy);
            if cell.is_empty() {
                continue;
            }

            if cell.is_liquid() {
                result.in_liquid = true;
                result.liquid_density = result.liquid_density.max(cell.density());
                if cell.material == MaterialId::Lava {
                    result.touching_lava = true;
                }
                if cell.material == MaterialId::Acid {
                    result.touching_acid = true;
                }
                apply_liquid_drag(body, cell.density());
                continue;
            }

            if cell.material == MaterialId::Fire {
                result.touching_fire = true;
                continue;
            }

            if cell.is_solid() {
                let closest_x = body.x.max(cx as f32).min((cx + 1) as f32);
                let closest_y = body.y.max(cy as f32).min((cy + 1) as f32);
                let dx = body.x - closest_x;
                let dy = body.y - closest_y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq < r * r {
                    let dist = dist_sq.sqrt();
                    if dist > 0.0001 {
                        let overlap = r - dist;
                        let nx = dx / dist;
                        let ny = dy / dist;
                        body.x += nx * overlap;
                        body.y += ny * overlap;
                        if ny < -0.5 {
                            result.on_ground = true;
                        }
                    } else {
                        let bcx = cx as f32 + 0.5;
                        let bcy = cy as f32 + 0.5;
                        let dx = body.x - bcx;
                        let dy = body.y - bcy;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist > 0.0001 {
                            body.x = bcx + dx / dist * r * 1.1;
                            body.y = bcy + dy / dist * r * 1.1;
                        }
                    }
                }
            }
        }
    }

    result
}

fn apply_liquid_drag(body: &mut SubBody, density: f32) {
    let drag = 1.0 - density * 0.08;
    let drag = drag.max(0.5);
    let vx = body.vx() * drag;
    let vy = body.vy() * drag;
    body.set_vel(vx, vy);
}
