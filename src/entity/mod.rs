pub mod entity;
pub mod player;
pub mod body_template;

pub use entity::{EntityManager, EntityKind};
pub use player::Player;
pub use body_template::{BodyTemplate, BodyPart, template_for_kind};
