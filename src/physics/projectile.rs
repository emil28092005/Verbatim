use crate::entity::entity::{Entity, EntityId};
use crate::world::cell::{Cell, MaterialId};
use crate::world::chunked_grid::ChunkedGrid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectileType {
    Arrow,
    Fireball,
    MagicBolt,
}

pub struct Projectile {
    pub id: u32,
    pub typ: ProjectileType,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub radius: f32,
    pub damage: f32,
    pub lifetime: u32,
    pub max_lifetime: u32,
    pub owner: EntityId,
    pub alive: bool,
    pub gravity: bool,
    pub hit_grid: bool,
    pub damage_bonus: f32,
}

impl Projectile {
    pub fn new(
        id: u32,
        typ: ProjectileType,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        owner: EntityId,
    ) -> Self {
        let (radius, damage, max_lifetime, gravity) = match typ {
            ProjectileType::Arrow => (0.3, 15.0, 120, true),
            ProjectileType::Fireball => (0.6, 12.0, 90, false),
            ProjectileType::MagicBolt => (0.25, 20.0, 100, false),
        };
        Self {
            id,
            typ,
            x,
            y,
            vx,
            vy,
            radius,
            damage,
            lifetime: 0,
            max_lifetime,
            owner,
            alive: true,
            gravity,
            hit_grid: false,
            damage_bonus: 0.0,
        }
    }

    pub fn total_damage(&self) -> f32 {
        self.damage + self.damage_bonus
    }

    pub fn update(&mut self, grid: &ChunkedGrid) {
        if !self.alive {
            return;
        }
        self.lifetime += 1;
        if self.lifetime >= self.max_lifetime {
            self.alive = false;
            return;
        }
        if self.gravity {
            self.vy += 0.04;
        }
        let speed = (self.vx * self.vx + self.vy * self.vy).sqrt();
        if speed > 4.0 {
            self.vx = self.vx / speed * 4.0;
            self.vy = self.vy / speed * 4.0;
        }
        self.x += self.vx;
        self.y += self.vy;

        let min_x = (self.x - self.radius).floor() as i32;
        let max_x = (self.x + self.radius).ceil() as i32;
        let min_y = (self.y - self.radius).floor() as i32;
        let max_y = (self.y + self.radius).ceil() as i32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if !grid.in_bounds(x, y) {
                    self.hit_grid = true;
                    self.alive = false;
                    return;
                }
                let cell = grid.get(x, y);
                if cell.is_solid() {
                    self.hit_grid = true;
                    self.alive = false;
                    return;
                }
            }
        }
    }

    pub fn check_entity_hit(&self, entity: &Entity) -> bool {
        if !self.alive || !entity.alive || entity.id == self.owner {
            return false;
        }
        let dx = self.x - entity.cx;
        let dy = self.y - entity.cy;
        let hit_w = entity.half_w + self.radius;
        let hit_h = entity.half_h + self.radius;
        dx.abs() < hit_w && dy.abs() < hit_h
    }

    pub fn apply_impact(
        &self,
        grid: &mut ChunkedGrid,
        entity: &mut Entity,
        ui: &mut crate::ui::UiLayer,
    ) {
        if self.typ == ProjectileType::Fireball {
            let min_x = (self.x - 1.5).floor() as i32;
            let max_x = (self.x + 1.5).ceil() as i32;
            let min_y = (self.y - 1.5).floor() as i32;
            let max_y = (self.y + 1.5).ceil() as i32;
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    if !grid.in_bounds(x, y) {
                        continue;
                    }
                    let cell = grid.get(x, y);
                    if cell.is_empty() {
                        grid.set(x, y, Cell::new(MaterialId::Fire));
                    } else if cell.material == MaterialId::Wood
                        || cell.material == MaterialId::Grass
                        || cell.material == MaterialId::Flesh
                    {
                        let mut ignited = cell;
                        ignited.material = MaterialId::Fire;
                        ignited.temp = 400.0;
                        grid.set(x, y, ignited);
                    }
                }
            }
            if entity.id != self.owner {
                let before = entity.health;
                entity.take_damage(self.total_damage());
                ui.add_damage_number(
                    entity.cx,
                    entity.cy - entity.half_h - 2.0,
                    &format!("-{:.0}", before - entity.health),
                );
            }
            return;
        }

        if entity.id == self.owner {
            return;
        }

        let before = entity.health;
        if self.typ == ProjectileType::MagicBolt {
            entity.take_damage(self.total_damage());
        } else {
            entity.take_damage(self.total_damage());
            let dir = if self.vx < 0.0 { -1.0 } else { 1.0 };
            entity.set_horizontal_vel(dir * 0.6);
        }
        ui.add_damage_number(
            entity.cx,
            entity.cy - entity.half_h - 2.0,
            &format!("-{:.0}", before - entity.health),
        );
    }

    pub fn draw_char(&self) -> char {
        match self.typ {
            ProjectileType::Arrow => '/',
            ProjectileType::Fireball => 'o',
            ProjectileType::MagicBolt => '*',
        }
    }

    pub fn draw_color(&self) -> [u8; 3] {
        match self.typ {
            ProjectileType::Arrow => [200, 200, 200],
            ProjectileType::Fireball => [255, 120, 30],
            ProjectileType::MagicBolt => [120, 200, 255],
        }
    }
}

pub struct ProjectileManager {
    projectiles: Vec<Projectile>,
    next_id: u32,
}

impl ProjectileManager {
    pub fn new() -> Self {
        Self {
            projectiles: Vec::new(),
            next_id: 0,
        }
    }

    pub fn spawn(
        &mut self,
        typ: ProjectileType,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        owner: EntityId,
        damage_bonus: f32,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let mut p = Projectile::new(id, typ, x, y, vx, vy, owner);
        p.damage_bonus = damage_bonus;
        self.projectiles.push(p);
        id
    }

    pub fn spawn_arrow(&mut self, x: f32, y: f32, vx: f32, vy: f32, owner: EntityId) -> u32 {
        self.spawn(ProjectileType::Arrow, x, y, vx, vy, owner, 0.0)
    }

    pub fn update(&mut self, grid: &ChunkedGrid) {
        for p in &mut self.projectiles {
            p.update(grid);
        }
    }

    pub fn resolve_hits(
        &mut self,
        grid: &mut ChunkedGrid,
        entities: &mut [Entity],
        ui: &mut crate::ui::UiLayer,
    ) {
        for p in &mut self.projectiles {
            if !p.alive && !p.hit_grid {
                continue;
            }
            let mut hit = false;
            for e in entities.iter_mut() {
                if p.check_entity_hit(e) {
                    let was_alive = e.alive;
                    p.apply_impact(grid, e, ui);
                    if was_alive && !e.alive {
                        ui.add_message(&format!("{} dies!", e.name()));
                    } else if was_alive {
                        ui.add_message(&format!(
                            "{} hit for {:.0} damage",
                            e.name(),
                            p.total_damage()
                        ));
                    }
                    hit = true;
                    break;
                }
            }
            if !hit && p.hit_grid && p.typ == ProjectileType::Fireball {
                let mut dummy = Entity::new(p.owner, crate::entity::entity::EntityKind::Player);
                p.apply_impact(grid, &mut dummy, ui);
            }
            if hit || p.hit_grid {
                p.alive = false;
            }
        }
    }

    pub fn cull_dead(&mut self) {
        self.projectiles.retain(|p| p.alive);
    }

    pub fn all(&self) -> &[Projectile] {
        &self.projectiles
    }
}
