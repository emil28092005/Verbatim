use verbatim::ai::AiAction;
use verbatim::ai::GameSession;

fn setup_empty() -> GameSession {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(95, 95, 40, 40);
    s
}

#[test]
fn acid_dissolves_wood() {
    let mut s = setup_empty();
    s.perform_action(&AiAction::FillRect {
        x: 100,
        y: 120,
        w: 10,
        h: 1,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::SetCell {
        x: 104,
        y: 118,
        material: "wood".into(),
    });
    s.perform_action(&AiAction::SetCell {
        x: 105,
        y: 119,
        material: "acid".into(),
    });
    s.perform_action(&AiAction::SetCell {
        x: 104,
        y: 119,
        material: "acid".into(),
    });
    s.step(30);
    assert_ne!(
        s.get_cell(104, 118).material,
        "wood",
        "acid should have dissolved the wood"
    );
}

#[test]
fn acid_does_not_dissolve_stone() {
    let mut s = setup_empty();
    s.perform_action(&AiAction::SetCell {
        x: 105,
        y: 110,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::SetCell {
        x: 105,
        y: 111,
        material: "acid".into(),
    });
    s.step(20);
    assert_eq!(
        s.get_cell(105, 110).material,
        "stone",
        "acid should not dissolve stone"
    );
}

#[test]
fn acid_flows_down() {
    let mut s = setup_empty();
    s.perform_action(&AiAction::FillRect {
        x: 100,
        y: 120,
        w: 10,
        h: 1,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::SetCell {
        x: 105,
        y: 105,
        material: "acid".into(),
    });
    s.step(20);
    assert_ne!(
        s.get_cell(105, 105).material,
        "acid",
        "acid should have flowed down from y=105"
    );
}
