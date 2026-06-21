use verbatim::ai::AiAction;
use verbatim::ai::GameSession;

fn setup() -> GameSession {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(90, 90, 50, 50);
    s.perform_action(&AiAction::FillRect {
        x: 80,
        y: 130,
        w: 80,
        h: 15,
        material: "stone".into(),
    });
    s
}

#[test]
fn entity_at_world_boundary_stays_in_bounds() {
    let mut s = setup();
    s.perform_action(&AiAction::Spawn {
        kind: "goblin".into(),
        x: 5.0,
        y: 120.0,
    });
    s.step(60);
    let entities = s.get_entities();
    if let Some(g) = entities.into_iter().find(|e| e.kind == "Goblin" && e.alive) {
        assert!(
            g.pos[0] >= 0.0 && g.pos[0] <= 250.0,
            "goblin should stay in bounds: x={}",
            g.pos[0]
        );
        assert!(
            g.pos[1] >= 0.0 && g.pos[1] <= 250.0,
            "goblin should stay in bounds: y={}",
            g.pos[1]
        );
    }
}

#[test]
fn entity_at_right_boundary() {
    let mut s = setup();
    s.perform_action(&AiAction::Spawn {
        kind: "goblin".into(),
        x: 245.0,
        y: 120.0,
    });
    s.step(60);
    let entities = s.get_entities();
    if let Some(g) = entities.into_iter().find(|e| e.kind == "Goblin" && e.alive) {
        assert!(
            g.pos[0] <= 248.0,
            "goblin should not exit right boundary: x={}",
            g.pos[0]
        );
    }
}

#[test]
fn many_entities_simulate_without_crash() {
    let mut s = setup();
    for i in 0..10 {
        let x = 100.0 + (i as f32 * 5.0);
        s.perform_action(&AiAction::Spawn {
            kind: "goblin".into(),
            x,
            y: 110.0,
        });
    }
    s.step(60);
    let alive = s.get_entities().into_iter().filter(|e| e.alive).count();
    assert!(
        alive > 0,
        "at least some entities should survive 60 ticks with 10 goblins"
    );
}

#[test]
fn spawn_many_goblins_stress() {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(0, 0, 250, 250);
    s.perform_action(&AiAction::FillRect {
        x: 0,
        y: 200,
        w: 250,
        h: 50,
        material: "stone".into(),
    });
    for i in 0..20 {
        let x = 20.0 + (i as f32 * 10.0);
        s.perform_action(&AiAction::Spawn {
            kind: "goblin".into(),
            x,
            y: 180.0,
        });
    }
    s.step(30);
    let entities = s.get_entities();
    let alive = entities.iter().filter(|e| e.alive).count();
    assert!(alive >= 15, "most goblins should survive: {}/20", alive);
}

#[test]
fn player_at_spawn_is_alive() {
    let mut s = GameSession::new_seeded(42);
    s.init();
    let p = s.get_player().unwrap();
    assert!(p.alive, "player should be alive at spawn");
    assert_eq!(p.health, 100.0, "player should have full health at spawn");
}

#[test]
fn player_bodies_count_matches_layout() {
    let mut s = GameSession::new_seeded(42);
    s.init();
    let p = s.get_player().unwrap();
    assert_eq!(
        p.body_count, 23,
        "player should have 23 bodies (humanoid shape)"
    );
}

#[test]
fn goblin_has_correct_max_health() {
    let mut s = setup();
    s.perform_action(&AiAction::Spawn {
        kind: "goblin".into(),
        x: 130.0,
        y: 120.0,
    });
    let entities = s.get_entities();
    let g = entities.into_iter().find(|e| e.kind == "Goblin").unwrap();
    assert_eq!(g.max_health, 40.0, "goblin max health should be 40");
}

#[test]
fn player_has_correct_max_health() {
    let mut s = GameSession::new_seeded(42);
    s.init();
    let p = s.get_player().unwrap();
    assert_eq!(p.max_health, 100.0, "player max health should be 100");
}

#[test]
fn fill_rect_creates_correct_material_count() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect {
        x: 100,
        y: 100,
        w: 5,
        h: 3,
        material: "wood".into(),
    });
    let count = s.count_material_in_region(100, 100, 5, 3, "wood");
    assert_eq!(
        count, 15,
        "5x3 fill_rect should create 15 wood cells, got {}",
        count
    );
}

