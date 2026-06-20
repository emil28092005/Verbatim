use crate::entity::entity::{Entity, EntityKind, EntityManager, EntityId};

pub struct Player {
    pub entity_id: EntityId,
    pub move_speed: f32,
    pub jump_force: f32,
}

impl Player {
    pub fn new(manager: &mut EntityManager) -> Self {
        let id = manager.spawn(EntityKind::Player);
        Self {
            entity_id: id,
            move_speed: 0.3,
            jump_force: 1.2,
        }
    }

    pub fn spawn_at(&self, manager: &mut EntityManager, cx: f32, cy: f32) {
        if let Some(e) = manager.get_mut(self.entity_id) {
            e.build_humanoid(cx, cy);
        }
    }

    pub fn move_left(&self, manager: &mut EntityManager) {
        if let Some(e) = manager.get_mut(self.entity_id) {
            if let Some(head) = e.head_mut() {
                head.add_vel(-self.move_speed, 0.0);
            }
        }
    }

    pub fn move_right(&self, manager: &mut EntityManager) {
        if let Some(e) = manager.get_mut(self.entity_id) {
            if let Some(head) = e.head_mut() {
                head.add_vel(self.move_speed, 0.0);
            }
        }
    }

    pub fn jump(&self, manager: &mut EntityManager, on_ground: bool) {
        if on_ground {
            if let Some(e) = manager.get_mut(self.entity_id) {
                for b in &mut e.bodies {
                    if b.alive {
                        b.add_vel(0.0, -self.jump_force);
                    }
                }
            }
        }
    }

    pub fn entity<'a>(&self, manager: &'a EntityManager) -> Option<&'a Entity> {
        manager.get(self.entity_id)
    }

    pub fn entity_mut<'a>(&self, manager: &'a mut EntityManager) -> Option<&'a mut Entity> {
        manager.get_mut(self.entity_id)
    }

    pub fn center(&self, manager: &EntityManager) -> (f32, f32) {
        manager.get(self.entity_id).map(|e| e.center()).unwrap_or((0.0, 0.0))
    }
}
