use crate::entity::entity::{Entity, EntityKind};
use crate::physics::verlet::SubBody;
use crate::world::cell::MaterialId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BodyPart {
    pub x: f32,
    pub y: f32,
    pub color: [u8; 4],
    pub label: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BodyTemplate {
    pub name: String,
    pub half_w: f32,
    pub half_h: f32,
    pub radius: f32,
    pub parts: Vec<BodyPart>,
    pub constraints: Vec<(usize, usize)>,
}

impl BodyTemplate {
    pub fn humanoid_player() -> Self {
        Self {
            name: "player".to_string(),
            half_w: 2.5,
            half_h: 3.5,
            radius: 0.5,
            parts: vec![
                BodyPart {
                    x: 0.0,
                    y: -3.0,
                    color: [220, 200, 240, 255],
                    label: "head".into(),
                },
                BodyPart {
                    x: 0.0,
                    y: -2.0,
                    color: [220, 200, 240, 255],
                    label: "head".into(),
                },
                BodyPart {
                    x: 0.0,
                    y: -1.0,
                    color: [140, 80, 200, 255],
                    label: "shirt".into(),
                },
                BodyPart {
                    x: -1.0,
                    y: -1.0,
                    color: [210, 185, 235, 255],
                    label: "hand".into(),
                },
                BodyPart {
                    x: 1.0,
                    y: -1.0,
                    color: [210, 185, 235, 255],
                    label: "hand".into(),
                },
                BodyPart {
                    x: 0.0,
                    y: 0.0,
                    color: [140, 80, 200, 255],
                    label: "shirt".into(),
                },
                BodyPart {
                    x: -1.0,
                    y: 0.0,
                    color: [210, 185, 235, 255],
                    label: "hand".into(),
                },
                BodyPart {
                    x: 1.0,
                    y: 0.0,
                    color: [210, 185, 235, 255],
                    label: "hand".into(),
                },
                BodyPart {
                    x: 0.0,
                    y: 1.0,
                    color: [90, 50, 150, 255],
                    label: "pants".into(),
                },
                BodyPart {
                    x: -1.0,
                    y: 2.0,
                    color: [90, 50, 150, 255],
                    label: "pants".into(),
                },
                BodyPart {
                    x: 1.0,
                    y: 2.0,
                    color: [90, 50, 150, 255],
                    label: "pants".into(),
                },
                BodyPart {
                    x: 0.0,
                    y: 2.0,
                    color: [90, 50, 150, 255],
                    label: "pants".into(),
                },
                BodyPart {
                    x: -1.0,
                    y: 3.0,
                    color: [60, 35, 100, 255],
                    label: "boots".into(),
                },
                BodyPart {
                    x: 0.0,
                    y: 3.0,
                    color: [60, 35, 100, 255],
                    label: "boots".into(),
                },
                BodyPart {
                    x: 1.0,
                    y: 3.0,
                    color: [60, 35, 100, 255],
                    label: "boots".into(),
                },
            ],
            constraints: vec![
                (0, 1),
                (1, 2),
                (2, 5),
                (5, 8),
                (2, 3),
                (2, 4),
                (5, 6),
                (5, 7),
                (8, 11),
                (11, 9),
                (11, 10),
                (9, 12),
                (10, 14),
                (11, 13),
            ],
        }
    }

    pub fn humanoid_goblin() -> Self {
        Self {
            name: "goblin".to_string(),
            half_w: 2.5,
            half_h: 3.5,
            radius: 0.5,
            parts: vec![
                BodyPart {
                    x: 0.0,
                    y: -3.0,
                    color: [130, 210, 100, 255],
                    label: "head".into(),
                },
                BodyPart {
                    x: 0.0,
                    y: -2.0,
                    color: [130, 210, 100, 255],
                    label: "head".into(),
                },
                BodyPart {
                    x: 0.0,
                    y: -1.0,
                    color: [55, 130, 45, 255],
                    label: "shirt".into(),
                },
                BodyPart {
                    x: -1.0,
                    y: -1.0,
                    color: [110, 180, 85, 255],
                    label: "hand".into(),
                },
                BodyPart {
                    x: 1.0,
                    y: -1.0,
                    color: [110, 180, 85, 255],
                    label: "hand".into(),
                },
                BodyPart {
                    x: 0.0,
                    y: 0.0,
                    color: [55, 130, 45, 255],
                    label: "shirt".into(),
                },
                BodyPart {
                    x: -1.0,
                    y: 0.0,
                    color: [110, 180, 85, 255],
                    label: "hand".into(),
                },
                BodyPart {
                    x: 1.0,
                    y: 0.0,
                    color: [110, 180, 85, 255],
                    label: "hand".into(),
                },
                BodyPart {
                    x: 0.0,
                    y: 1.0,
                    color: [40, 100, 30, 255],
                    label: "pants".into(),
                },
                BodyPart {
                    x: -1.0,
                    y: 2.0,
                    color: [40, 100, 30, 255],
                    label: "pants".into(),
                },
                BodyPart {
                    x: 1.0,
                    y: 2.0,
                    color: [40, 100, 30, 255],
                    label: "pants".into(),
                },
                BodyPart {
                    x: 0.0,
                    y: 2.0,
                    color: [40, 100, 30, 255],
                    label: "pants".into(),
                },
                BodyPart {
                    x: -1.0,
                    y: 3.0,
                    color: [25, 70, 18, 255],
                    label: "boots".into(),
                },
                BodyPart {
                    x: 0.0,
                    y: 3.0,
                    color: [25, 70, 18, 255],
                    label: "boots".into(),
                },
                BodyPart {
                    x: 1.0,
                    y: 3.0,
                    color: [25, 70, 18, 255],
                    label: "boots".into(),
                },
            ],
            constraints: vec![
                (0, 1),
                (1, 2),
                (2, 5),
                (5, 8),
                (2, 3),
                (2, 4),
                (5, 6),
                (5, 7),
                (8, 11),
                (11, 9),
                (11, 10),
                (9, 12),
                (10, 14),
                (11, 13),
            ],
        }
    }

    pub fn boulder() -> Self {
        Self {
            name: "boulder".to_string(),
            half_w: 2.5,
            half_h: 2.5,
            radius: 0.5,
            parts: vec![
                BodyPart {
                    x: -1.0,
                    y: -2.0,
                    color: [100, 100, 110, 255],
                    label: "rock".into(),
                },
                BodyPart {
                    x: 0.0,
                    y: -2.0,
                    color: [110, 110, 120, 255],
                    label: "rock".into(),
                },
                BodyPart {
                    x: 1.0,
                    y: -2.0,
                    color: [100, 100, 110, 255],
                    label: "rock".into(),
                },
                BodyPart {
                    x: -2.0,
                    y: -1.0,
                    color: [105, 105, 115, 255],
                    label: "rock".into(),
                },
                BodyPart {
                    x: -1.0,
                    y: -1.0,
                    color: [115, 115, 125, 255],
                    label: "rock".into(),
                },
                BodyPart {
                    x: 0.0,
                    y: -1.0,
                    color: [115, 115, 125, 255],
                    label: "rock".into(),
                },
                BodyPart {
                    x: 1.0,
                    y: -1.0,
                    color: [115, 115, 125, 255],
                    label: "rock".into(),
                },
                BodyPart {
                    x: 2.0,
                    y: -1.0,
                    color: [105, 105, 115, 255],
                    label: "rock".into(),
                },
                BodyPart {
                    x: -2.0,
                    y: 0.0,
                    color: [100, 100, 110, 255],
                    label: "rock".into(),
                },
                BodyPart {
                    x: -1.0,
                    y: 0.0,
                    color: [110, 110, 120, 255],
                    label: "rock".into(),
                },
                BodyPart {
                    x: 0.0,
                    y: 0.0,
                    color: [120, 120, 130, 255],
                    label: "rock".into(),
                },
                BodyPart {
                    x: 1.0,
                    y: 0.0,
                    color: [110, 110, 120, 255],
                    label: "rock".into(),
                },
                BodyPart {
                    x: 2.0,
                    y: 0.0,
                    color: [100, 100, 110, 255],
                    label: "rock".into(),
                },
                BodyPart {
                    x: -1.0,
                    y: 1.0,
                    color: [95, 95, 105, 255],
                    label: "rock".into(),
                },
                BodyPart {
                    x: 0.0,
                    y: 1.0,
                    color: [105, 105, 115, 255],
                    label: "rock".into(),
                },
                BodyPart {
                    x: 1.0,
                    y: 1.0,
                    color: [95, 95, 105, 255],
                    label: "rock".into(),
                },
            ],
            constraints: vec![
                (0, 1),
                (1, 2),
                (0, 3),
                (1, 4),
                (2, 5),
                (3, 4),
                (4, 5),
                (5, 6),
                (3, 7),
                (6, 7),
                (3, 8),
                (4, 9),
                (5, 10),
                (6, 11),
                (7, 12),
                (8, 9),
                (9, 10),
                (10, 11),
                (11, 12),
                (8, 13),
                (9, 14),
                (10, 15),
                (14, 15),
                (13, 14),
            ],
        }
    }

    pub fn apply_to(&self, entity: &mut Entity, cx: f32, cy: f32) {
        entity.bodies.clear();
        entity.constraints.clear();
        entity.rest_offsets.clear();
        entity.cx = cx;
        entity.cy = cy;
        entity.cvx = 0.0;
        entity.cvy = 0.0;
        entity.half_w = self.half_w;
        entity.half_h = self.half_h;

        let mat = MaterialId::Flesh;
        for part in &self.parts {
            entity.rest_offsets.push((part.x, part.y));
            let mut b = SubBody::new(cx + part.x, cy + part.y, self.radius, mat);
            b.color = part.color;
            entity.bodies.push(b);
        }

        for &(a, b) in &self.constraints {
            let dx = entity.bodies[a].x - entity.bodies[b].x;
            let dy = entity.bodies[a].y - entity.bodies[b].y;
            let len = (dx * dx + dy * dy).sqrt().max(0.5);
            entity
                .constraints
                .push(crate::physics::verlet::Constraint::new(a, b, len, 1.0));
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }

    pub fn preview_ascii(&self) -> String {
        if self.parts.is_empty() {
            return "(empty template)".to_string();
        }

        let min_x = self.parts.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        let max_x = self
            .parts
            .iter()
            .map(|p| p.x)
            .fold(f32::NEG_INFINITY, f32::max);
        let min_y = self.parts.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let max_y = self
            .parts
            .iter()
            .map(|p| p.y)
            .fold(f32::NEG_INFINITY, f32::max);

        let w = ((max_x - min_x) as i32 + 3) as usize;
        let h = ((max_y - min_y) as i32 + 3) as usize;

        let mut grid = vec![vec![' '; w]; h];

        let label_colors: std::collections::HashMap<&str, char> =
            std::collections::HashMap::from([
                ("head", 'O'),
                ("shirt", '#'),
                ("hand", '+'),
                ("pants", '|'),
                ("boots", '"'),
                ("rock", '*'),
                ("skin", 'O'),
                ("body", '#'),
                ("leg", '|'),
                ("arm", '+'),
                ("wing", '~'),
                ("tail", '.'),
            ]);

        for part in &self.parts {
            let gx = (part.x - min_x) as i32 + 1;
            let gy = (part.y - min_y) as i32 + 1;
            if gx >= 0 && gx < w as i32 && gy >= 0 && gy < h as i32 {
                let ch = label_colors.get(part.label.as_str()).unwrap_or(&'#');
                grid[gy as usize][gx as usize] = *ch;
            }
        }

        let mut out = String::new();
        for row in grid {
            out.push_str(&row.iter().collect::<String>());
            out.push('\n');
        }
        out
    }
}

pub fn template_for_kind(kind: EntityKind) -> BodyTemplate {
    match kind {
        EntityKind::Player => BodyTemplate::humanoid_player(),
        EntityKind::Goblin => BodyTemplate::humanoid_goblin(),
        EntityKind::Corpse => BodyTemplate::humanoid_player(),
    }
}
