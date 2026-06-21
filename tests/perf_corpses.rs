use verbatim::ai::AiAction;
use verbatim::ai::GameSession;

fn setup() -> GameSession {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(90, 90, 80, 80);
    s
}

#[test]
fn corpses_do_not_cause_lag() {
    let mut s = setup();
    for i in 0..20 {
        s.perform_action(&AiAction::Spawn {
            kind: "goblin".into(),
            x: 100.0 + (i % 5) as f32 * 3.0,
            y: 100.0 + (i / 5) as f32 * 3.0,
        });
    }
    s.step(20);
    let ids: Vec<u32> = s
        .get_entities()
        .into_iter()
        .filter(|e| e.kind == "Goblin")
        .map(|e| e.id)
        .collect();

    let start_before = std::time::Instant::now();
    s.step(60);
    let before_ms = start_before.elapsed().as_secs_f32() * 1000.0 / 60.0;

    for id in &ids {
        s.perform_action(&AiAction::DamageEntity {
            id: *id,
            amount: 100.0,
        });
    }
    let start_after = std::time::Instant::now();
    s.step(60);
    let after_ms = start_after.elapsed().as_secs_f32() * 1000.0 / 60.0;

    assert!(
        after_ms < before_ms * 3.0,
        "corpse simulation should not be dramatically slower: before={:.2}ms after={:.2}ms",
        before_ms,
        after_ms
    );
}
