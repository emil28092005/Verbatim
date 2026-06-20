pub mod cell;
pub mod material;
pub mod grid;
pub mod cellular;

pub use cell::{Cell, MaterialId};
pub use material::{Material, MaterialRegistry};
pub use grid::{Grid, WORLD_W, WORLD_H};
pub use cellular::CellularAutomaton;
