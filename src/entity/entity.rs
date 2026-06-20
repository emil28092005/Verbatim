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
    pub alive: bool,
    pub health: f32,
    pub max_health: f32,
    pub constraint_stiffness: f32,
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
            alive: true,
            health: 100.0,
            max_health: 100.0,
            constraint_stiffness: 1.0,
            on_fire: false,
            fire_timer: 0,
        }
    }

    pub fn center(&self) -> (f32, f32) {
        if self.bodies.is_empty() {
            return (0.0, 0.0);
        }
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

    pub fn head(&self) -> Option<&SubBody> {
        self.bodies.first()
    }

    pub fn head_mut(&mut self) -> Option<&mut SubBody> {
        self.bodies.first_mut()
    }

    pub fn kill(&mut self) {
        self.alive = false;
        self.health = 0.0;
        self.constraint_stiffness = 0.0;
        for c in &mut self.constraints {
            c.stiffness = 0.0;
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

        let r = 0.4;
        let mat = match self.kind {
            EntityKind::Player => MaterialId::Flesh,
            EntityKind::Goblin => MaterialId::Flesh,
            EntityKind::Corpse => MaterialId::Flesh,
        };

        self.bodies.push(SubBody::new(cx, cy - 2.0, r, mat));
        self.bodies.push(SubBody::new(cx, cy - 1.0, r, mat));
        self.bodies.push(SubBody::new(cx - 0.8, cy - 1.0, r, mat));
        self.bodies.push(SubBody::new(cx + 0.8, cy - 1.0, r, mat));
        self.bodies.push(SubBody::new(cx - 0.5, cy, r, mat));
        self.bodies.push(SubBody::new(cx + 0.5, cy, r, mat));
        self.bodies.push(SubBody::new(cx, cy + 1.0, r, MaterialId::Bone));

        let s = self.constraint_stiffness;
        let mk = |a: usize, b: usize, len: f32| Constraint::new(a, b, len, s);

        self.constraints.push(mk(0, 1, 1.0));
        self.constraints.push(mk(1, 2, 0.9));
        self.constraints.push(mk(1, 3, 0.9));
        self.constraints.push(mk(2, 4, 0.9));
        self.constraints.push(mk(3, 5, 0.9));
        self.constraints.push(mk(1, 6, 2.0));
        self.constraints.push(mk(4, 5, 1.0));
        self.constraints.push(mk(4, 6, 1.1));
        self.constraints.push(mk(5, 6, 1.1));
    }

    pub fn apply_fire_damage(&mut self) {
        if !self.on_fire {
            return;
        }
        self.fire_timer += 1;
        let dmg = 0.5;
        self.take_damage(dmg);
        for b in &mut self.bodies {
            if b.alive {
                b.health -= dmg;
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
