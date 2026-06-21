use verbatim::ai::action::AiAction;
use verbatim::ai::session::GameSession;
use verbatim::world::cell::MaterialId;
use verbatim::world::chunk::CHUNK_SIZE;
use verbatim::world::chunked_grid::ChunkedGrid;

fn setup() -> GameSession {
    let mut s = GameSession::new();
    s.init_empty();
    s
}

#[test]
fn fire_produces_co2_gas() {
    let mut s = setup();
    s.perform_action(&AiAction::SetCell {
        x: 100,
        y: 100,
        material: "wood".into(),
    });
    s.perform_action(&AiAction::SetCell {
        x: 99,
        y: 100,
        material: "fire".into(),
    });
    s.step(2);
    let mut found_co2 = false;
    for dy in -5..=0 {
        let (gt, gd) = s.game.grid.get_gas(99, 100 + dy);
        if gt == 3 && gd > 0 {
            found_co2 = true;
            break;
        }
    }
    assert!(found_co2, "fire should produce CO2 (type 3) near fire");
}

#[test]
fn heat_transfer_diffuses_through_solid() {
    let mut grid = ChunkedGrid::with_size(250, 250);
    grid.set_material(100, 100, MaterialId::Stone);
    grid.set_material(101, 100, MaterialId::Stone);
    grid.set_temp(100, 100, 500.0);
    grid.mark_dirty(100, 100);
    let mut ca = verbatim::world::cellular::CellularAutomaton::new();
    for _ in 0..100 {
        ca.step(&mut grid);
    }
    let neighbor_temp = grid.get_temp(101, 100);
    assert!(
        neighbor_temp > 25.0,
        "heat should diffuse to adjacent stone, got {:.1}",
        neighbor_temp
    );
}

#[test]
fn lava_heats_adjacent_water_to_steam() {
    let mut s = setup();
    s.perform_action(&AiAction::SetCell {
        x: 100,
        y: 100,
        material: "lava".into(),
    });
    s.perform_action(&AiAction::SetCell {
        x: 101,
        y: 100,
        material: "water".into(),
    });
    s.step(5);
    let cell = s.get_cell(101, 100);
    assert!(
        cell.material == "steam" || cell.material == "stone" || cell.material == "empty",
        "water should become steam, lava stone, or steam rises away, got '{}'",
        cell.material
    );
}

#[test]
fn gas_layer_exists_and_defaults_to_air() {
    let s = setup();
    let (gt, gd) = s.game.grid.get_gas(100, 100);
    assert_eq!(gt, 0, "default gas type should be air (0)");
    assert_eq!(gd, 0, "default gas density should be 0");
}

#[test]
fn gas_rises_upward() {
    let mut s = setup();
    s.game.grid.set_gas(100, 105, 1, 200);
    s.step(3);
    let mut found_above = false;
    for y in 100..=104 {
        let (_, gd) = s.game.grid.get_gas(100, y);
        if gd > 0 {
            found_above = true;
            break;
        }
    }
    assert!(found_above, "smoke gas should rise upward");
}

#[test]
fn pressure_layer_defaults_to_atmospheric() {
    let s = setup();
    let p = s.game.grid.get_pressure(100, 100);
    assert_eq!(p, 128, "default pressure should be 128 (atmospheric)");
}

#[test]
fn pressure_equalizes_between_neighbors() {
    let mut s = setup();
    s.game.grid.set_pressure(100, 100, 200);
    s.game.grid.set_pressure(101, 100, 128);
    s.step(30);
    let p1 = s.game.grid.get_pressure(100, 100);
    let p2 = s.game.grid.get_pressure(101, 100);
    let diff = (p1 as i32 - p2 as i32).abs();
    assert!(
        diff < 30,
        "pressure should equalize, got p1={} p2={} diff={}",
        p1,
        p2,
        diff
    );
}

#[test]
fn light_layer_defaults_to_dark() {
    let s = setup();
    let l = s.game.grid.get_light(100, 100);
    assert_eq!(l, [0, 0, 0], "default light should be dark");
}

