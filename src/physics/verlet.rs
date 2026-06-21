use crate::world::cell::MaterialId;

#[derive(Clone, Copy, Debug)]
pub struct SubBody {
    pub x: f32,
    pub y: f32,
    pub old_x: f32,
    pub old_y: f32,
    pub ax: f32,
    pub ay: f32,
    pub radius: f32,
    pub material: MaterialId,
    pub alive: bool,
    pub health: f32,
    pub on_fire: bool,
    pub fire_timer: u32,
    pub color: [u8; 4],
}

impl SubBody {
    pub fn new(x: f32, y: f32, radius: f32, material: MaterialId) -> Self {
        Self {
            x,
            y,
            old_x: x,
            old_y: y,
            ax: 0.0,
            ay: 0.0,
            radius,
            material,
            alive: true,
            health: 100.0,
            on_fire: false,
            fire_timer: 0,
            color: [255, 255, 255, 255],
        }
    }

    #[inline]
    pub fn vx(&self) -> f32 {
        self.x - self.old_x
    }

    #[inline]
    pub fn vy(&self) -> f32 {
        self.y - self.old_y
    }

    #[inline]
    pub fn set_vel(&mut self, vx: f32, vy: f32) {
        self.old_x = self.x - vx;
        self.old_y = self.y - vy;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Constraint {
    pub a: usize,
    pub b: usize,
    pub rest_length: f32,
    pub stiffness: f32,
}

impl Constraint {
    pub fn new(a: usize, b: usize, rest_length: f32, stiffness: f32) -> Self {
        Self {
            a,
            b,
            rest_length,
            stiffness,
        }
    }
}

#[derive(Clone)]
pub struct VerletSolver {
    pub gravity: f32,
    pub damping: f32,
    pub dt: f32,
    pub max_vel: f32,
    pub substeps: u32,
}

impl VerletSolver {
    pub fn new() -> Self {
        Self {
            gravity: 0.04,
            damping: 0.97,
            dt: 1.0,
            max_vel: 2.0,
            substeps: 8,
        }
    }

    pub fn integrate(&self, bodies: &mut [SubBody]) {
        for b in bodies.iter_mut() {
            if !b.alive {
                continue;
            }
            let mut vx = (b.x - b.old_x) * self.damping;
            let mut vy = (b.y - b.old_y) * self.damping;

            let v_mag = (vx * vx + vy * vy).sqrt();
            if v_mag > self.max_vel {
                vx = vx / v_mag * self.max_vel;
                vy = vy / v_mag * self.max_vel;
            }

            b.old_x = b.x;
            b.old_y = b.y;
            b.x += vx + b.ax * self.dt * self.dt;
            b.y += vy + (b.ay + self.gravity) * self.dt * self.dt;
            b.ax = 0.0;
            b.ay = 0.0;
        }
    }

    pub fn solve_constraints(
        &self,
        bodies: &mut [SubBody],
        constraints: &[Constraint],
        iterations: u32,
    ) {
        const MAX_CORRECTION: f32 = 0.5;
        for _ in 0..iterations {
            for c in constraints {
                let (ba, bb) = if c.a < bodies.len() && c.b < bodies.len() {
                    (bodies[c.a], bodies[c.b])
                } else {
                    continue;
                };
                if !ba.alive || !bb.alive {
                    continue;
                }
                let dx = bb.x - ba.x;
                let dy = bb.y - ba.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < 0.0001 {
                    continue;
                }
                let diff = (dist - c.rest_length) / dist;
                let mut sx = dx * 0.5 * diff * c.stiffness;
                let mut sy = dy * 0.5 * diff * c.stiffness;
                let corr_mag = (sx * sx + sy * sy).sqrt();
                if corr_mag > MAX_CORRECTION {
                    let scale = MAX_CORRECTION / corr_mag;
                    sx *= scale;
                    sy *= scale;
                }
                bodies[c.a].x += sx;
                bodies[c.a].y += sy;
                bodies[c.b].x -= sx;
                bodies[c.b].y -= sy;
            }
        }
    }
}
