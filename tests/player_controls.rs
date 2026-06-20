use verbatim::ai::GameSession;
use verbatim::ai::AiAction;

fn setup() -> GameSession {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(90, 90, 50, 50);
    s.perform_action(&AiAction::FillRect { x: 80, y: 130, w: 80, h: 15, material: "stone".into() });
    s
}

#[test]
fn move_left_changes_x_position() {
    let mut s = setup();
    s.step(30);
    let p0 = s.get_player().unwrap();
    let x0 = p0.pos[0];
    s.perform_action(&AiAction::MoveLeft);
    s.step(5);
    let p1 = s.get_player().unwrap();
    assert!(p1.pos[0] < x0, "player should move left: {} -> {}", x0, p1.pos[0]);
}

#[test]
fn move_right_changes_x_position() {
    let mut s = setup();
    s.step(30);
    let p0 = s.get_player().unwrap();
    let x0 = p0.pos[0];
    s.perform_action(&AiAction::MoveRight);
    s.step(5);
    let p1 = s.get_player().unwrap();
    assert!(p1.pos[0] > x0, "player should move right: {} -> {}", x0, p1.pos[0]);
}

#[test]
fn move_left_then_right_cancels() {
    let mut s = setup();
    s.step(30);
    let p0 = s.get_player().unwrap();
    let x0 = p0.pos[0];
    s.perform_action(&AiAction::MoveLeft);
    s.step(5);
    s.perform_action(&AiAction::MoveRight);
    s.step(5);
    let p1 = s.get_player().unwrap();
    assert!((p1.pos[0] - x0).abs() < 3.0, "left+right should roughly cancel: {} -> {}", x0, p1.pos[0]);
}

#[test]
fn jump_goes_up_then_falls_back() {
    let mut s = setup();
    s.step(30);
    let p0 = s.get_player().unwrap();
    let y0 = p0.pos[1];
    s.perform_action(&AiAction::Jump);
    s.step(5);
    let p1 = s.get_player().unwrap();
    assert!(p1.pos[1] < y0, "player should go up after jump: {} -> {}", y0, p1.pos[1]);
    s.step(60);
    let p2 = s.get_player().unwrap();
    assert!((p2.pos[1] - y0).abs() < 5.0, "player should fall back after jump: {} -> {}", y0, p2.pos[1]);
}

#[test]
fn jump_while_airborne_does_nothing() {
    let mut s1 = setup();
    s1.step(30);
    s1.perform_action(&AiAction::Jump);
    s1.step(3);
    let mut s2 = setup();
    s2.step(30);
    s2.perform_action(&AiAction::Jump);
    s2.step(3);
    s2.perform_action(&AiAction::Jump);
    s1.step(3);
    s2.step(3);
    let p1 = s1.get_player().unwrap();
    let p2 = s2.get_player().unwrap();
    assert!((p1.pos[1] - p2.pos[1]).abs() < 2.0, "double jump should not add height: single={} double={}", p1.pos[1], p2.pos[1]);
}

#[test]
fn rapid_move_right_accumulates_velocity() {
    let mut s = setup();
    s.step(30);
    let p0 = s.get_player().unwrap();
    let x0 = p0.pos[0];
    for _ in 0..5 {
        s.perform_action(&AiAction::MoveRight);
        s.step(1);
    }
    let p1 = s.get_player().unwrap();
    let single = {
        let mut s2 = setup();
        s2.step(30);
        s2.perform_action(&AiAction::MoveRight);
        s2.step(1);
        s2.get_player().unwrap().pos[0]
    };
    let single_dx = single - x0;
    let multi_dx = p1.pos[0] - x0;
    assert!(multi_dx > single_dx, "rapid moves should accumulate: 1x={} 5x={}", single_dx, multi_dx);
}

#[test]
fn wait_does_not_move() {
    let mut s = setup();
    s.step(30);
    let p0 = s.get_player().unwrap();
    let pos0 = (p0.pos[0], p0.pos[1]);
    s.perform_action(&AiAction::Wait);
    s.step(10);
    let p1 = s.get_player().unwrap();
    assert!((p1.pos[0] - pos0.0).abs() < 1.0 && (p1.pos[1] - pos0.1).abs() < 1.0,
        "wait should not move player: ({},{}) -> ({},{})", pos0.0, pos0.1, p1.pos[0], p1.pos[1]);
}

#[test]
fn continuous_movement_does_not_fall_through_floor() {
    let mut s = setup();
    s.step(30);
    let p0 = s.get_player().unwrap();
    let y0 = p0.pos[1];
    for _ in 0..20 {
        s.perform_action(&AiAction::MoveRight);
        s.step(2);
    }
    let p1 = s.get_player().unwrap();
    assert!(p1.alive, "player should survive extended movement");
    assert!((p1.pos[1] - y0).abs() < 5.0, "player should not fall through floor during movement: y0={} y1={}", y0, p1.pos[1]);
}

#[test]
fn player_on_ground_check() {
    let mut s = setup();
    s.step(40);
    assert!(s.game.check_on_ground(), "player should be on ground after settling");
}

#[test]
fn player_not_on_ground_while_jumping() {
    let mut s = setup();
    s.step(40);
    assert!(s.game.check_on_ground(), "player should start on ground");
    s.perform_action(&AiAction::Jump);
    s.step(3);
    assert!(!s.game.check_on_ground(), "player should not be on ground mid-jump");
}

#[test]
fn player_health_stays_full_without_damage() {
    let mut s = setup();
    s.step(60);
    let p = s.get_player().unwrap();
    assert_eq!(p.health, 100.0, "player should have full health without damage");
}

#[test]
fn player_survives_long_idle() {
    let mut s = setup();
    s.step(200);
    let p = s.get_player().unwrap();
    assert!(p.alive, "player should survive 200 ticks of idle");
    assert!(p.health > 90.0, "player should not lose health idling");
}