#[test]
fn lava_emits_world_space_light() {
    let mut s = setup();
    s.perform_action(&AiAction::SetCell {
        x: 100,
        y: 100,
        material: "lava".into(),
    });
    s.step(15);
    let l = s.game.grid.get_light(100, 100);
    assert!(
        l[0] > 0 || l[1] > 0 || l[2] > 0,
        "lava should emit world-space light, got {:?}",
        l
    );
}

#[test]
fn light_blocked_by_solid_walls() {
    let mut s = setup();
    s.perform_action(&AiAction::SetCell {
        x: 100,
        y: 100,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::SetCell {
        x: 100,
        y: 101,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::SetCell {
        x: 100,
        y: 102,
        material: "lava".into(),
    });
    for x in 101..=104 {
        for y in 96..=108 {
            s.perform_action(&AiAction::SetCell {
                x,
                y,
                material: "stone".into(),
            });
        }
    }
    s.step(15);
    let l_behind = s.game.grid.get_light(105, 102);
    assert!(
        l_behind[0] < 30,
        "light should be blocked by thick stone wall, got {:?}",
        l_behind
    );
}

#[test]
fn chunk_save_load_roundtrip_preserves_all_layers() {
    let mut grid = ChunkedGrid::with_size(250, 250);
    grid.set_material(70, 70, MaterialId::Lava);
    grid.set_temp(70, 70, 1500.0);
    grid.set_gas(70, 70, 3, 100);
    grid.set_pressure(70, 70, 200);
    grid.set_light(70, 70, [255, 128, 64]);

    let path = "/tmp/verbatim_multilayer_chunk_1_1.bin";
    let _ = std::fs::remove_file(path);
    grid.save_chunk(path, 1, 1).unwrap();

    let mut grid2 = ChunkedGrid::with_size(250, 250);
    grid2.load_chunk(path, 1, 1).unwrap();

    assert_eq!(grid2.get(70, 70).material, MaterialId::Lava);
    assert!(
        (grid2.get_temp(70, 70) - 1500.0).abs() < 1.0,
        "temp should roundtrip"
    );
    assert_eq!(grid2.get_gas(70, 70), (3, 100), "gas should roundtrip");
    assert_eq!(grid2.get_pressure(70, 70), 200, "pressure should roundtrip");
    assert_eq!(
        grid2.get_light(70, 70),
        [255, 128, 64],
        "light should roundtrip"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn steam_gas_condenses_to_water_when_cold() {
    let mut s = setup();
    s.game.grid.set_gas(100, 100, 4, 200);
    s.game.grid.set_temp(100, 100, 50.0);
    s.game.grid.mark_dirty(100, 100);
    s.step(1);
    let cell = s.get_cell(100, 100);
    assert_eq!(
        cell.material, "water",
        "steam gas at 50C should condense to water, got '{}'",
        cell.material
    );
}

#[test]
fn poison_gas_damages_entity() {
    let mut s = setup();
    let (px, py) = s.game.player.center(&s.game.entities);
    let health_before = s
        .game
        .player
        .entity(&s.game.entities)
        .map(|e| e.health)
        .unwrap_or(0.0);
    for _ in 0..10 {
        s.game.grid.set_gas(px as i32, py as i32, 2, 200);
        s.step(1);
    }
    let health_after = s
        .game
        .player
        .entity(&s.game.entities)
        .map(|e| e.health)
        .unwrap_or(0.0);
    assert!(
        health_after < health_before,
        "poison gas should damage entity: before={} after={}",
        health_before,
        health_after
    );
}

#[test]
fn temperature_persists_across_chunk_boundary() {
    let mut grid = ChunkedGrid::with_size(256, 256);
    let boundary = CHUNK_SIZE as i32;
    grid.set_material(boundary - 1, 0, MaterialId::Stone);
    grid.set_material(boundary, 0, MaterialId::Stone);
    grid.set_temp(boundary - 1, 0, 500.0);
    grid.set_temp(boundary, 0, 20.0);
    let mut ca = verbatim::world::cellular::CellularAutomaton::new();
    for _ in 0..100 {
        ca.step(&mut grid);
    }
    let t_right = grid.get_temp(boundary, 0);
    assert!(
        t_right > 30.0,
        "heat should cross chunk boundary, got {:.1}",
        t_right
    );
}
