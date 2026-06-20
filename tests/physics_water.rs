use verbatim::ai::GameSession;
use verbatim::ai::AiAction;

fn setup_empty() -> GameSession {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(95, 95, 40, 40);
    s
}

#[test]
fn water_flows_down() {
    let mut s = setup_empty();
    s.perform_action(&AiAction::FillRect { x: 95, y: 115, w: 40, h: 3, material: "stone".into() });
    s.perform_action(&AiAction::SetCell { x: 110, y: 105, material: "water".into() });
    s.perform_action(&AiAction::SetCell { x: 110, y: 106, material: "water".into() });
    s.perform_action(&AiAction::SetCell { x: 110, y: 107, material: "water".into() });
    s.step(20);
    let water_near_bottom = s.count_material_in_region(105, 112, 10, 4, "water");
    assert!(water_near_bottom > 0, "water should have flowed down to near the stone floor, found {} water cells near bottom", water_near_bottom);
}

#[test]
fn water_spreads_sideways() {
    let mut s = setup_empty();
    s.perform_action(&AiAction::FillRect { x: 95, y: 115, w: 50, h: 3, material: "stone".into() });
    s.perform_action(&AiAction::FillRect { x: 110, y: 111, w: 1, h: 4, material: "water".into() });
    s.step(50);
    let left_count = s.count_material_in_region(100, 110, 10, 5, "water");
    let right_count = s.count_material_in_region(111, 110, 10, 5, "water");
    assert!(left_count > 0 || right_count > 0, "water should spread sideways: left={} right={}", left_count, right_count);
}

#[test]
fn water_does_not_pass_through_stone_wall() {
    let mut s = setup_empty();
    s.perform_action(&AiAction::FillRect { x: 95, y: 115, w: 50, h: 3, material: "stone".into() });
    s.perform_action(&AiAction::FillRect { x: 115, y: 110, w: 1, h: 5, material: "stone".into() });
    s.perform_action(&AiAction::FillRect { x: 100, y: 110, w: 15, h: 5, material: "water".into() });
    s.step(30);
    let right_water = s.count_material_in_region(116, 108, 10, 10, "water");
    assert_eq!(right_water, 0, "water should not pass through stone wall");
}
