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
fn player_blocked_by_left_wall() {
    let mut s = setup();
    s.step(30);
    let p = s.get_player().unwrap();
    let wall_x = (p.pos[0] as i32) - 6;
    s.perform_action(&AiAction::FillRect { x: wall_x, y: 125, w: 1, h: 10, material: "stone".into() });
    s.perform_action(&AiAction::MoveLeft);
    s.step(20);
    let p2 = s.get_player().unwrap();
    assert!(p2.pos[0] > wall_x as f32, "player should not pass through left wall: wall={} player={}", wall_x, p2.pos[0]);
}

#[test]
fn player_blocked_by_right_wall() {
    let mut s = setup();
    s.step(30);
    let p = s.get_player().unwrap();
    let wall_x = (p.pos[0] as i32) + 6;
    s.perform_action(&AiAction::FillRect { x: wall_x, y: 125, w: 1, h: 10, material: "stone".into() });
    s.perform_action(&AiAction::MoveRight);
    s.step(20);
    let p2 = s.get_player().unwrap();
    assert!(p2.pos[0] < wall_x as f32, "player should not pass through right wall: wall={} player={}", wall_x, p2.pos[0]);
}

#[test]
fn player_slides_along_wall() {
    let mut s = setup();
    s.step(30);
    let p = s.get_player().unwrap();
    let wall_x = (p.pos[0] as i32) + 6;
    s.perform_action(&AiAction::FillRect { x: wall_x, y: 125, w: 1, h: 10, material: "stone".into() });
    s.perform_action(&AiAction::MoveRight);
    s.step(30);
    let p2 = s.get_player().unwrap();
    assert!(p2.pos[0] < wall_x as f32, "player should stay left of wall");
    assert!(p2.alive, "player should be alive");
    let y_diff = (p2.pos[1] - p.pos[1]).abs();
    assert!(y_diff < 5.0, "player should not fall through floor while sliding: dy={}", y_diff);
}

#[test]
fn player_blocked_by_ceiling() {
    let mut s = setup();
    s.step(30);
    let p = s.get_player().unwrap();
    let ceiling_y = (p.pos[1] as i32) - 8;
    s.perform_action(&AiAction::FillRect { x: p.pos[0] as i32 - 5, y: ceiling_y, w: 10, h: 1, material: "stone".into() });
    s.perform_action(&AiAction::Jump);
    s.step(10);
    let p2 = s.get_player().unwrap();
    assert!(p2.pos[1] > ceiling_y as f32, "player should not pass through ceiling: ceiling={} player={}", ceiling_y, p2.pos[1]);
}

#[test]
fn player_navigates_corridor() {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(100, 100, 40, 30);
    s.perform_action(&AiAction::FillRect { x: 95, y: 120, w: 50, h: 10, material: "stone".into() });
    s.perform_action(&AiAction::FillRect { x: 115, y: 110, w: 1, h: 10, material: "stone".into() });
    s.perform_action(&AiAction::FillRect { x: 125, y: 110, w: 1, h: 10, material: "stone".into() });
    s.perform_action(&AiAction::FillRect { x: 115, y: 108, w: 11, h: 1, material: "stone".into() });
    s.step(30);
    let p = s.get_player().unwrap();
    assert!(p.pos[0] < 115.0 || p.pos[0] > 125.0, "player should be outside corridor initially");
}

#[test]
fn player_does_not_stick_to_wall() {
    let mut s = setup();
    s.step(30);
    let p = s.get_player().unwrap();
    let wall_x = (p.pos[0] as i32) + 8;
    s.perform_action(&AiAction::FillRect { x: wall_x, y: 125, w: 1, h: 10, material: "stone".into() });
    for _ in 0..10 {
        s.perform_action(&AiAction::MoveRight);
        s.step(2);
    }
    let p_at_wall = s.get_player().unwrap();
    let x_at_wall = p_at_wall.pos[0];
    for _ in 0..20 {
        s.perform_action(&AiAction::MoveLeft);
        s.step(2);
    }
    let p_away = s.get_player().unwrap();
    assert!(p_away.pos[0] < x_at_wall - 1.0, "player should move away from wall: {} -> {}", x_at_wall, p_away.pos[0]);
}

#[test]
fn player_squeezes_through_gap() {
    let mut s = setup();
    s.step(30);
    let p = s.get_player().unwrap();
    let px = p.pos[0] as i32;
    s.perform_action(&AiAction::FillRect { x: px + 8, y: 128, w: 1, h: 2, material: "stone".into() });
    s.perform_action(&AiAction::FillRect { x: px + 8, y: 122, w: 1, h: 2, material: "stone".into() });
    s.perform_action(&AiAction::MoveRight);
    s.step(20);
    let p2 = s.get_player().unwrap();
    assert!(p2.alive, "player should survive squeeze attempt");
}

#[test]
fn player_blocked_by_two_walls_both_sides() {
    let mut s = setup();
    s.step(30);
    let p = s.get_player().unwrap();
    let px = p.pos[0] as i32;
    s.perform_action(&AiAction::FillRect { x: px - 6, y: 125, w: 1, h: 10, material: "stone".into() });
    s.perform_action(&AiAction::FillRect { x: px + 6, y: 125, w: 1, h: 10, material: "stone".into() });
    s.perform_action(&AiAction::MoveRight);
    s.step(10);
    let p_right = s.get_player().unwrap();
    s.perform_action(&AiAction::MoveLeft);
    s.step(10);
    let p_left = s.get_player().unwrap();
    assert!(p_right.pos[0] < (px + 6) as f32, "blocked right");
    assert!(p_left.pos[0] > (px - 6) as f32, "blocked left");
    assert!((p_left.pos[0] - p_right.pos[0]).abs() < 12.0, "player should stay between walls");
}

#[test]
fn player_walks_up_slope() {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(90, 100, 50, 35);
    for x in 100..130 {
        let h = ((x - 100) / 3).min(15);
        for y in 0..h {
            s.perform_action(&AiAction::SetCell { x, y: 135 - 1 - y, material: "stone".into() });
        }
    }
    s.step(40);
    let p = s.get_player().unwrap();
    assert!(p.alive, "player should survive on slope");
    assert!(p.pos[1] < 135.0, "player should be above floor on slope");
}

#[test]
fn entity_collision_with_dirt_wall() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect { x: 140, y: 125, w: 1, h: 5, material: "dirt".into() });
    s.perform_action(&AiAction::Spawn { kind: "goblin".into(), x: 135.0, y: 120.0 });
    s.step(40);
    let entities = s.get_entities();
    if let Some(g) = entities.into_iter().find(|e| e.kind == "Goblin" && e.alive) {
        assert!(g.pos[0] < 140.0, "goblin should be blocked by dirt wall: x={}", g.pos[0]);
    }
}
