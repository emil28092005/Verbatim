use verbatim::ai::GameSession;
use verbatim::ai::AiAction;

fn setup_empty() -> GameSession {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(95, 95, 40, 40);
    s
}

#[test]
fn lava_flows_down() {
    let mut s = setup_empty();
    s.perform_action(&AiAction::FillRect { x: 100, y: 115, w: 10, h: 3, material: "stone".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 110, material: "lava".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 111, material: "lava".into() });
    s.step(10);
    let lava_count = s.count_material_in_region(103, 110, 5, 6, "lava");
    let stone_count = s.count_material_in_region(103, 110, 5, 6, "stone");
    assert!(lava_count > 0 || stone_count >= 4,
        "lava should have flowed down or cooled to stone, lava={} stone={}", lava_count, stone_count);
}

#[test]
fn lava_plus_water_makes_steam() {
    let mut s = setup_empty();
    s.perform_action(&AiAction::FillRect { x: 100, y: 120, w: 20, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::FillRect { x: 100, y: 99, w: 20, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 110, material: "lava".into() });
    s.perform_action(&AiAction::SetCell { x: 106, y: 110, material: "water".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 109, material: "lava".into() });
    s.perform_action(&AiAction::SetCell { x: 106, y: 109, material: "water".into() });
    s.step(20);
    let lava_remaining = s.count_material_in_region(100, 105, 20, 15, "lava");
    assert_eq!(lava_remaining, 0, "all lava should have been converted by water");
}

#[test]
fn lava_ignites_wood() {
    let mut s = setup_empty();
    s.perform_action(&AiAction::FillRect { x: 100, y: 115, w: 10, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::SetCell { x: 104, y: 114, material: "wood".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 114, material: "wood".into() });
    s.perform_action(&AiAction::SetCell { x: 106, y: 114, material: "wood".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 113, material: "lava".into() });
    s.step(15);
    let wood_remaining = s.count_material_in_region(103, 113, 5, 3, "wood");
    assert_eq!(wood_remaining, 0, "all wood should have been ignited by lava, got {} wood cells", wood_remaining);
}

#[test]
fn lava_ignites_grass() {
    let mut s = setup_empty();
    s.perform_action(&AiAction::FillRect { x: 100, y: 115, w: 10, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::FillRect { x: 100, y: 114, w: 5, h: 1, material: "grass".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 113, material: "lava".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 114, material: "lava".into() });
    s.step(15);
    let grass_remaining = s.count_material_in_region(99, 113, 7, 3, "grass");
    assert_eq!(grass_remaining, 0, "grass should have been ignited by lava");
}
