pub mod body_template;
pub mod entity;
pub mod item;
pub mod player;

pub use body_template::{template_for_kind, BodyPart, BodyTemplate};
pub use entity::{EntityKind, EntityManager};
pub use item::{Item, ItemManager, ItemType};
pub use player::Player;
