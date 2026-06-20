use serde::{Deserialize, Serialize};
use crate::ai::state::material_from_name;
use crate::ai::state::parse_entity_kind;
use crate::entity::EntityKind;
use crate::game::Game;
use crate::world::cell::{Cell, MaterialId};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AiAction {
    MoveLeft,
    MoveRight,
    Jump,
    Wait,
    Paint {
        x: i32,
        y: i32,
        material: String,
        radius: i32,
    },
    SetCell {
        x: i32,
        y: i32,
        material: String,
    },
    FillRect {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        material: String,
    },
    ClearRegion {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    },
    Spawn {
        kind: String,
        x: f32,
        y: f32,
    },
    KillEntity {
        id: u32,
    },
    DamageEntity {
        id: u32,
        amount: f32,
    },
    SetGravity {
        value: f32,
    },
    SetCamera {
        x: i32,
        y: i32,
    },
    CenterCamera,
}

impl AiAction {
    pub fn name(&self) -> &'static str {
        match self {
            AiAction::MoveLeft => "move_left",
            AiAction::MoveRight => "move_right",
            AiAction::Jump => "jump",
            AiAction::Wait => "wait",
            AiAction::Paint { .. } => "paint",
            AiAction::SetCell { .. } => "set_cell",
            AiAction::FillRect { .. } => "fill_rect",
            AiAction::ClearRegion { .. } => "clear_region",
            AiAction::Spawn { .. } => "spawn",
            AiAction::KillEntity { .. } => "kill_entity",
            AiAction::DamageEntity { .. } => "damage_entity",
            AiAction::SetGravity { .. } => "set_gravity",
            AiAction::SetCamera { .. } => "set_camera",
            AiAction::CenterCamera => "center_camera",
        }
    }

    pub fn execute(&self, game: &mut Game) {
        match self {
            AiAction::MoveLeft => {
                game.player.move_left(&mut game.entities);
            }
            AiAction::MoveRight => {
                game.player.move_right(&mut game.entities);
            }
            AiAction::Jump => {
                let on_ground = game.check_on_ground();
                game.player.jump(&mut game.entities, on_ground);
            }
            AiAction::Wait => {}
            AiAction::Paint { x, y, material, radius } => {
                if let Some(mat) = material_from_name(material) {
                    for dy in -*radius..=*radius {
                        for dx in -*radius..=*radius {
                            if dx * dx + dy * dy <= radius * radius + 1 {
                                game.grid.set_material(*x + dx, *y + dy, mat);
                            }
                        }
                    }
                }
            }
            AiAction::SetCell { x, y, material } => {
                if let Some(mat) = material_from_name(material) {
                    game.grid.set_material(*x, *y, mat);
                }
            }
            AiAction::FillRect { x, y, w, h, material } => {
                if let Some(mat) = material_from_name(material) {
                    for dy in 0..*h {
                        for dx in 0..*w {
                            game.grid.set_material(*x + dx, *y + dy, mat);
                        }
                    }
                }
            }
            AiAction::ClearRegion { x, y, w, h } => {
                for dy in 0..*h {
                    for dx in 0..*w {
                        game.grid.set(*x + dx, *y + dy, Cell::empty());
                    }
                }
            }
            AiAction::Spawn { kind, x, y } => {
                if let Some(k) = parse_entity_kind(kind) {
                    let id = game.entities.spawn(k);
                    if let Some(e) = game.entities.get_mut(id) {
                        e.build_humanoid(*x, *y);
                    }
                }
            }
            AiAction::KillEntity { id } => {
                if let Some(e) = game.entities.get_mut(*id) {
                    e.kill();
                }
            }
            AiAction::DamageEntity { id, amount } => {
                if let Some(e) = game.entities.get_mut(*id) {
                    e.take_damage(*amount);
                }
            }
            AiAction::SetGravity { value } => {
                game.verlet.gravity = *value;
            }
            AiAction::SetCamera { x, y } => {
                game.cam_x = *x;
                game.cam_y = *y;
            }
            AiAction::CenterCamera => {
                let (px, py) = game.player.center(&game.entities);
                game.center_camera_on(px, py);
            }
        }
    }
}
