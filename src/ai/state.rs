use crate::entity::{EntityKind, EntityManager};
use crate::game::Game;
use crate::world::cell::MaterialId;
use crate::world::grid::Grid;
use crate::world::material::MaterialRegistry;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameState {
    pub tick: u64,
    pub world_size: [usize; 2],
    pub camera: [i32; 2],
    pub player: Option<EntityInfo>,
    pub entities: Vec<EntityInfo>,
    pub view: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EntityInfo {
    pub id: u32,
    pub kind: String,
    pub alive: bool,
    pub health: f32,
    pub max_health: f32,
    pub pos: [f32; 2],
    pub on_fire: bool,
    pub poisoned: bool,
    pub frozen: bool,
    pub bleeding: bool,
    pub level: u32,
    pub xp: u32,
    pub strength: u32,
    pub agility: u32,
    pub toughness: u32,
    pub willpower: u32,
    pub body_count: usize,
    pub bodies: Vec<SubBodyInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubBodyInfo {
    pub idx: usize,
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub health: f32,
    pub alive: bool,
    pub on_fire: bool,
    pub material: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CellInfo {
    pub x: i32,
    pub y: i32,
    pub material: String,
    pub temp: f32,
    pub is_solid: bool,
    pub is_liquid: bool,
    pub is_gas: bool,
}

impl CellInfo {
    pub fn from_grid(grid: &Grid, x: i32, y: i32) -> Self {
        if !grid.in_bounds(x, y) {
            return Self {
                x,
                y,
                material: "out_of_bounds".to_string(),
                temp: 0.0,
                is_solid: false,
                is_liquid: false,
                is_gas: false,
            };
        }
        let cell = grid.get(x, y);
        let reg = MaterialRegistry::instance();
        let mat = reg.get(cell.material);
        Self {
            x,
            y,
            material: mat.name.to_string(),
            temp: cell.temp,
            is_solid: mat.solid,
            is_liquid: mat.liquid,
            is_gas: mat.gas,
        }
    }
}

pub fn entity_info(e: &crate::entity::entity::Entity) -> EntityInfo {
    let (px, py) = e.center();
    let bodies: Vec<SubBodyInfo> = e
        .bodies
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let reg = MaterialRegistry::instance();
            SubBodyInfo {
                idx: i,
                pos: [b.x, b.y],
                vel: [b.vx(), b.vy()],
                health: b.health,
                alive: b.alive,
                on_fire: b.on_fire,
                material: reg.get(b.material).name.to_string(),
            }
        })
        .collect();

    EntityInfo {
        id: e.id,
        kind: entity_kind_name(e.kind).to_string(),
        alive: e.alive,
        health: e.health,
        max_health: e.max_health,
        pos: [px, py],
        on_fire: e.on_fire,
        poisoned: e.poisoned,
        frozen: e.frozen,
        bleeding: e.bleeding,
        level: e.level,
        xp: e.xp,
        strength: e.strength,
        agility: e.agility,
        toughness: e.toughness,
        willpower: e.willpower,
        body_count: bodies.iter().filter(|b| b.alive).count(),
        bodies,
    }
}

pub fn build_game_state(game: &Game, view_w: usize, view_h: usize) -> GameState {
    let player_info = game.player.entity(&game.entities).map(entity_info);

    let entities: Vec<EntityInfo> = game
        .entities
        .all()
        .iter()
        .filter(|e| e.id != game.player.entity_id)
        .map(entity_info)
        .collect();

    let (px, py) = game.player.center(&game.entities);
    let cam_x = px as i32 - (view_w as i32 / 2);
    let cam_y = py as i32 - (view_h as i32 / 2);

    let view = render_view(&game.grid, &game.entities, cam_x, cam_y, view_w, view_h);

    GameState {
        tick: game.tick,
        world_size: [game.grid.width, game.grid.height],
        camera: [cam_x, cam_y],
        player: player_info,
        entities,
        view,
    }
}

pub fn render_view(
    grid: &Grid,
    entities: &EntityManager,
    cam_x: i32,
    cam_y: i32,
    vw: usize,
    vh: usize,
) -> String {
    let mut entity_map = std::collections::HashMap::new();
    for e in entities.all() {
        for b in &e.bodies {
            if !b.alive {
                continue;
            }
            let sx = b.x as i32 - cam_x;
            let sy = b.y as i32 - cam_y;
            if sx >= 0 && sx < vw as i32 && sy >= 0 && sy < vh as i32 {
                let ch = match e.kind {
                    EntityKind::Player if e.alive => '@',
                    EntityKind::Goblin if e.alive => 'g',
                    EntityKind::Slime if e.alive => 's',
                    _ => '%',
                };
                entity_map.insert((sx, sy), ch);
            }
        }
    }

    let mut buf = String::with_capacity(vw * vh + vh);
    for dy in 0..vh {
        for dx in 0..vw {
            let x = cam_x + dx as i32;
            let y = cam_y + dy as i32;
            if let Some(&ch) = entity_map.get(&(dx as i32, dy as i32)) {
                buf.push(ch);
            } else if !grid.in_bounds(x, y) {
                buf.push('?');
            } else {
                let cell = grid.get(x, y);
                buf.push(cell.material.display_char());
            }
        }
        buf.push('\n');
    }
    buf
}

pub fn material_from_name(name: &str) -> Option<MaterialId> {
    match name.to_lowercase().as_str() {
        "empty" | "air" | " " => Some(MaterialId::Empty),
        "sand" => Some(MaterialId::Sand),
        "water" => Some(MaterialId::Water),
        "stone" => Some(MaterialId::Stone),
        "lava" => Some(MaterialId::Lava),
        "wood" => Some(MaterialId::Wood),
        "flesh" => Some(MaterialId::Flesh),
        "bone" => Some(MaterialId::Bone),
        "steam" => Some(MaterialId::Steam),
        "fire" => Some(MaterialId::Fire),
        "acid" => Some(MaterialId::Acid),
        "smoke" => Some(MaterialId::Smoke),
        "grass" => Some(MaterialId::Grass),
        "dirt" => Some(MaterialId::Dirt),
        "stairs" => Some(MaterialId::Stairs),
        _ => None,
    }
}

pub fn entity_kind_name(kind: EntityKind) -> &'static str {
    match kind {
        EntityKind::Player => "Player",
        EntityKind::Goblin => "Goblin",
        EntityKind::Slime => "Slime",
        EntityKind::Corpse => "Corpse",
    }
}

pub fn parse_entity_kind(name: &str) -> Option<EntityKind> {
    match name.to_lowercase().as_str() {
        "player" => Some(EntityKind::Player),
        "goblin" => Some(EntityKind::Goblin),
        "slime" => Some(EntityKind::Slime),
        "corpse" => Some(EntityKind::Corpse),
        _ => None,
    }
}
