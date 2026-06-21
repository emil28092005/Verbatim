use crate::entity::player::Player;
use crate::entity::{EntityManager, ItemManager, ItemType};
use crate::world::cell::{Cell, MaterialId};
use crate::world::cellular::CellularAutomaton;
use crate::world::chunk::CHUNK_SIZE;
use crate::world::chunked_grid::ChunkedGrid;

pub struct WorldGenerator<'a> {
    ca: &'a mut CellularAutomaton,
}

#[derive(Clone, Copy, Debug)]
struct Rect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl<'a> WorldGenerator<'a> {
    pub fn new(ca: &'a mut CellularAutomaton) -> Self {
        Self { ca }
    }

    pub fn generate(
        &mut self,
        grid: &mut ChunkedGrid,
        items: &mut ItemManager,
        player: &mut Player,
        entities: &mut EntityManager,
        depth: u32,
    ) -> (f32, f32) {
        let (px, py) = if grid.width <= 2048 && grid.height <= 2048 {
            if depth <= 3 {
                self.generate_surface(grid, depth)
            } else if depth <= 6 {
                self.generate_caves(grid, depth)
            } else {
                self.generate_dungeon(grid, items, depth)
            }
        } else {
            self.generate_spawn_region(grid, items, depth)
        };

        player.spawn_at(entities, px as f32, py as f32);
        if let Some(e) = player.entity_mut(entities) {
            e.facing_right = true;
        }

        self.place_items(grid, items, px, py, depth);
        (px as f32, py as f32)
    }

    fn generate_spawn_region(
        &mut self,
        grid: &mut ChunkedGrid,
        _items: &mut ItemManager,
        _depth: u32,
    ) -> (i32, i32) {
        let sx = if grid.is_infinite() {
            1000
        } else {
            (grid.width as i32 / 2).max(100)
        };
        let surface_y = self.surface_height_world(sx);
        let spawn_cx = sx / CHUNK_SIZE as i32;
        let spawn_cy = (surface_y / CHUNK_SIZE as i32).max(0);
        for dy in -1..=2 {
            for dx in -2..=2 {
                let cx = spawn_cx + dx;
                let cy = spawn_cy + dy;
                if grid.in_bounds(cx * CHUNK_SIZE as i32, cy * CHUNK_SIZE as i32) {
                    self.generate_chunk(grid, cx, cy);
                }
            }
        }
        grid.set_material(sx, surface_y, MaterialId::Stairs);
        if grid.get(sx, surface_y + 1).is_empty() {
            grid.set_material(sx, surface_y + 1, MaterialId::Stone);
        }
        let spawn_y = surface_y - 8;
        (sx, spawn_y)
    }

    pub fn generate_chunk(&mut self, grid: &mut ChunkedGrid, cx: i32, cy: i32) {
        if grid.is_chunk_generated(cx, cy) {
            return;
        }
        if cy < 2 {
            self.generate_surface_chunk(grid, cx, cy);
        } else if cy < 6 {
            self.generate_cave_chunk(grid, cx, cy);
        } else {
            self.generate_dungeon_chunk(grid, cx, cy);
        }
        if let Some(chunk) = grid.get_chunk_mut(cx, cy) {
            chunk.generated = true;
            chunk.modified = true;
            chunk.was_modified = true;
        }
    }

    fn generate_surface_chunk(&mut self, grid: &mut ChunkedGrid, cx: i32, cy: i32) {
        let x0 = cx * CHUNK_SIZE as i32;
        let y0 = cy * CHUNK_SIZE as i32;
        let mut surface = Vec::with_capacity(CHUNK_SIZE);
        for lx in 0..CHUNK_SIZE as i32 {
            let x = x0 + lx;
            let s = self.surface_height_world(x);
            surface.push(s);
            for y in y0..(y0 + CHUNK_SIZE as i32) {
                if y < s {
                    continue;
                }
                if y == s {
                    grid.set_material(x, y, MaterialId::Grass);
                } else if y > s + 8 {
                    grid.set_material(x, y, MaterialId::Stone);
                } else {
                    grid.set_material(x, y, MaterialId::Dirt);
                }
            }
        }
        let chunk_depth = (cy / 2).max(1) as u32;
        self.place_surface_features_chunk(grid, cx, cy, &surface, chunk_depth);
        grid.fill_border(MaterialId::Stone);
    }

    fn generate_cave_chunk(&mut self, grid: &mut ChunkedGrid, cx: i32, cy: i32) {
        let x0 = cx * CHUNK_SIZE as i32;
        let y0 = cy * CHUNK_SIZE as i32;
        let depth = (cy / 2).max(2) as u32;
        for y in y0..(y0 + CHUNK_SIZE as i32) {
            for x in x0..(x0 + CHUNK_SIZE as i32) {
                grid.set_material(x, y, MaterialId::Stone);
            }
        }
        let fill_prob = 0.42;
        let iterations = 4;
        self.carve_ca_caves_chunk(grid, cx, cy, fill_prob, iterations);
        let pool_count = (1 + depth / 3).min(3) as i32;
        for _ in 0..pool_count {
            let typ = self.random_cave_pool_type(depth);
            self.place_cave_pool_chunk(grid, cx, cy, typ, 2, 4);
        }
        grid.fill_border(MaterialId::Stone);
    }

