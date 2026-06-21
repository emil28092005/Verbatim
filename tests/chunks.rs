use verbatim::world::cell::{Cell, MaterialId};
use verbatim::world::chunk::{CHUNK_SIZE, Chunk, world_to_chunk};
use verbatim::world::grid::Grid;

#[test]
fn grid_has_expected_chunks() {
    let g = Grid::new();
    assert_eq!(g.chunk_size, CHUNK_SIZE);
    assert_eq!(g.chunks_x, 4);
    assert_eq!(g.chunks_y, 4);
    assert_eq!(g.chunks.len(), 16);
    assert!(g.chunks.iter().all(|c| c.active));
}

#[test]
fn world_to_chunk_mapping() {
    assert_eq!(world_to_chunk(0, 0), (0, 0, 0, 0));
    assert_eq!(world_to_chunk(63, 63), (0, 0, 63, 63));
    assert_eq!(world_to_chunk(64, 64), (1, 1, 0, 0));
    assert_eq!(world_to_chunk(100, 118), (1, 1, 36, 54));
}

#[test]
fn chunk_local_get_set() {
    let mut c = Chunk::new();
    assert!(c.get(0, 0).is_empty());
    c.set(0, 0, Cell::new(MaterialId::Sand));
    assert_eq!(c.get(0, 0).material, MaterialId::Sand);
    assert!(c.modified);
}

#[test]
fn grid_get_set_marks_chunk_modified() {
    let mut g = Grid::new();
    g.set(10, 10, Cell::new(MaterialId::Water));
    assert!(g.chunks[g.chunk_index(0, 0)].modified);
    assert!(!g.chunks[g.chunk_index(1, 1)].modified);
    g.set(70, 70, Cell::new(MaterialId::Lava));
    assert!(g.chunks[g.chunk_index(1, 1)].modified);
}

#[test]
fn cell_serialization_roundtrip() {
    let c = Cell::new(MaterialId::Wood);
    let bytes = c.to_bytes();
    let c2 = Cell::from_bytes(&bytes);
    assert_eq!(c.material, c2.material);
    assert_eq!(c.fg, c2.fg);
    assert_eq!(c.bg, c2.bg);
    assert_eq!(c.variant, c2.variant);
    assert!((c.temp - c2.temp).abs() < 0.001);
}

#[test]
fn save_and_load_chunk_roundtrip() {
    let mut g = Grid::new();
    g.set(70, 70, Cell::new(MaterialId::Sand));
    g.set(71, 70, Cell::new(MaterialId::Water));
    let path = "/tmp/verbatim_chunk_test_1_1.bin";
    let _ = std::fs::remove_file(path);
    g.save_chunk(path, 1, 1).unwrap();
    let mut g2 = Grid::new();
    g2.load_chunk(path, 1, 1).unwrap();
    assert_eq!(g2.get(70, 70).material, MaterialId::Sand);
    assert_eq!(g2.get(71, 70).material, MaterialId::Water);
    assert!(g2.chunks[g2.chunk_index(1, 1)].active);
    let _ = std::fs::remove_file(path);
}

#[test]
fn inactive_chunk_cells_are_skipped() {
    let mut g = Grid::new();
    g.set(10, 10, Cell::new(MaterialId::Sand));
    g.set(10, 11, Cell::new(MaterialId::Empty));
    g.deactivate_all();
    assert!(!g.cell_active(10, 10));
    assert!(!g.cell_active(10, 11));
    g.set_chunk_active(0, 0, true);
    assert!(g.cell_active(10, 10));
}

#[test]
fn chunk_bounds_clip_to_world() {
    let g = Grid::new();
    let (x0, y0, x1, y1) = g.chunk_bounds(3, 3);
    assert_eq!(x0, 192);
    assert_eq!(y0, 192);
    assert_eq!(x1, 250);
    assert_eq!(y1, 250);
}
