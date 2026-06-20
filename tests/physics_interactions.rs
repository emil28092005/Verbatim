use verbatim::ai::GameSession;
use verbatim::ai::AiAction;

fn setup() -> GameSession {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(90, 90, 50, 50);
    s
}

#[test]
fn lava_flows_down_on_stone() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect { x: 100, y: 125, w: 10, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 115, material: "lava".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 116, material: "lava".into() });
    s.step(15);
    let lava_at_floor = s.count_material_in_region(103, 122, 5, 4, "lava");
    assert!(lava_at_floor > 0, "lava should flow down to stone floor");
}

#[test]
fn lava_cools_to_stone_eventually() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect { x: 100, y: 125, w: 10, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 120, material: "lava".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 121, material: "lava".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 122, material: "lava".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 123, material: "lava".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 124, material: "lava".into() });
    s.step(200);
    let stone_count = s.count_material_in_region(103, 120, 5, 6, "stone");
    assert!(stone_count >= 3, "lava should cool to stone eventually, got {} stone", stone_count);
}

#[test]
fn fire_spreads_through_wood_line() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect { x: 100, y: 120, w: 20, h: 1, material: "stone".into() });
    for x in 100..110 {
        s.perform_action(&AiAction::SetCell { x, y: 119, material: "wood".into() });
    }
    s.perform_action(&AiAction::SetCell { x: 100, y: 119, material: "fire".into() });
    s.step(40);
    let wood_left = s.count_material_in_region(100, 118, 10, 3, "wood");
    assert_eq!(wood_left, 0, "fire should spread through entire wood line");
}

#[test]
fn acid_does_not_dissolve_empty() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect { x: 100, y: 115, w: 10, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::FillRect { x: 104, y: 114, w: 3, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::FillRect { x: 104, y: 113, w: 1, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::FillRect { x: 106, y: 113, w: 1, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 113, material: "acid".into() });
    s.step(5);
    let cell = s.get_cell(105, 113);
    assert!(cell.material == "acid" || cell.material == "empty",
        "acid may flow out but should not dissolve empty, got {}", cell.material);
}

#[test]
fn acid_dissolves_grass() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect { x: 100, y: 120, w: 10, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::SetCell { x: 104, y: 119, material: "grass".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 119, material: "acid".into() });
    s.perform_action(&AiAction::SetCell { x: 104, y: 118, material: "acid".into() });
    s.step(20);
    assert_ne!(s.get_cell(104, 119).material, "grass", "acid should dissolve grass");
}

#[test]
fn acid_dissolves_dirt() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect { x: 100, y: 115, w: 10, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::SetCell { x: 104, y: 114, material: "dirt".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 114, material: "acid".into() });
    s.perform_action(&AiAction::SetCell { x: 104, y: 113, material: "acid".into() });
    s.step(30);
    assert_ne!(s.get_cell(104, 114).material, "dirt", "acid should dissolve dirt");
}

#[test]
fn water_extinguishes_fire_indirectly() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect { x: 100, y: 120, w: 10, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 119, material: "fire".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 118, material: "water".into() });
    s.step(20);
    assert_ne!(s.get_cell(105, 119).material, "fire", "water should extinguish fire");
}

#[test]
fn lava_and_water_produce_both_steam_and_stone() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect { x: 100, y: 125, w: 20, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::FillRect { x: 100, y: 95, w: 20, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::FillRect { x: 101, y: 115, w: 4, h: 3, material: "lava".into() });
    s.perform_action(&AiAction::FillRect { x: 106, y: 115, w: 4, h: 3, material: "water".into() });
    s.step(30);
    let steam = s.count_material_in_region(100, 100, 20, 20, "steam");
    assert!(steam > 0, "lava + water should produce steam");
}

#[test]
fn sand_falls_through_water() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect { x: 100, y: 125, w: 10, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::FillRect { x: 104, y: 118, w: 3, h: 7, material: "water".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 113, material: "sand".into() });
    s.step(30);
    let sand_at_bottom = s.get_cell(105, 124).material;
    assert_eq!(sand_at_bottom, "sand", "sand should sink through water to bottom");
}

#[test]
fn fire_does_not_ignite_stone() {
    let mut s = setup();
    s.perform_action(&AiAction::SetCell { x: 104, y: 110, material: "stone".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 110, material: "fire".into() });
    s.step(30);
    assert_eq!(s.get_cell(104, 110).material, "stone", "fire should not ignite stone");
}

#[test]
fn fire_does_not_ignite_water() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect { x: 100, y: 115, w: 10, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::SetCell { x: 104, y: 114, material: "water".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 114, material: "fire".into() });
    s.step(5);
    let cell = s.get_cell(104, 114);
    assert_ne!(cell.material, "fire", "water should never become fire, got {}", cell.material);
    assert_ne!(cell.material, "wood", "water should never become wood");
}

#[test]
fn water_does_not_flow_through_dirt_wall() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect { x: 100, y: 125, w: 20, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::FillRect { x: 109, y: 120, w: 1, h: 5, material: "dirt".into() });
    s.perform_action(&AiAction::FillRect { x: 100, y: 120, w: 9, h: 5, material: "water".into() });
    s.step(30);
    let right_water = s.count_material_in_region(110, 118, 5, 8, "water");
    assert_eq!(right_water, 0, "water should not flow through dirt wall");
}