    fn generate_dungeon_chunk(&mut self, grid: &mut ChunkedGrid, cx: i32, cy: i32) {
        let x0 = cx * CHUNK_SIZE as i32;
        let y0 = cy * CHUNK_SIZE as i32;
        for y in y0..(y0 + CHUNK_SIZE as i32) {
            for x in x0..(x0 + CHUNK_SIZE as i32) {
                grid.set_material(x, y, MaterialId::Stone);
            }
        }
        let root = Rect {
            x: x0 + 6,
            y: y0 + 6,
            w: CHUNK_SIZE as i32 - 12,
            h: CHUNK_SIZE as i32 - 12,
        };
        let tree = self.build_bsp(root);
        let mut rooms = Vec::new();
        self.collect_rooms(&tree, &mut rooms);
        for room in &rooms {
            self.carve_room(grid, *room);
        }
        self.connect_bsp_rooms(&tree, grid);
        grid.fill_border(MaterialId::Stone);
        if let Some(chunk) = grid.get_chunk_mut(cx, cy) {
            chunk.modified = true;
            chunk.was_modified = true;
        }
    }

    fn generate_surface(&mut self, grid: &mut ChunkedGrid, depth: u32) -> (i32, i32) {
        let w = grid.width as i32;
        let h = grid.height as i32;
        let mut surface = vec![h - 3; grid.width];

        for x in 0..grid.width {
            let s = self.surface_height(x as i32, h);
            surface[x] = s;
            for y in s..h - 2 {
                if y == s {
                    let mat = if depth == 1 {
                        MaterialId::Grass
                    } else {
                        MaterialId::Dirt
                    };
                    grid.set_material(x as i32, y, mat);
                } else if y > s + 8 {
                    grid.set_material(x as i32, y, MaterialId::Stone);
                } else {
                    grid.set_material(x as i32, y, MaterialId::Dirt);
                }
            }
            grid.set_material(x as i32, h - 2, MaterialId::Dirt);
            grid.set_material(x as i32, h - 1, MaterialId::Stone);
        }

        self.carve_underground_caves(grid, &surface);
        self.place_surface_features(grid, &surface, depth);

        grid.fill_border(MaterialId::Stone);

        let sx = w / 2;
        let mut surface_y = h - 3;
        for y in 0..h {
            let cell = grid.get(sx, y);
            if cell.is_solid() && cell.material != MaterialId::Stone {
                surface_y = y;
                break;
            }
        }
        grid.set_material(sx, surface_y, MaterialId::Stairs);
        if grid.get(sx, surface_y + 1).is_empty() {
            grid.set_material(sx, surface_y + 1, MaterialId::Stone);
        }

        let spawn_y = surface_y - 5;
        (sx, spawn_y)
    }

    fn generate_caves(&mut self, grid: &mut ChunkedGrid, depth: u32) -> (i32, i32) {
        let w = grid.width as i32;
        let h = grid.height as i32;

        for y in 0..h {
            for x in 0..w {
                grid.set_material(x, y, MaterialId::Stone);
            }
        }

        self.carve_ca_caves(grid, 0.45, 5);
        self.seal_disconnected_regions(grid);

        let pool_count = (2 + depth / 2).min(6) as i32;
        for _ in 0..pool_count {
            let typ = self.random_cave_pool_type(depth);
            self.place_cave_pool(grid, typ, 3, 5);
        }

        let stalactites = self.random_range_i32(3, 7);
        for _ in 0..stalactites {
            let x = self.random_range_i32(8, w - 8);
            let y_top = self.random_range_i32(3, 12);
            let len = self.random_range_i32(3, 9);
            for dy in 0..len {
                let y = y_top + dy;
                if grid.get(x, y).material != MaterialId::Stone {
                    continue;
                }
                grid.set_material(x, y, MaterialId::Stone);
            }
        }

        grid.fill_border(MaterialId::Stone);

        let spawn = self.find_safe_spawn(grid);
        let stairs = self.find_empty_with_floor(grid, w - 15, h - 15);
        if let Some((sx, sy)) = stairs {
            grid.set_material(sx, sy, MaterialId::Stairs);
            if grid.get(sx, sy + 1).is_empty() {
                grid.set_material(sx, sy + 1, MaterialId::Stone);
            }
        }

        spawn
    }

