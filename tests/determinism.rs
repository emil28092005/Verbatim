use verbatim::ai::GameSession;
use verbatim::ai::AiAction;
use verbatim::ai::ReplayPlayer;

fn setup() -> GameSession {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(90, 90, 50, 50);
    s.perform_action(&AiAction::FillRect { x: 80, y: 130, w: 80, h: 15, material: "stone".into() });
    s
}

#[test]
fn same_seed_same_player_position() {
    let mut s1 = GameSession::new_seeded(777);
    s1.init();
    s1.step(30);
    let mut s2 = GameSession::new_seeded(777);
    s2.init();
    s2.step(30);
    let p1 = s1.get_player().unwrap();
    let p2 = s2.get_player().unwrap();
    assert!((p1.pos[0] - p2.pos[0]).abs() < 0.01, "x mismatch");
    assert!((p1.pos[1] - p2.pos[1]).abs() < 0.01, "y mismatch");
}

#[test]
fn same_seed_same_entity_count() {
    let mut s1 = GameSession::new_seeded(555);
    s1.init();
    s1.step(60);
    let mut s2 = GameSession::new_seeded(555);
    s2.init();
    s2.step(60);
    assert_eq!(s1.get_entities().len(), s2.get_entities().len(), "entity count should match");
}

#[test]
fn same_seed_same_grid_state() {
    let mut s1 = GameSession::new_seeded(333);
    s1.init();
    s1.step(20);
    let mut s2 = GameSession::new_seeded(333);
    s2.init();
    s2.step(20);
    for y in 100..150 {
        for x in 100..150 {
            let c1 = s1.get_cell(x, y);
            let c2 = s2.get_cell(x, y);
            assert_eq!(c1.material, c2.material, "material mismatch at ({},{})", x, y);
        }
    }
}

// Note: different seeds currently produce the same world because
// the CA RNG is internal (not seeded from GameSession).
// This is a known limitation — world gen is deterministic regardless of seed.
// Seed matters for replay recording/determinism within a session.

#[test]
fn replay_exact_match() {
    let mut s = GameSession::new_seeded(888);
    s.init();
    s.set_recording(true);
    s.perform_action(&AiAction::MoveRight);
    s.step(10);
    s.perform_action(&AiAction::Jump);
    s.step(10);
    s.perform_action(&AiAction::MoveLeft);
    s.step(10);
    let state_orig = s.get_state();
    s.save_replay("/tmp/verbatim_replay_exact.json").expect("save");

    let player = ReplayPlayer::load("/tmp/verbatim_replay_exact.json").expect("load");
    let s2 = player.play();
    let state_replay = s2.get_state();

    assert_eq!(state_orig.tick, state_replay.tick, "tick mismatch");
    if let (Some(p1), Some(p2)) = (&state_orig.player, &state_replay.player) {
        assert!((p1.pos[0] - p2.pos[0]).abs() < 0.1, "x mismatch: {} vs {}", p1.pos[0], p2.pos[0]);
        assert!((p1.pos[1] - p2.pos[1]).abs() < 0.1, "y mismatch: {} vs {}", p1.pos[1], p2.pos[1]);
        assert!((p1.health - p2.health).abs() < 1.0, "health mismatch");
    }
}

#[test]
fn replay_stop_at_tick() {
    let mut s = GameSession::new_seeded(444);
    s.init();
    s.set_recording(true);
    s.step(20);
    s.perform_action(&AiAction::MoveRight);
    s.step(20);
    s.save_replay("/tmp/verbatim_replay_partial.json").expect("save");

    let player = ReplayPlayer::load("/tmp/verbatim_replay_partial.json").expect("load");
    let s_half = player.play_until_tick(10);
    assert_eq!(s_half.tick(), 10, "should stop at tick 10, got {}", s_half.tick());

    let s_full = player.play();
    assert_eq!(s_full.tick(), 40, "full replay should reach tick 40, got {}", s_full.tick());
}

#[test]
fn recording_captures_all_actions() {
    let mut s = GameSession::new_seeded(123);
    s.init();
    s.set_recording(true);
    s.perform_action(&AiAction::MoveRight);
    s.perform_action(&AiAction::Jump);
    s.perform_action(&AiAction::MoveLeft);
    s.step(5);
    s.save_replay("/tmp/verbatim_replay_capture.json").expect("save");

    let player = ReplayPlayer::load("/tmp/verbatim_replay_capture.json").expect("load");
    let event_count = player.recording().events.len();
    assert!(event_count >= 4, "recording should have at least 4 events (3 actions + 1 step), got {}", event_count);
}

#[test]
fn determinism_with_spawn_and_damage() {
    let mut s1 = setup();
    s1.perform_action(&AiAction::Spawn { kind: "goblin".into(), x: 135.0, y: 120.0 });
    s1.perform_action(&AiAction::DamageEntity { id: 1, amount: 20.0 });
    s1.step(30);

    let mut s2 = setup();
    s2.perform_action(&AiAction::Spawn { kind: "goblin".into(), x: 135.0, y: 120.0 });
    s2.perform_action(&AiAction::DamageEntity { id: 1, amount: 20.0 });
    s2.step(30);

    let e1 = s1.get_entities().into_iter().find(|e| e.id == 1).unwrap();
    let e2 = s2.get_entities().into_iter().find(|e| e.id == 1).unwrap();
    assert!((e1.health - e2.health).abs() < 0.01, "health should match: {} vs {}", e1.health, e2.health);
    assert!((e1.pos[0] - e2.pos[0]).abs() < 0.1, "x should match");
    assert!((e1.pos[1] - e2.pos[1]).abs() < 0.1, "y should match");
}

#[test]
fn hundred_tick_determinism() {
    let mut s1 = GameSession::new_seeded(99);
    s1.init();
    s1.perform_action(&AiAction::MoveRight);
    s1.step(100);

    let mut s2 = GameSession::new_seeded(99);
    s2.init();
    s2.perform_action(&AiAction::MoveRight);
    s2.step(100);

    let p1 = s1.get_player().unwrap();
    let p2 = s2.get_player().unwrap();
    assert!((p1.pos[0] - p2.pos[0]).abs() < 0.01, "100-tick determinism failed: {} vs {}", p1.pos[0], p2.pos[0]);
}
