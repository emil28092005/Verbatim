use crate::entity::entity::{Entity, EntityKind};
use crate::physics::verlet::SubBody;
use crate::world::cell::MaterialId;
use serde::{Deserialize, Serialize};

macro_rules! p {
    ($x:expr, $y:expr, $r:expr, $g:expr, $b:expr, $label:expr) => {
        BodyPart {
            x: $x as f32,
            y: $y as f32,
            color: [$r, $g, $b, 255],
            label: $label.into(),
        }
    };
}

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
        let parts = vec![
            p!(-1, -6, 210, 185, 235, "head"),
            p!(0, -6, 225, 205, 245, "head"),
            p!(1, -6, 210, 185, 235, "head"),
            p!(-1, -5, 215, 190, 240, "head"),
            p!(0, -5, 230, 210, 250, "head"),
            p!(1, -5, 215, 190, 240, "head"),
            p!(-2, -5, 195, 165, 220, "head"),
            p!(2, -5, 195, 165, 220, "head"),
            p!(0, -5, 40, 30, 50, "eye"),
            p!(-1, -5, 60, 40, 70, "eye"),
            p!(1, -5, 60, 40, 70, "eye"),
            p!(-3, -6, 120, 80, 40, "hair"),
            p!(-2, -7, 140, 90, 50, "hair"),
            p!(-1, -7, 150, 100, 55, "hair"),
            p!(0, -7, 150, 100, 55, "hair"),
            p!(1, -7, 140, 90, 50, "hair"),
            p!(-3, -5, 110, 70, 35, "hair"),
            p!(-4, -5, 100, 60, 30, "hair"),
            p!(-2, -4, 80, 50, 130, "hat"),
            p!(2, -4, 80, 50, 130, "hat"),
            p!(-1, -5, 90, 55, 140, "hat"),
            p!(0, -5, 100, 60, 150, "hat"),
            p!(1, -5, 90, 55, 140, "hat"),
            p!(0, -4, 230, 190, 110, "belt"),
            p!(-1, -4, 150, 90, 210, "shirt"),
            p!(1, -4, 150, 90, 210, "shirt"),
            p!(-2, -4, 205, 180, 235, "hand"),
            p!(2, -4, 205, 180, 235, "hand"),
            p!(-3, -4, 195, 170, 225, "hand"),
            p!(3, -4, 195, 170, 225, "hand"),
            p!(-1, -3, 150, 90, 210, "shirt"),
            p!(0, -3, 160, 100, 220, "shirt"),
            p!(1, -3, 150, 90, 210, "shirt"),
            p!(-2, -3, 210, 185, 240, "hand"),
            p!(2, -3, 210, 185, 240, "hand"),
            p!(3, -3, 220, 195, 245, "weapon"),
            p!(4, -3, 230, 205, 250, "weapon"),
            p!(-1, -2, 150, 90, 210, "shirt"),
            p!(0, -2, 160, 100, 220, "shirt"),
            p!(1, -2, 150, 90, 210, "shirt"),
            p!(-2, -2, 200, 170, 230, "hand"),
            p!(2, -2, 200, 170, 230, "hand"),
            p!(-4, -2, 80, 50, 130, "cape"),
            p!(-3, -2, 75, 45, 120, "cape"),
            p!(-1, -1, 150, 90, 210, "shirt"),
            p!(0, -1, 160, 100, 220, "shirt"),
            p!(1, -1, 150, 90, 210, "shirt"),
            p!(-4, -1, 70, 40, 110, "cape"),
            p!(-3, -1, 70, 40, 110, "cape"),
            p!(-1, 0, 230, 190, 110, "belt"),
            p!(0, 0, 240, 200, 120, "belt"),
            p!(1, 0, 230, 190, 110, "belt"),
            p!(-4, 0, 60, 35, 100, "cape"),
            p!(-3, 0, 65, 38, 105, "cape"),
            p!(-1, 1, 95, 55, 155, "pants"),
            p!(0, 1, 95, 55, 155, "pants"),
            p!(1, 1, 95, 55, 155, "pants"),
            p!(-1, 2, 85, 45, 145, "pants"),
            p!(0, 2, 85, 45, 145, "pants"),
            p!(1, 2, 85, 45, 145, "pants"),
            p!(-2, 2, 85, 45, 145, "pants"),
            p!(2, 2, 85, 45, 145, "pants"),
            p!(-2, 3, 75, 40, 125, "pants"),
            p!(0, 3, 75, 40, 125, "pants"),
            p!(2, 3, 75, 40, 125, "pants"),
            p!(-2, 4, 55, 30, 95, "boots"),
            p!(0, 4, 55, 30, 95, "boots"),
            p!(2, 4, 55, 30, 95, "boots"),
            p!(-2, 5, 45, 25, 80, "boots"),
            p!(2, 5, 45, 25, 80, "boots"),
        ];

        let constraints = Self::proximity_constraints(1.5)(&parts);
        Self {
            name: "player".to_string(),
            half_w: 4.0,
            half_h: 6.0,
            radius: 0.5,
            parts,
            constraints,
        }
    }

    pub fn humanoid_goblin() -> Self {
        let parts = vec![
            p!(-1, -6, 120, 200, 95, "head"),
            p!(0, -6, 130, 210, 100, "head"),
            p!(1, -6, 120, 200, 95, "head"),
            p!(-1, -5, 125, 205, 97, "head"),
            p!(0, -5, 135, 215, 102, "head"),
            p!(1, -5, 125, 205, 97, "head"),
            p!(-2, -5, 110, 190, 85, "head"),
            p!(2, -5, 110, 190, 85, "head"),
            p!(-1, -5, 200, 50, 50, "eye"),
            p!(1, -5, 200, 50, 50, "eye"),
            p!(-2, -5, 120, 80, 50, "ear"),
            p!(2, -5, 120, 80, 50, "ear"),
            p!(-3, -5, 110, 70, 40, "ear"),
            p!(3, -5, 110, 70, 40, "ear"),
            p!(-1, -4, 90, 180, 70, "tooth"),
            p!(0, -4, 90, 180, 70, "tooth"),
            p!(1, -4, 90, 180, 70, "tooth"),
            p!(0, -4, 100, 180, 75, "skin"),
            p!(-1, -4, 100, 180, 75, "skin"),
            p!(1, -4, 100, 180, 75, "skin"),
            p!(-1, -3, 55, 130, 45, "shirt"),
            p!(1, -3, 55, 130, 45, "shirt"),
            p!(0, -3, 65, 140, 50, "shirt"),
            p!(-2, -3, 110, 180, 85, "hand"),
            p!(2, -3, 110, 180, 85, "hand"),
            p!(-3, -3, 100, 170, 75, "hand"),
            p!(3, -3, 100, 170, 75, "hand"),
            p!(4, -3, 180, 160, 60, "weapon"),
            p!(-1, -2, 55, 130, 45, "shirt"),
            p!(0, -2, 65, 140, 50, "shirt"),
            p!(1, -2, 55, 130, 45, "shirt"),
            p!(-2, -2, 110, 180, 85, "hand"),
            p!(2, -2, 110, 180, 85, "hand"),
            p!(-1, -1, 50, 120, 40, "shirt"),
            p!(0, -1, 60, 130, 45, "shirt"),
            p!(1, -1, 50, 120, 40, "shirt"),
            p!(-2, -1, 100, 170, 75, "hand"),
            p!(2, -1, 100, 170, 75, "hand"),
            p!(-1, 0, 40, 90, 30, "pants"),
            p!(0, 0, 45, 100, 32, "pants"),
            p!(1, 0, 40, 90, 30, "pants"),
            p!(-1, 1, 40, 100, 30, "pants"),
            p!(0, 1, 40, 100, 30, "pants"),
            p!(1, 1, 40, 100, 30, "pants"),
            p!(-1, 2, 35, 85, 25, "pants"),
            p!(1, 2, 35, 85, 25, "pants"),
            p!(-2, 2, 35, 85, 25, "pants"),
            p!(2, 2, 35, 85, 25, "pants"),
            p!(-2, 3, 110, 180, 85, "skin"),
            p!(0, 3, 110, 180, 85, "skin"),
            p!(2, 3, 110, 180, 85, "skin"),
            p!(-2, 4, 100, 170, 75, "skin"),
            p!(2, 4, 100, 170, 75, "skin"),
        ];

        let constraints = Self::proximity_constraints(1.5)(&parts);
        Self {
            name: "goblin".to_string(),
            half_w: 4.0,
            half_h: 6.0,
            radius: 0.5,
            parts,
            constraints,
        }
    }

    pub fn slime() -> Self {
        let parts = vec![
            p!(-2, -1, 60, 180, 80, "body"),
            p!(-1, -2, 80, 200, 100, "body"),
            p!(0, -2, 90, 210, 110, "body"),
            p!(1, -2, 80, 200, 100, "body"),
            p!(2, -1, 60, 180, 80, "body"),
            p!(-2, 0, 70, 190, 90, "body"),
            p!(-1, 0, 100, 220, 130, "body"),
            p!(0, 0, 110, 230, 140, "body"),
            p!(1, 0, 100, 220, 130, "body"),
            p!(2, 0, 70, 190, 90, "body"),
            p!(-2, 1, 50, 170, 70, "body"),
            p!(-1, 1, 80, 200, 100, "body"),
            p!(0, 1, 90, 210, 110, "body"),
            p!(1, 1, 80, 200, 100, "body"),
            p!(2, 1, 50, 170, 70, "body"),
            p!(-1, 2, 40, 160, 60, "body"),
            p!(0, 2, 50, 170, 70, "body"),
            p!(1, 2, 40, 160, 60, "body"),
            p!(-1, -1, 120, 230, 150, "eye"),
            p!(1, -1, 120, 230, 150, "eye"),
        ];

        let n = parts.len();
        Self {
            name: "slime".to_string(),
            half_w: 3.0,
            half_h: 2.0,
            radius: 0.5,
            parts,
            constraints: Self::auto_constraints(n),
        }
    }

    pub fn boulder() -> Self {
        let parts = vec![
            p!(-1, -2, 100, 100, 110, "rock"),
            p!(0, -2, 115, 115, 125, "rock"),
            p!(1, -2, 100, 100, 110, "rock"),
            p!(-2, -1, 105, 105, 115, "rock"),
            p!(-1, -1, 120, 120, 130, "rock"),
            p!(0, -1, 125, 125, 135, "rock"),
            p!(1, -1, 120, 120, 130, "rock"),
            p!(2, -1, 105, 105, 115, "rock"),
            p!(-2, 0, 100, 100, 110, "rock"),
            p!(-1, 0, 115, 115, 125, "rock"),
            p!(0, 0, 130, 130, 140, "rock"),
            p!(1, 0, 115, 115, 125, "rock"),
            p!(2, 0, 100, 100, 110, "rock"),
            p!(-1, 1, 95, 95, 105, "rock"),
            p!(0, 1, 110, 110, 120, "rock"),
            p!(1, 1, 95, 95, 105, "rock"),
        ];

        let n = parts.len();
        Self {
            name: "boulder".to_string(),
            half_w: 2.5,
            half_h: 2.5,
            radius: 0.5,
            parts,
            constraints: Self::auto_constraints(n),
        }
    }

    fn auto_constraints(n: usize) -> Vec<(usize, usize)> {
        let mut c = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                c.push((i, j));
            }
        }
        c
    }

    fn proximity_constraints(max_dist: f32) -> impl Fn(&[BodyPart]) -> Vec<(usize, usize)> {
        move |parts| {
            let mut c = Vec::new();
            let threshold = max_dist * max_dist;
            for i in 0..parts.len() {
                for j in (i + 1)..parts.len() {
                    let dx = parts[i].x - parts[j].x;
                    let dy = parts[i].y - parts[j].y;
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq <= threshold {
                        c.push((i, j));
                    }
                }
            }
            c
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

        let label_chars: std::collections::HashMap<&str, char> = std::collections::HashMap::from([
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
            ("hat", '^'),
            ("belt", '='),
            ("eye", ':'),
            ("hair", '~'),
            ("cape", '>'),
            ("tooth", '.'),
            ("ear", '<'),
            ("weapon", '/'),
        ]);

        for part in &self.parts {
            let gx = (part.x - min_x) as i32 + 1;
            let gy = (part.y - min_y) as i32 + 1;
            if gx >= 0 && gx < w as i32 && gy >= 0 && gy < h as i32 {
                let ch = label_chars.get(part.label.as_str()).unwrap_or(&'#');
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
        EntityKind::Slime => BodyTemplate::slime(),
        EntityKind::Corpse => BodyTemplate::humanoid_player(),
    }
}
