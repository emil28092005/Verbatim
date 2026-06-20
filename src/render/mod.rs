pub mod terminal;
pub mod window_input;
pub mod vulkan;

use crate::entity::EntityManager;
use crate::world::grid::Grid;

pub trait Renderer {
    fn init(&mut self) -> std::io::Result<()>;
    fn render(&mut self, grid: &Grid, entities: &EntityManager, cam_x: i32, cam_y: i32) -> std::io::Result<()>;
    fn shutdown(&mut self) -> std::io::Result<()>;
    fn viewport_w(&self) -> usize;
    fn viewport_h(&self) -> usize;
}
