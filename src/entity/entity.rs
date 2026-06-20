use crate::physics::verlet::{SubBody, Constraint};
use crate::world::cell::MaterialId;

pub type EntityId = u32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntityKind {
    Player,
    Goblin,
    Corpse,
}

pub struct Entity {
    pub id: EntityId,
    pub kind: EntityKind,
    pub bodies: Vec<SubBody>,
    pub constraints: Vec<Constraint>,
    pub rest_offsets: Vec<(f32, f32)>,
    pub alive: bool,
    pub rigid: bool,
    pub cx: f32,
    pub cy: f32,
    pub cvx: f32,
    pub cvy: f32,
    pub half_w: f32,
    pub half_h: f32,
    pub health: f32,
    pub max_health: f32,
    pub on_fire: bool,
    pub fire_timer: u32,
}

impl Entity {
    pub fn new(id: EntityId, kind: EntityKind) -> Self {
        Self {
            id,
            kind,
            bodies: Vec::new(),
            constraints: Vec::new(),
            rest_offsets: Vec::new(),
            alive: true,
            rigid: true,
            cx: 0.0,
            cy: 0.0,
            cvx: 0.0,
            cvy: 0.0,
            health: 100.0,
            max_health: 100.0,
            on_fire: false,
            fire_timer: 0,
            half_w: 3.5,
            half_h: 3.0,
        }
    }

    pub fn center(&self) -> (f32, f32) {
        if self.rigid {
            (self.cx, self.cy)
        } else if self.bodies.is_empty() {
            (0.0, 0.0)
        } else {
            let mut sx = 0.0;
            let mut sy = 0.0;
            let mut n = 0;
            for b in &self.bodies {
                if b.alive {
                    sx += b.x;
                    sy += b.y;
                    n += 1;
                }
            }
            if n > 0 {
                (sx / n as f32, sy / n as f32)
            } else {
                (self.bodies[0].x, self.bodies[0].y)
            }
        }
    }

    pub fn kill(&mut self) {
        self.alive = false;
        self.rigid = false;
        self.health = 0.0;
        for c in &mut self.constraints {
            c.stiffness = 0.0;
        }
        let (cvx, cvy) = (self.cvx, self.cvy);
        for b in &mut self.bodies {
            if b.alive {
                b.set_vel(cvx + (b.x - self.cx) * 0.3, cvy + (b.y - self.cy) * 0.3);
            }
        }
        if self.kind == EntityKind::Player || self.kind == EntityKind::Goblin {
            self.kind = EntityKind::Corpse;
        }
    }

    pub fn take_damage(&mut self, dmg: f32) {
        self.health -= dmg;
        if self.health <= 0.0 && self.alive {
            self.kill();
        }
    }

    pub fn build_humanoid(&mut self, cx: f32, cy: f32) {
        self.bodies.clear();
        self.constraints.clear();
        self.rest_offsets.clear();
        self.cx = cx;
        self.cy = cy;
        self.cvx = 0.0;
        self.cvy = 0.0;
        self.half_w = 3.5;
        self.half_h = 2.5;

        let r = 0.5;
        let mat = match self.kind {
            EntityKind::Player => MaterialId::Flesh,
            EntityKind::Goblin => MaterialId::Flesh,
            EntityKind::Corpse => MaterialId::Flesh,
        };

        // 5x5 body + 2 arms = 27 bodies
        let layout: [(f32, f32, MaterialId); 27] = [
            // Row 0 (top): head
            (-2.0, -2.0, mat), (-1.0, -2.0, mat), ( 0.0, -2.0, mat), ( 1.0, -2.0, mat), ( 2.0, -2.0, mat),
            // Row 1: shoulders
            (-2.0, -1.0, mat), (-1.0, -1.0, mat), ( 0.0, -1.0, mat), ( 1.0, -1.0, mat), ( 2.0, -1.0, mat),
            // Row 2: torso
            (-2.0,  0.0, mat), (-1.0,  0.0, mat), ( 0.0,  0.0, mat), ( 1.0,  0.0, mat), ( 2.0,  0.0, mat),
            // Row 3: hips
            (-2.0,  1.0, mat), (-1.0,  1.0, mat), ( 0.0,  1.0, mat), ( 1.0,  1.0, mat), ( 2.0,  1.0, mat),
            // Row 4: legs
            (-2.0,  2.0, mat), (-1.0,  2.0, mat), ( 0.0,  2.0, mat), ( 1.0,  2.0, mat), ( 2.0,  2.0, mat),
            // Arms
            ( 3.0, -1.0, mat), ( 3.0,  0.0, mat),
        ];

        for &(ox, oy, m) in &layout {
            self.rest_offsets.push((ox, oy));
            self.bodies.push(SubBody::new(cx + ox, cy + oy, r, m));
        }

        let mk = |a: usize, b: usize, len: f32| Constraint::new(a, b, len, 1.0);

        // Horizontal connections
        for row in 0..5 {
            let base = row * 5;
            for col in 0..4 {
                self.constraints.push(mk(base + col, base + col + 1, 1.0));
            }
        }
        // Vertical connections
        for col in 0..5 {
            for row in 0..4 {
                self.constraints.push(mk(row * 5 + col, (row + 1) * 5 + col, 1.0));
            }
        }
        // Arm connections
        self.constraints.push(mk(9, 25, 1.0));
        self.constraints.push(mk(25, 26, 1.0));
    }

    pub fn sync_bodies_to_center(&mut self) {
        for (i, b) in self.bodies.iter_mut().enumerate() {
            if !b.alive {
                continue;
            }
            let (ox, oy) = self.rest_offsets[i];
            b.x = self.cx + ox;
            b.y = self.cy + oy;
            b.old_x = b.x - self.cvx;
            b.old_y = b.y - self.cvy;
        }
    }

    pub fn move_center(&mut self, dx: f32, dy: f32) {
        if self.rigid {
            self.cvx += dx;
            self.cvy += dy;
        }
    }

    pub fn apply_fire_damage(&mut self) {
        if !self.on_fire {
            return;
        }
        self.fire_timer += 1;
        self.take_damage(0.5);
        for b in &mut self.bodies {
            if b.alive {
                b.health -= 0.5;
            }
        }
        if self.fire_timer > 180 {
            self.on_fire = false;
            self.fire_timer = 0;
        }
    }
}

pub struct EntityManager {
    entities: Vec<Entity>,
    next_id: EntityId,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            next_id: 0,
        }
    }

    pub fn spawn(&mut self, kind: EntityKind) -> EntityId {
        let id = self.next_id;
        self.next_id += 1;
        let mut e = Entity::new(id, kind);
        match kind {
            EntityKind::Player => {
                e.max_health = 100.0;
                e.health = 100.0;
            }
            EntityKind::Goblin => {
                e.max_health = 40.0;
                e.health = 40.0;
            }
            EntityKind::Corpse => {
                e.alive = false;
                e.rigid = false;
            }
        }
        self.entities.push(e);
        id
    }

    pub fn get(&self, id: EntityId) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id == id)
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|e| e.id == id)
    }

    pub fn all(&self) -> &[Entity] {
        &self.entities
    }

    pub fn all_mut(&mut self) -> &mut [Entity] {
        &mut self.entities
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
        self.entities.iter_mut()
    }
}