    fn generate_dungeon(
        &mut self,
        grid: &mut ChunkedGrid,
        items: &mut ItemManager,
        _depth: u32,
    ) -> (i32, i32) {
        let w = grid.width as i32;
        let h = grid.height as i32;

        for y in 0..h {
            for x in 0..w {
                grid.set_material(x, y, MaterialId::Stone);
            }
        }

        let root = Rect {
            x: 8,
            y: 8,
            w: w - 16,
            h: h - 16,
        };
        let tree = self.build_bsp(root);
        let mut rooms = Vec::new();
        self.collect_rooms(&tree, &mut rooms);

        if rooms.is_empty() {
            return self.fallback_spawn(grid);
        }

        for room in &rooms {
            self.carve_room(grid, *room);
        }
        self.connect_bsp_rooms(&tree, grid);
        grid.fill_border(MaterialId::Stone);

        let spawn_room = rooms[0];
        let spawn_x = spawn_room.x + spawn_room.w / 2;
        let spawn_y = spawn_room.y + spawn_room.h - 5;
        let spawn = (spawn_x, spawn_y);

        let mut stairs_room = rooms[0];
        let mut best_dist = 0;
        for room in &rooms {
            let cx = room.x + room.w / 2;
            let cy = room.y + room.h / 2;
            let d = (cx - spawn_x).abs() + (cy - spawn_y).abs();
            if d > best_dist {
                best_dist = d;
                stairs_room = *room;
            }
        }
        let sx = stairs_room.x + stairs_room.w / 2;
        let sy = stairs_room.y + stairs_room.h;
        grid.set_material(sx, sy, MaterialId::Stairs);

        let item_count = (3 + self.random_range_i32(0, 3)).min(rooms.len() as i32);
        let mut placed_rooms = rooms.clone();
        self.shuffle_rooms(&mut placed_rooms);
        for i in 0..item_count as usize {
            let room = placed_rooms[i];
            let (ix, iy) = self.random_point_in_room(room);
            if let Some(typ) = self.random_item_type() {
                items.spawn(typ, ix, iy);
            }
        }

        let trap_count = self.random_range_i32(1, 3).min(rooms.len() as i32);
        for _ in 0..trap_count as usize {
            let room = rooms[self.random_usize(rooms.len())];
            let (tx, ty) = self.random_point_in_room(room);
            if grid.get(tx, ty).is_empty() {
                grid.set_material(tx, ty, MaterialId::Acid);
            }
        }

        spawn
    }

    fn place_items(
        &mut self,
        grid: &ChunkedGrid,
        items: &mut ItemManager,
        px: i32,
        py: i32,
        depth: u32,
    ) {
        if depth <= 3 {
            let base_y = py + 1;
            let offsets = [(-6, 1), (6, 1), (-3, -8), (10, 1), (-10, 1), (3, -6)];
            let types = [
                ItemType::Sword,
                ItemType::HealthPotion,
                ItemType::LeatherArmor,
                ItemType::Bow,
                ItemType::Shield,
                ItemType::ManaPotion,
            ];
            for (i, (dx, dy)) in offsets.iter().enumerate() {
                let x = px + dx;
                let y = base_y + dy;
                if grid.in_bounds(x, y) {
                    items.spawn(types[i], x, y);
                }
            }
        }
    }

    fn place_surface_features(&mut self, grid: &mut ChunkedGrid, surface: &[i32], depth: u32) {
        let tree_count = self.random_range_i32(3, 7);
        self.place_trees(grid, surface, tree_count);

        let pool_count = (2 + depth / 2).min(5) as i32;
        for _ in 0..pool_count {
            let typ = self.random_surface_pool_type(depth);
            self.place_surface_pool(grid, typ, 3, 6);
        }

        let dune_count = self.random_range_i32(1, 4);
        self.place_sand_dunes(grid, surface, dune_count);

        let wall_count = self.random_range_i32(1, 4);
        self.place_walls(grid, surface, wall_count);
    }

    fn place_surface_features_chunk(
        &mut self,
        grid: &mut ChunkedGrid,
        cx: i32,
        _cy: i32,
        surface: &[i32],
        depth: u32,
    ) {
        let x0 = cx * CHUNK_SIZE as i32;
        let x1 = x0 + CHUNK_SIZE as i32;
        let tree_count = self.random_range_i32(0, 2);
        self.place_trees_chunk(grid, x0, x1, surface, tree_count);

        let pool_count = (depth / 2).min(2) as i32;
        for _ in 0..pool_count {
            let typ = self.random_surface_pool_type(depth);
            self.place_surface_pool_chunk(grid, x0, x1, surface, typ, 2, 4);
        }

        let dune_count = self.random_range_i32(0, 2);
        self.place_sand_dunes_chunk(grid, x0, x1, surface, dune_count);

        let wall_count = self.random_range_i32(0, 2);
        self.place_walls_chunk(grid, x0, x1, surface, wall_count);
    }

