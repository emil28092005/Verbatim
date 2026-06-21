pub mod capture;
pub mod graphics;
pub mod lighting;
pub mod terminal;
pub mod vulkan;
pub mod window_input;

use crate::entity::item::ItemManager;
use crate::entity::EntityManager;
use crate::ui::UiLayer;
use crate::world::grid::Grid;

pub trait Renderer {
    fn init(&mut self) -> std::io::Result<()>;
    fn render(
        &mut self,
        grid: &Grid,
        entities: &EntityManager,
        items: &ItemManager,
        ui: &UiLayer,
        cam_x: i32,
        cam_y: i32,
        lighting: Option<&lighting::LightGrid>,
    ) -> std::io::Result<()>;
    fn shutdown(&mut self) -> std::io::Result<()>;
    fn viewport_w(&self) -> usize;
    fn viewport_h(&self) -> usize;
}