#[test]
fn clear_region_removes_all_materials() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect {
        x: 100,
        y: 100,
        w: 5,
        h: 5,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::ClearRegion {
        x: 100,
        y: 100,
        w: 5,
        h: 5,
    });
    let count = s.count_material_in_region(100, 100, 5, 5, "stone");
    assert_eq!(count, 0, "clear_region should remove all materials");
}

#[test]
fn set_cell_overwrites_existing() {
    let mut s = setup();
    s.perform_action(&AiAction::SetCell {
        x: 105,
        y: 110,
        material: "stone".into(),
    });
    assert_eq!(s.get_cell(105, 110).material, "stone");
    s.perform_action(&AiAction::SetCell {
        x: 105,
        y: 110,
        material: "water".into(),
    });
    assert_eq!(
        s.get_cell(105, 110).material,
        "water",
        "set_cell should overwrite"
    );
}

#[test]
fn out_of_bounds_cell_returns_error() {
    let s = setup();
    let cell = s.get_cell(-1, -1);
    assert_eq!(
        cell.material, "out_of_bounds",
        "out of bounds cell should report correctly"
    );
    let cell2 = s.get_cell(999, 999);
    assert_eq!(cell2.material, "out_of_bounds");
}

#[test]
fn paint_creates_material_in_radius() {
    let mut s = setup();
    s.perform_action(&AiAction::Paint {
        x: 120,
        y: 110,
        material: "sand".into(),
        radius: 3,
    });
    let count = s.count_material_in_region(116, 106, 8, 8, "sand");
    assert!(count > 0, "paint should create sand cells in radius");
    assert!(
        count < 50,
        "paint should not create too many cells: {}",
        count
    );
}

#[test]
fn set_gravity_affects_player() {
    let mut s = setup();
    s.step(30);
    s.perform_action(&AiAction::SetGravity { value: 0.0 });
    s.perform_action(&AiAction::Jump);
    s.step(5);
    let p1 = s.get_player().unwrap();
    let y_up = p1.pos[1];
    s.step(30);
    let p2 = s.get_player().unwrap();
    assert!(
        p2.pos[1] <= y_up + 2.0,
        "with zero gravity, player should not fall: y_up={} y_after={}",
        y_up,
        p2.pos[1]
    );
}

#[test]
fn set_camera_works() {
    let mut s = setup();
    s.perform_action(&AiAction::SetCamera { x: 50, y: 50 });
    assert_eq!(s.game.cam_x, 50, "camera x should be set");
    assert_eq!(s.game.cam_y, 50, "camera y should be set");
}

#[test]
fn center_camera_on_player() {
    let mut s = setup();
    s.perform_action(&AiAction::SetCamera { x: 0, y: 0 });
    s.step(30);
    s.perform_action(&AiAction::CenterCamera);
    assert!(
        s.game.cam_x > 0,
        "camera should move from 0 toward player: got {}",
        s.game.cam_x
    );
    assert!(
        s.game.cam_y > 0,
        "camera should move from 0 toward player: got {}",
        s.game.cam_y
    );
}

#[test]
fn kill_entity_removes_alive_status() {
    let mut s = setup();
    s.perform_action(&AiAction::Spawn {
        kind: "goblin".into(),
        x: 130.0,
        y: 120.0,
    });
    s.step(10);
    s.perform_action(&AiAction::KillEntity { id: 1 });
    s.step(1);
    let entities = s.get_entities();
    let g = entities.into_iter().find(|e| e.id == 1).unwrap();
    assert!(!g.alive, "entity should be dead after kill_entity");
}

#[test]
fn find_material_locates_existing() {
    let mut s = setup();
    s.perform_action(&AiAction::SetCell {
        x: 123,
        y: 115,
        material: "lava".into(),
    });
    let found = s.find_material("lava");
    assert!(found.is_some(), "find_material should locate lava");
    let (fx, fy) = found.unwrap();
    assert_eq!(fx, 123, "found x should match");
    assert_eq!(fy, 115, "found y should match");
}

#[test]
fn find_material_returns_none_for_absent() {
    let s = setup();
    let found = s.find_material("lava");
    assert!(
        found.is_none(),
        "find_material should return None for absent material"
    );
}
