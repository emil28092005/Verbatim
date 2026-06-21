pub mod body_template;
pub mod entity;
pub mod item;
pub mod player;

pub use body_template::{BodyPart, BodyTemplate, template_for_kind};
pub use entity::{EntityKind, EntityManager};
pub use item::{Item, ItemManager, ItemType};
pub use player::Player;