    fn place_trees_chunk(
        &mut self,
        grid: &mut ChunkedGrid,
        x0: i32,
        _x1: i32,
        surface: &[i32],
        count: i32,
    ) {
        for _ in 0..count {
            let lx = self.random_range_i32(2, CHUNK_SIZE as i32 - 2);
            let x = x0 + lx;
            let s = surface[lx as usize];
            if s < 10 {
                continue;
            }
            for y in (s - 6)..s {
                if grid.in_bounds(x, y) {
                    grid.set_material(x, y, MaterialId::Wood);
                }
            }
            for dy in -2..=0 {
                for dx in -2..=2 {
                    if dx * dx + dy * dy <= 5 {
                        let cx = x + dx;
                        let cy = s - 6 + dy;
                        if grid.in_bounds(cx, cy) && grid.get(cx, cy).is_empty() {
                            grid.set_material(cx, cy, MaterialId::Grass);
                        }
                    }
                }
            }
        }
    }

    fn place_surface_pool_chunk(
        &mut self,
        grid: &mut ChunkedGrid,
        x0: i32,
        _x1: i32,
        surface: &[i32],
        typ: MaterialId,
        min_r: i32,
        max_r: i32,
    ) {
        let lx = self.random_range_i32(4, CHUNK_SIZE as i32 - 4);
        let x = x0 + lx;
        let s = surface[lx as usize];
        let r = self.random_range_i32(min_r, max_r);
        let cy = self.random_range_i32(s + 2, s + r + 2);
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    let px = x + dx;
                    let py = cy + dy;
                    if grid.in_bounds(px, py) {
                        grid.set_material(px, py, typ);
                    }
                }
            }
        }
    }

    fn place_cave_pool_chunk(
        &mut self,
        grid: &mut ChunkedGrid,
        cx: i32,
        cy: i32,
        typ: MaterialId,
        min_r: i32,
        max_r: i32,
    ) {
        let x0 = cx * CHUNK_SIZE as i32;
        let y0 = cy * CHUNK_SIZE as i32;
        let x1 = x0 + CHUNK_SIZE as i32;
        let y1 = y0 + CHUNK_SIZE as i32;
        for _ in 0..100 {
            let px = self.random_range_i32(x0 + 4, x1 - 4);
            let py = self.random_range_i32(y0 + 4, y1 - 4);
            if grid.get(px, py).is_empty() {
                continue;
            }
            let mut has_empty_neighbor = false;
            for (dx, dy) in NEIGHBORS4 {
                if grid.get(px + dx, py + dy).is_empty() {
                    has_empty_neighbor = true;
                    break;
                }
            }
            if !has_empty_neighbor {
                continue;
            }
            let r = self.random_range_i32(min_r, max_r);
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx * dx + dy * dy <= r * r {
                        let x = px + dx;
                        let y = py + dy;
                        if grid.in_bounds(x, y) && !grid.get(x, y).is_empty() {
                            grid.set_material(x, y, typ);
                        }
                    }
                }
            }
            return;
        }
    }

    fn place_sand_dunes_chunk(
        &mut self,
        grid: &mut ChunkedGrid,
        x0: i32,
        x1: i32,
        surface: &[i32],
        count: i32,
    ) {
        for _ in 0..count {
            let lx = self.random_range_i32(4, (x1 - x0 - 4).max(5));
            let x = x0 + lx;
            let s = surface[lx as usize];
            let width = self.random_range_i32(4, 10);
            for dx in -width / 2..=width / 2 {
                let pile = (width / 2 - dx.abs()).max(1) + 1;
                for dy in 0..pile {
                    let y = s - 1 - dy;
                    if grid.in_bounds(x + dx, y) && grid.get(x + dx, y).is_empty() {
                        grid.set_material(x + dx, y, MaterialId::Sand);
                    }
                }
            }
        }
    }

    fn place_walls_chunk(
        &mut self,
        grid: &mut ChunkedGrid,
        x0: i32,
        x1: i32,
        surface: &[i32],
        count: i32,
    ) {
        for _ in 0..count {
            let lx = self.random_range_i32(4, (x1 - x0 - 4).max(5));
            let x = x0 + lx;
            let s = surface[lx as usize];
            let h = self.random_range_i32(2, 5);
            for y in (s - h)..s {
                if grid.in_bounds(x, y) {
                    grid.set_material(x, y, MaterialId::Stone);
                }
                if grid.in_bounds(x + 1, y) {
                    grid.set_material(x + 1, y, MaterialId::Stone);
                }
            }
        }
    }

    fn place_trees(&mut self, grid: &mut ChunkedGrid, surface: &[i32], count: i32) {
        let w = grid.width as i32;
        for _ in 0..count {
            let x = self.random_range_i32(10, w - 10);
            let s = surface[x as usize];
            if s < 10 {
                continue;
            }
            for y in (s - 6)..s {
                if grid.in_bounds(x, y) {
                    grid.set_material(x, y, MaterialId::Wood);
                }
            }
            for dy in -2..=0 {
                for dx in -2..=2 {
                    if dx * dx + dy * dy <= 5 {
                        let cx = x + dx;
                        let cy = s - 6 + dy;
                        if grid.in_bounds(cx, cy) && grid.get(cx, cy).is_empty() {
                            grid.set_material(cx, cy, MaterialId::Grass);
                        }
                    }
                }
            }
        }
    }

    fn place_surface_pool(
        &mut self,
        grid: &mut ChunkedGrid,
        typ: MaterialId,
        min_r: i32,
        max_r: i32,
    ) {
        let w = grid.width as i32;
        let cx = self.random_range_i32(15, w - 15);
        let surface = self.find_surface_near(grid, cx);
        let r = self.random_range_i32(min_r, max_r);
        let cy = self.random_range_i32(surface + 2, surface + r + 2);
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    let x = cx + dx;
                    let y = cy + dy;
                    if grid.in_bounds(x, y) {
                        grid.set_material(x, y, typ);
                    }
                }
            }
        }
    }

    fn place_cave_pool(&mut self, grid: &mut ChunkedGrid, typ: MaterialId, min_r: i32, max_r: i32) {
        let w = grid.width as i32;
        let h = grid.height as i32;
        for _ in 0..100 {
            let cx = self.random_range_i32(10, w - 10);
            let cy = self.random_range_i32(10, h - 10);
            if grid.get(cx, cy).is_empty() {
                continue;
            }
            let mut has_empty_neighbor = false;
            for (dx, dy) in NEIGHBORS4 {
                if grid.get(cx + dx, cy + dy).is_empty() {
                    has_empty_neighbor = true;
                    break;
                }
            }
            if !has_empty_neighbor {
                continue;
            }
            let r = self.random_range_i32(min_r, max_r);
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx * dx + dy * dy <= r * r {
                        let x = cx + dx;
                        let y = cy + dy;
                        if grid.in_bounds(x, y) && !grid.get(x, y).is_empty() {
                            grid.set_material(x, y, typ);
                        }
                    }
                }
            }
            return;
        }
    }

    fn place_sand_dunes(&mut self, grid: &mut ChunkedGrid, surface: &[i32], count: i32) {
        let w = grid.width as i32;
        for _ in 0..count {
            let x = self.random_range_i32(15, w - 15);
            let s = surface[x as usize];
            let width = self.random_range_i32(6, 16);
            for dx in -width / 2..=width / 2 {
                let pile = (width / 2 - dx.abs()).max(1) + 1;
                for dy in 0..pile {
                    let y = s - 1 - dy;
                    if grid.in_bounds(x + dx, y) && grid.get(x + dx, y).is_empty() {
                        grid.set_material(x + dx, y, MaterialId::Sand);
                    }
                }
            }
        }
    }

    fn place_walls(&mut self, grid: &mut ChunkedGrid, surface: &[i32], count: i32) {
        let w = grid.width as i32;
        for _ in 0..count {
            let x = self.random_range_i32(10, w - 10);
            let s = surface[x as usize];
            let h = self.random_range_i32(3, 7);
            for y in (s - h)..s {
                if grid.in_bounds(x, y) {
                    grid.set_material(x, y, MaterialId::Stone);
                }
                if grid.in_bounds(x + 1, y) {
                    grid.set_material(x + 1, y, MaterialId::Stone);
                }
            }
        }
    }

    fn carve_underground_caves(&mut self, grid: &mut ChunkedGrid, surface: &[i32]) {
        let w = grid.width as i32;
        let h = grid.height as i32;
        for _ in 0..8 {
            let cx = self.random_range_i32(10, w - 10);
            let lower = (h * 2 / 3).max(surface[cx as usize] + 12);
            let cy = self.random_range_i32(lower, h - 10);
            let r = self.random_range_i32(3, 6);
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx * dx + dy * dy <= r * r {
                        let x = cx + dx;
                        let y = cy + dy;
                        if grid.in_bounds(x, y) && y > surface[x as usize] + 4 {
                            grid.set(x, y, Cell::empty());
                        }
                    }
                }
            }
        }
    }

    fn carve_ca_caves(&mut self, grid: &mut ChunkedGrid, fill_prob: f64, iterations: i32) {
        let w = grid.width as i32;
        let h = grid.height as i32;
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                if self.ca.random_u32() as f64 / (u32::MAX as f64) < fill_prob {
                    grid.set(x, y, Cell::empty());
                }
            }
        }

        let size = (w * h) as usize;
        let mut buf = Vec::with_capacity(size);
        for y in 0..h {
            for x in 0..w {
                buf.push(grid.get(x, y));
            }
        }
        for _ in 0..iterations {
            for y in 1..h - 1 {
                for x in 1..w - 1 {
                    let walls = self.wall_count(grid, x, y);
                    let i = (y * w + x) as usize;
                    if walls > 4 {
                        buf[i] = Cell::new(MaterialId::Stone);
                    } else if walls < 4 {
                        buf[i] = Cell::empty();
                    }
                }
            }
            for y in 0..h {
                for x in 0..w {
                    let i = (y * w + x) as usize;
                    grid.set(x, y, buf[i]);
                }
            }
        }
    }

    fn carve_ca_caves_chunk(
        &mut self,
        grid: &mut ChunkedGrid,
        cx: i32,
        cy: i32,
        fill_prob: f64,
        iterations: i32,
    ) {
        let x0 = cx * CHUNK_SIZE as i32;
        let y0 = cy * CHUNK_SIZE as i32;
        let x1 = x0 + CHUNK_SIZE as i32;
        let y1 = y0 + CHUNK_SIZE as i32;
        for y in y0 + 1..y1 - 1 {
            for x in x0 + 1..x1 - 1 {
                if self.ca.random_u32() as f64 / (u32::MAX as f64) < fill_prob {
                    grid.set(x, y, Cell::empty());
                }
            }
        }

        let size = CHUNK_SIZE * CHUNK_SIZE;
        let mut buf = Vec::with_capacity(size);
        for y in y0..y1 {
            for x in x0..x1 {
                buf.push(grid.get(x, y));
            }
        }
        let w = CHUNK_SIZE as i32;
        for _ in 0..iterations {
            for y in y0 + 1..y1 - 1 {
                for x in x0 + 1..x1 - 1 {
                    let walls = self.wall_count(grid, x, y);
                    let ly = y - y0;
                    let lx = x - x0;
                    let i = (ly * w + lx) as usize;
                    if walls > 4 {
                        buf[i] = Cell::new(MaterialId::Stone);
                    } else if walls < 4 {
                        buf[i] = Cell::empty();
                    }
                }
            }
            for y in y0..y1 {
                for x in x0..x1 {
                    let ly = y - y0;
                    let lx = x - x0;
                    let i = (ly * w + lx) as usize;
                    grid.set(x, y, buf[i]);
                }
            }
        }
    }

    fn wall_count(&self, grid: &ChunkedGrid, x: i32, y: i32) -> i32 {
        let mut count = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if !grid.in_bounds(nx, ny) {
                    count += 1;
                } else {
                    let cell = grid.get(nx, ny);
                    if cell.is_solid() || cell.material == MaterialId::Stone {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn seal_disconnected_regions(&mut self, grid: &mut ChunkedGrid) {
        let w = grid.width as i32;
        let h = grid.height as i32;
        let size = (w * h) as usize;
        let mut visited = vec![false; size];
        let mut best = Vec::new();

        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) as usize;
                if grid.get(x, y).is_empty() && !visited[idx] {
                    let mut comp = Vec::new();
                    self.flood_fill(grid, &mut visited, x, y, &mut comp);
                    if comp.len() > best.len() {
                        best = comp;
                    }
                }
            }
        }

        let mut in_best = vec![false; size];
        for (x, y) in &best {
            in_best[(*y * w + *x) as usize] = true;
        }

        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) as usize;
                if grid.get(x, y).is_empty() && !in_best[idx] {
                    grid.set_material(x, y, MaterialId::Stone);
                }
            }
        }
    }

    fn flood_fill(
        &self,
        grid: &ChunkedGrid,
        visited: &mut [bool],
        x: i32,
        y: i32,
        comp: &mut Vec<(i32, i32)>,
    ) {
        let w = grid.width as i32;
        let mut stack = vec![(x, y)];
        while let Some((cx, cy)) = stack.pop() {
            let idx = (cy * w + cx) as usize;
            if visited[idx] || !grid.in_bounds(cx, cy) || !grid.get(cx, cy).is_empty() {
                continue;
            }
            visited[idx] = true;
            comp.push((cx, cy));
            for &(dx, dy) in &NEIGHBORS4 {
                stack.push((cx + dx, cy + dy));
            }
        }
    }

    fn build_bsp(&mut self, rect: Rect) -> BspNode {
        let mut node = BspNode {
            left: None,
            right: None,
            room: None,
        };
        if let Some((left, right)) = self.split_rect(rect) {
            node.left = Some(Box::new(self.build_bsp(left)));
            node.right = Some(Box::new(self.build_bsp(right)));
        } else {
            node.room = Some(self.carve_room_rect(rect));
        }
        node
    }

    fn split_rect(&mut self, rect: Rect) -> Option<(Rect, Rect)> {
        let min_size = 22;
        if rect.w < min_size * 2 || rect.h < min_size * 2 {
            return None;
        }
        if rect.w > rect.h {
            let split = self.random_range_i32(min_size, rect.w - min_size);
            let left = Rect {
                x: rect.x,
                y: rect.y,
                w: split,
                h: rect.h,
            };
            let right = Rect {
                x: rect.x + split,
                y: rect.y,
                w: rect.w - split,
                h: rect.h,
            };
            Some((left, right))
        } else {
            let split = self.random_range_i32(min_size, rect.h - min_size);
            let top = Rect {
                x: rect.x,
                y: rect.y,
                w: rect.w,
                h: split,
            };
            let bottom = Rect {
                x: rect.x,
                y: rect.y + split,
                w: rect.w,
                h: rect.h - split,
            };
            Some((top, bottom))
        }
    }

    fn carve_room_rect(&mut self, rect: Rect) -> Rect {
        let min_w = 9;
        let min_h = 9;
        let max_pad_x = ((rect.w - min_w) / 2).max(1);
        let max_pad_y = ((rect.h - min_h) / 2).max(1);
        let pad_x = self.random_range_i32(1, max_pad_x + 1);
        let pad_y = self.random_range_i32(1, max_pad_y + 1);
        Rect {
            x: rect.x + pad_x,
            y: rect.y + pad_y,
            w: rect.w - pad_x * 2,
            h: rect.h - pad_y * 2,
        }
    }

    fn carve_room(&mut self, grid: &mut ChunkedGrid, room: Rect) {
        for y in room.y..room.y + room.h {
            for x in room.x..room.x + room.w {
                grid.set(x, y, Cell::empty());
            }
        }
    }

    fn collect_rooms(&self, node: &BspNode, rooms: &mut Vec<Rect>) {
        if let Some(room) = node.room {
            rooms.push(room);
        }
        if let Some(ref left) = node.left {
            self.collect_rooms(left, rooms);
        }
        if let Some(ref right) = node.right {
            self.collect_rooms(right, rooms);
        }
    }

    fn connect_bsp_rooms(&mut self, node: &BspNode, grid: &mut ChunkedGrid) {
        if let (Some(left), Some(right)) = (&node.left, &node.right) {
            let r1 = self.find_first_room(left);
            let r2 = self.find_first_room(right);
            if let (Some(a), Some(b)) = (r1, r2) {
                self.carve_l_corridor(grid, &a, &b);
            }
            self.connect_bsp_rooms(left, grid);
            self.connect_bsp_rooms(right, grid);
        }
    }

    fn find_first_room(&self, node: &BspNode) -> Option<Rect> {
        if let Some(room) = node.room {
            return Some(room);
        }
        if let Some(ref left) = node.left {
            if let Some(room) = self.find_first_room(left) {
                return Some(room);
            }
        }
        if let Some(ref right) = node.right {
            if let Some(room) = self.find_first_room(right) {
                return Some(room);
            }
        }
        None
    }

    fn carve_l_corridor(&mut self, grid: &mut ChunkedGrid, a: &Rect, b: &Rect) {
        let c1 = (a.x + a.w / 2, a.y + a.h / 2);
        let c2 = (b.x + b.w / 2, b.y + b.h / 2);
        let x0 = c1.0.min(c2.0);
        let x1 = c1.0.max(c2.0);
        for x in x0..=x1 {
            for dy in 0..2 {
                if grid.in_bounds(x, c1.1 + dy) {
                    grid.set(x, c1.1 + dy, Cell::empty());
                }
            }
        }
        let y0 = c1.1.min(c2.1);
        let y1 = c1.1.max(c2.1);
        for y in y0..=y1 {
            for dx in 0..2 {
                if grid.in_bounds(c2.0 + dx, y) {
                    grid.set(c2.0 + dx, y, Cell::empty());
                }
            }
        }
    }

    fn find_safe_spawn(&mut self, grid: &ChunkedGrid) -> (i32, i32) {
        let w = grid.width as i32;
        let h = grid.height as i32;
        for floor_y in 10..h - 10 {
            for x in 10..w - 10 {
                if !grid.get(x, floor_y).is_solid() {
                    continue;
                }
                let cy = floor_y - 3;
                if !self.vertical_clear(grid, x, cy) {
                    continue;
                }
                if !self.horizontal_clear(grid, x, cy) {
                    continue;
                }
                return (x, cy);
            }
        }
        (w / 2, h / 2)
    }

    fn find_empty_with_floor(
        &mut self,
        grid: &ChunkedGrid,
        target_x: i32,
        target_y: i32,
    ) -> Option<(i32, i32)> {
        let w = grid.width as i32;
        let h = grid.height as i32;
        let mut best = None;
        let mut best_dist = i32::MAX;
        for y in 5..h - 5 {
            for x in 5..w - 5 {
                if grid.get(x, y).is_empty()
                    && grid.in_bounds(x, y + 1)
                    && grid.get(x, y + 1).is_solid()
                {
                    let d = (x - target_x).abs() + (y - target_y).abs();
                    if d < best_dist {
                        best_dist = d;
                        best = Some((x, y));
                    }
                }
            }
        }
        best
    }

    fn vertical_clear(&self, grid: &ChunkedGrid, x: i32, y: i32) -> bool {
        for dy in -3..=2 {
            if !grid.in_bounds(x, y + dy) || !grid.get(x, y + dy).is_empty() {
                return false;
            }
        }
        grid.in_bounds(x, y + 3) && grid.get(x, y + 3).is_solid()
    }

    fn horizontal_clear(&self, grid: &ChunkedGrid, x: i32, y: i32) -> bool {
        for dx in -3..=3 {
            if !grid.in_bounds(x + dx, y) || !grid.get(x + dx, y).is_empty() {
                return false;
            }
        }
        true
    }

    fn fallback_spawn(&mut self, grid: &mut ChunkedGrid) -> (i32, i32) {
        let w = grid.width as i32;
        let h = grid.height as i32;
        for y in (10..h - 10).rev() {
            for x in 10..w - 10 {
                if grid.get(x, y).is_empty() {
                    grid.set(x, y, Cell::empty());
                    return (x, y);
                }
            }
        }
        (w / 2, h / 2)
    }

    fn surface_height(&mut self, x: i32, h: i32) -> i32 {
        let base = (h - 3) - ((x as f32 * 0.08).sin() * 4.0) as i32;
        let detail = ((x as f32 * 0.23).sin() * 2.0) as i32;
        let micro = ((x as f32 * 0.57).sin() * 1.0) as i32;
        (base + detail + micro).max(10).min(h - 3)
    }

    const SURFACE_BASE_Y: i32 = 120;

    fn surface_height_world(&mut self, x: i32) -> i32 {
        let base = Self::SURFACE_BASE_Y - ((x as f32 * 0.08).sin() * 4.0) as i32;
        let detail = ((x as f32 * 0.23).sin() * 2.0) as i32;
        let micro = ((x as f32 * 0.57).sin() * 1.0) as i32;
        (base + detail + micro).max(10)
    }

    fn find_surface_near(&mut self, grid: &ChunkedGrid, x: i32) -> i32 {
        let h = grid.height as i32;
        for y in 0..h {
            if grid.get(x, y).is_solid() && grid.get(x, y).material != MaterialId::Stone {
                return y;
            }
        }
        h - 3
    }

    fn random_surface_pool_type(&mut self, depth: u32) -> MaterialId {
        let r = self.ca.random_u32() % 100;
        if depth == 1 {
            match r {
                0..=50 => MaterialId::Water,
                51..=80 => MaterialId::Sand,
                _ => MaterialId::Acid,
            }
        } else {
            match r {
                0..=40 => MaterialId::Water,
                41..=60 => MaterialId::Sand,
                61..=80 => MaterialId::Acid,
                _ => MaterialId::Lava,
            }
        }
    }

    fn random_cave_pool_type(&mut self, depth: u32) -> MaterialId {
        let r = self.ca.random_u32() % 100;
        match depth {
            4 => match r {
                0..=50 => MaterialId::Water,
                51..=75 => MaterialId::Lava,
                _ => MaterialId::Acid,
            },
            _ => match r {
                0..=30 => MaterialId::Water,
                31..=60 => MaterialId::Lava,
                _ => MaterialId::Acid,
            },
        }
    }

    fn random_item_type(&mut self) -> Option<ItemType> {
        let types = [
            ItemType::Dagger,
            ItemType::Sword,
            ItemType::Bow,
            ItemType::LeatherArmor,
            ItemType::PlateArmor,
            ItemType::Shield,
            ItemType::HealthPotion,
            ItemType::ManaPotion,
            ItemType::Food,
            ItemType::Scroll,
        ];
        let idx = self.random_usize(types.len());
        Some(types[idx])
    }

    fn random_point_in_room(&mut self, room: Rect) -> (i32, i32) {
        let x = room.x + self.random_range_i32(1, room.w - 1);
        let y = room.y + self.random_range_i32(1, room.h - 1);
        (x, y)
    }

    fn shuffle_rooms(&mut self, rooms: &mut Vec<Rect>) {
        for i in (1..rooms.len()).rev() {
            let j = self.random_usize(i + 1);
            rooms.swap(i, j);
        }
    }

    fn random_range_i32(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        min + (self.ca.random_u32() % (max - min) as u32) as i32
    }

    fn random_usize(&mut self, max: usize) -> usize {
        if max == 0 {
            return 0;
        }
        self.ca.random_usize(max)
    }
}

struct BspNode {
    left: Option<Box<BspNode>>,
    right: Option<Box<BspNode>>,
    room: Option<Rect>,
}

const NEIGHBORS4: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
