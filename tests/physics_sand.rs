use verbatim::ai::GameSession;
use verbatim::ai::AiAction;
use verbatim::world::cell::MaterialId;

fn setup_empty() -> GameSession {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(95, 95, 30, 40);
    s
}

#[test]
fn sand_falls_down() {
    let mut s = setup_empty();
    s.perform_action(&AiAction::FillRect { x: 100, y: 115, w: 10, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 105, material: "sand".into() });
    s.step(10);
    assert_eq!(s.get_cell(105, 105).material, "empty", "sand should have fallen from y=105");
    assert_eq!(s.get_cell(105, 114).material, "sand", "sand should be resting on stone at y=114");
}

#[test]
fn sand_displaces_water() {
    let mut s = setup_empty();
    s.perform_action(&AiAction::FillRect { x: 100, y: 115, w: 10, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::FillRect { x: 100, y: 110, w: 10, h: 5, material: "water".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 105, material: "sand".into() });
    s.step(20);
    let sand_at_bottom = s.get_cell(105, 114).material == "sand";
    assert!(sand_at_bottom, "sand should sink to bottom through water");
}

#[test]
fn sand_piles_on_stone() {
    let mut s = setup_empty();
    s.perform_action(&AiAction::FillRect { x: 100, y: 115, w: 10, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 105, material: "sand".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 104, material: "sand".into() });
    s.step(15);
    let count = s.count_material_in_region(104, 112, 3, 4, "sand");
    assert!(count >= 2, "both sand cells should have piled up, got {} sand cells", count);
}

#[test]
fn sand_does_not_fall_through_stone() {
    let mut s = setup_empty();
    s.perform_action(&AiAction::FillRect { x: 100, y: 110, w: 10, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::SetCell { x: 105, y: 105, material: "sand".into() });
    s.step(10);
    assert_eq!(s.get_cell(105, 109).material, "sand", "sand should rest on top of stone");
    assert_eq!(s.get_cell(105, 110).material, "stone", "stone should remain");
}
