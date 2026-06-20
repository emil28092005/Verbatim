use verbatim::ai::GameSession;
use verbatim::ai::AiAction;

fn setup_empty() -> GameSession {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(95, 95, 40, 35);
    s.perform_action(&AiAction::FillRect { x: 80, y: 135, w: 80, h: 15, material: "stone".into() });
    s
}

#[test]
fn player_falls_and_lands() {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(115, 115, 20, 15);
    s.perform_action(&AiAction::FillRect { x: 110, y: 130, w: 30, h: 15, material: "stone".into() });
    s.step(80);
    let player = s.get_player().expect("player should exist");
    assert!(player.alive, "player should be alive");
    let y = player.pos[1];
    assert!(y < 135.0, "player should not fall through stone floor, got y={}", y);
    s.step(30);
    let player2 = s.get_player().expect("player should exist");
    let dy = (player2.pos[1] - y).abs();
    assert!(dy < 3.0, "player should have stopped falling (dy={:.2}), y={} -> {}", dy, y, player2.pos[1]);
}

#[test]
fn player_blocked_by_stone_wall() {
    let mut s = setup_empty();
    s.step(30);
    let player = s.get_player().expect("player should exist");
    let x = player.pos[0] as i32;
    s.perform_action(&AiAction::FillRect { x: x + 5, y: 125, w: 1, h: 10, material: "stone".into() });
    s.perform_action(&AiAction::MoveRight);
    s.step(10);
    let player = s.get_player().expect("player should exist");
    assert!(player.pos[0] < (x + 5) as f32, "player should be blocked by wall");
}

#[test]
fn player_can_move_right() {
    let mut s = setup_empty();
    s.step(30);
    let player = s.get_player().expect("player should exist");
    let initial_x = player.pos[0];
    s.perform_action(&AiAction::MoveRight);
    s.step(10);
    let player = s.get_player().expect("player should exist");
    assert!(player.pos[0] > initial_x, "player should have moved right: {} -> {}", initial_x, player.pos[0]);
}

#[test]
fn player_survives_fall() {
    let mut s = setup_empty();
    s.step(60);
    let player = s.get_player().expect("player should exist");
    assert!(player.alive, "player should survive a fall onto stone");
    assert!(player.health > 50.0, "player should not take significant damage from landing, hp={}", player.health);
}
