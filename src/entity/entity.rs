use crate::physics::verlet::{Constraint, SubBody};

pub type EntityId = u32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntityKind {
    Player,
    Goblin,
    Slime,
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
    pub poisoned: bool,
    pub poison_timer: u32,
    pub frozen: bool,
    pub frozen_timer: u32,
    pub bleeding: bool,
    pub bleeding_timer: u32,
    pub level: u32,
    pub xp: u32,
    pub strength: u32,
    pub agility: u32,
    pub toughness: u32,
    pub willpower: u32,
    pub counted_for_score: bool,
}

impl Clone for Entity {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind,
            bodies: self.bodies.clone(),
            constraints: self.constraints.clone(),
            rest_offsets: self.rest_offsets.clone(),
            alive: self.alive,
            rigid: self.rigid,
            cx: self.cx,
            cy: self.cy,
            cvx: self.cvx,
            cvy: self.cvy,
            half_w: self.half_w,
            half_h: self.half_h,
            health: self.health,
            max_health: self.max_health,
            on_fire: self.on_fire,
            fire_timer: self.fire_timer,
            poisoned: self.poisoned,
            poison_timer: self.poison_timer,
            frozen: self.frozen,
            frozen_timer: self.frozen_timer,
            bleeding: self.bleeding,
            bleeding_timer: self.bleeding_timer,
            level: self.level,
            xp: self.xp,
            strength: self.strength,
            agility: self.agility,
            toughness: self.toughness,
            willpower: self.willpower,
            counted_for_score: self.counted_for_score,
        }
    }
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
            poisoned: false,
            poison_timer: 0,
            frozen: false,
            frozen_timer: 0,
            bleeding: false,
            bleeding_timer: 0,
            level: 1,
            xp: 0,
            strength: 10,
            agility: 10,
            toughness: 10,
            willpower: 10,
            counted_for_score: false,
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
        if self.kind == EntityKind::Player
            || self.kind == EntityKind::Goblin
            || self.kind == EntityKind::Slime
        {
            self.kind = EntityKind::Corpse;
        }
    }

    pub fn take_damage(&mut self, dmg: f32) {
        self.health -= dmg;
        if self.health <= 0.0 && self.alive {
            self.kill();
        }
    }

    pub fn name(&self) -> &'static str {
        match self.kind {
            EntityKind::Player => "Player",
            EntityKind::Goblin => "Goblin",
            EntityKind::Slime => "Slime",
            EntityKind::Corpse => "Corpse",
        }
    }

    pub fn build_humanoid(&mut self, cx: f32, cy: f32) {
        let template = crate::entity::body_template::template_for_kind(self.kind);
        template.apply_to(self, cx, cy);
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

    pub fn set_horizontal_vel(&mut self, vx: f32) {
        if self.rigid {
            self.cvx = vx;
        }
    }

    pub fn set_vertical_vel(&mut self, vy: f32) {
        if self.rigid {
            self.cvy = vy;
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

    pub fn recalc_max_health(&mut self) {
        let base = match self.kind {
            EntityKind::Player => 80.0,
            EntityKind::Goblin => 30.0,
            EntityKind::Slime => 15.0,
            EntityKind::Corpse => 0.0,
        };
        self.max_health = base + self.toughness as f32 * 5.0 + self.level as f32 * 10.0;
        self.health = self.health.min(self.max_health);
    }

    pub fn xp_to_level(&self) -> u32 {
        self.level * 100
    }

    pub fn add_xp(&mut self, amount: u32) {
        if !self.alive {
            return;
        }
        self.xp += amount;
        while self.xp >= self.xp_to_level() {
            self.xp -= self.xp_to_level();
            self.level += 1;
            self.recalc_max_health();
            self.health = self.max_health;
        }
    }

    pub fn apply_status_effects(&mut self) {
        if self.poisoned {
            self.poison_timer += 1;
            self.take_damage(0.2);
            if self.poison_timer > 180 {
                self.poisoned = false;
                self.poison_timer = 0;
            }
        }
        if self.bleeding {
            self.bleeding_timer += 1;
            self.take_damage(0.3);
            if self.bleeding_timer > 120 {
                self.bleeding = false;
                self.bleeding_timer = 0;
            }
        }
        if self.frozen {
            self.frozen_timer += 1;
            if self.frozen_timer > 90 {
                self.frozen = false;
                self.frozen_timer = 0;
            }
        }
    }

    pub fn status_effects_active(&self) -> bool {
        self.on_fire || self.poisoned || self.frozen || self.bleeding
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
                e.strength = 12;
                e.agility = 12;
                e.toughness = 12;
                e.willpower = 12;
                e.recalc_max_health();
                e.health = e.max_health;
            }
            EntityKind::Goblin => {
                e.strength = 8;
                e.agility = 10;
                e.toughness = 8;
                e.willpower = 6;
                e.recalc_max_health();
                e.health = e.max_health;
            }
            EntityKind::Slime => {
                e.strength = 6;
                e.agility = 6;
                e.toughness = 8;
                e.willpower = 4;
                e.recalc_max_health();
                e.health = e.max_health;
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
}
