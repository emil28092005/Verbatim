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

    for _iteration in 0..3 {
        let mut any_collision = false;

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
                    let cell_min_x = cx as f32;
                    let cell_max_x = (cx + 1) as f32;
                    let cell_min_y = cy as f32;
                    let cell_max_y = (cy + 1) as f32;

                    let inside_x = body.x >= cell_min_x && body.x < cell_max_x;
                    let inside_y = body.y >= cell_min_y && body.y < cell_max_y;

                    if inside_x && inside_y {
                        let dist_left = body.x - cell_min_x;
                        let dist_right = cell_max_x - body.x;
                        let dist_top = body.y - cell_min_y;
                        let dist_bottom = cell_max_y - body.y;

                        let min_dist = dist_left.min(dist_right).min(dist_top).min(dist_bottom);

                        if min_dist == dist_top {
                            body.y = cell_min_y - r;
                            result.on_ground = true;
                        } else if min_dist == dist_bottom {
                            body.y = cell_max_y + r;
                        } else if min_dist == dist_left {
                            body.x = cell_min_x - r;
                        } else {
                            body.x = cell_max_x + r;
                        }

                        let vy = body.vy();
                        if min_dist == dist_top && vy > 0.0 {
                            body.set_vel(body.vx(), 0.0);
                        }
                        let vx = body.vx();
                        if (min_dist == dist_left || min_dist == dist_right) && vx != 0.0 {
                            body.set_vel(0.0, body.vy());
                        }

                        any_collision = true;
                    } else {
                        let closest_x = body.x.max(cell_min_x).min(cell_max_x);
                        let closest_y = body.y.max(cell_min_y).min(cell_max_y);
                        let dx = body.x - closest_x;
                        let dy = body.y - closest_y;
                        let dist_sq = dx * dx + dy * dy;

                        if dist_sq < r * r && dist_sq > 0.0001 {
                            let dist = dist_sq.sqrt();
                            let overlap = r - dist;
                            let nx = dx / dist;
                            let ny = dy / dist;
                            body.x += nx * overlap;
                            body.y += ny * overlap;

                            if ny < -0.5 {
                                result.on_ground = true;
                                if body.vy() > 0.0 {
                                    body.set_vel(body.vx(), 0.0);
                                }
                            }

                            any_collision = true;
                        }
                    }
                }
            }
        }

        if !any_collision {
            break;
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
