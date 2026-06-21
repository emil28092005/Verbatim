use std::collections::VecDeque;

use verbatim::ai::GameSession;

fn init_at_depth(depth: u32) -> GameSession {
    let mut s = GameSession::new_seeded(123);
    s.game.depth = depth;
    s.init();
    s
}

fn count_distinct_materials(s: &GameSession) -> usize {
    let mut present = Vec::new();
    for y in 0..s.game.grid.height as i32 {
        for x in 0..s.game.grid.width as i32 {
            let mat = s.game.grid.get(x, y).material;
            if !present.contains(&mat) {
                present.push(mat);
            }
        }
    }
    present.len()
}

fn total_empty(s: &GameSession) -> usize {
    let mut count = 0;
    for y in 0..s.game.grid.height as i32 {
        for x in 0..s.game.grid.width as i32 {
            if s.game.grid.get(x, y).is_empty() {
                count += 1;
            }
        }
    }
    count
}

fn player_not_in_solid(s: &GameSession) -> bool {
    let (px, py) = s.game.player.center(&s.game.entities);
    let ix = px as i32;
    let iy = py as i32;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if s.game.grid.get(ix + dx, iy + dy).is_solid() {
                return false;
            }
        }
    }
    true
}

fn flood_fill_count(s: &GameSession, x: i32, y: i32) -> usize {
    let w = s.game.grid.width as i32;
    let h = s.game.grid.height as i32;
    let mut visited = vec![false; (w * h) as usize];
    let mut q = VecDeque::new();
    q.push_back((x, y));
    let mut count = 0;
    while let Some((cx, cy)) = q.pop_front() {
        let idx = (cy * w + cx) as usize;
        if visited[idx] || !s.game.grid.in_bounds(cx, cy) || !s.game.grid.get(cx, cy).is_empty() {
            continue;
        }
        visited[idx] = true;
        count += 1;
        for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
            q.push_back((cx + dx, cy + dy));
        }
    }
    count
}

fn count_empty_regions(s: &GameSession) -> usize {
    let w = s.game.grid.width as i32;
    let h = s.game.grid.height as i32;
    let mut visited = vec![false; (w * h) as usize];
    let mut regions = 0;
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if !visited[idx] && s.game.grid.get(x, y).is_empty() {
                regions += 1;
                let mut stack = vec![(x, y)];
                while let Some((cx, cy)) = stack.pop() {
                    let i = (cy * w + cx) as usize;
                    if visited[i]
                        || !s.game.grid.in_bounds(cx, cy)
                        || !s.game.grid.get(cx, cy).is_empty()
                    {
                        continue;
                    }
                    visited[i] = true;
                    for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                        stack.push((cx + dx, cy + dy));
                    }
                }
            }
        }
    }
    regions
}

#[test]
fn surface_generation_has_stairs_and_open_spawn() {
    let s = init_at_depth(1);
    assert!(
        s.find_material("stairs").is_some(),
        "surface should have stairs"
    );
    assert!(
        player_not_in_solid(&s),
        "player should spawn in an open cell"
    );
    assert!(
        count_distinct_materials(&s) >= 3,
        "surface should have several materials"
    );
}

#[test]
fn cave_generation_has_stairs_and_connected_empty() {
    let s = init_at_depth(4);
    assert!(
        s.find_material("stairs").is_some(),
        "cave should have stairs"
    );
    assert!(
        player_not_in_solid(&s),
        "player should spawn in an open cell"
    );

    let empty = total_empty(&s);
    assert!(
        empty > 100,
        "cave should have a meaningful empty region: {}",
        empty
    );

    let (px, py) = s.game.player.center(&s.game.entities);
    let connected = flood_fill_count(&s, px as i32, py as i32);
    assert!(
        connected >= empty * 95 / 100,
        "cave should be mostly connected: {} of {}",
        connected,
        empty
    );

    assert_eq!(
        count_empty_regions(&s),
        1,
        "cave should be a single connected empty region"
    );
}

#[test]
fn dungeon_generation_has_stairs_and_rooms() {
    let s = init_at_depth(7);
    assert!(
        s.find_material("stairs").is_some(),
        "dungeon should have stairs"
    );
    assert!(
        player_not_in_solid(&s),
        "player should spawn in an open cell"
    );
    assert!(
        count_distinct_materials(&s) >= 3,
        "dungeon should have several materials"
    );

    let empty = total_empty(&s);
    assert!(
        empty > 500,
        "dungeon should have many room cells: {}",
        empty
    );

    let regions = count_empty_regions(&s);
    assert!(
        regions >= 1 && regions <= 4,
        "dungeon rooms should be connected or nearly connected: {} regions",
        regions
    );
}

#[test]
fn different_depths_produce_different_structures() {
    let s1 = init_at_depth(1);
    let s2 = init_at_depth(4);
    let s3 = init_at_depth(7);

    let empty1 = total_empty(&s1);
    let empty2 = total_empty(&s2);
    let empty3 = total_empty(&s3);

    assert!(
        empty1 != empty2 || empty2 != empty3,
        "depths should differ in empty space: {} {} {}",
        empty1,
        empty2,
        empty3
    );

    let grass1 = s1.count_material_in_region(
        0,
        0,
        s1.game.grid.width as i32,
        s1.game.grid.height as i32,
        "grass",
    );
    let grass2 = s2.count_material_in_region(
        0,
        0,
        s2.game.grid.width as i32,
        s2.game.grid.height as i32,
        "grass",
    );
    let grass3 = s3.count_material_in_region(
        0,
        0,
        s3.game.grid.width as i32,
        s3.game.grid.height as i32,
        "grass",
    );
    assert!(
        grass1 > grass2 && grass2 == grass3,
        "grass should dominate surface and vanish in deeper levels: {} {} {}",
        grass1,
        grass2,
        grass3
    );
}

#[test]
fn dungeon_has_large_empty_rooms() {
    let s = init_at_depth(8);
    let mut found_room = false;
    for y in 5..s.game.grid.height as i32 - 5 {
        for x in 5..s.game.grid.width as i32 - 5 {
            let mut w = 0;
            while x + w < s.game.grid.width as i32 - 5 && s.game.grid.get(x + w, y).is_empty() {
                w += 1;
            }
            let mut h = 0;
            while y + h < s.game.grid.height as i32 - 5 && s.game.grid.get(x, y + h).is_empty() {
                h += 1;
            }
            if w >= 5 && h >= 5 {
                found_room = true;
            }
        }
    }
    assert!(found_room, "dungeon should contain rooms at least 5x5");
}

#[test]
fn world_generation_respects_seeds() {
    let s1 = init_at_depth(3);
    let s2 = init_at_depth(3);
    let (p1, _) = s1.game.player.center(&s1.game.entities);
    let (p2, _) = s2.game.player.center(&s2.game.entities);
    assert!(
        (p1 - p2).abs() < 0.01,
        "same seed should place player at the same x"
    );
    let m1 = s1.find_material("stairs");
    let m2 = s2.find_material("stairs");
    assert_eq!(m1, m2, "same seed should place stairs at the same location");
}
