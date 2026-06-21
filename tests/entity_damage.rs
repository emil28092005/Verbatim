use verbatim::ai::AiAction;
use verbatim::ai::GameSession;

fn setup_empty() -> GameSession {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(95, 95, 40, 40);
    s
}

#[test]
fn entity_takes_lava_damage() {
    let mut s = setup_empty();
    s.perform_action(&AiAction::FillRect {
        x: 100,
        y: 130,
        w: 20,
        h: 3,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::FillRect {
        x: 114,
        y: 127,
        w: 8,
        h: 3,
        material: "lava".into(),
    });
    s.perform_action(&AiAction::Spawn {
        kind: "goblin".into(),
        x: 118.0,
        y: 118.0,
    });
    s.step(80);
    let entities = s.get_entities();
    let goblin = entities.into_iter().find(|e| e.kind == "Goblin");
    assert!(goblin.is_some(), "goblin should exist");
    let g = goblin.unwrap();
    assert!(
        g.health < 40.0,
        "goblin should have taken damage from lava, hp={}",
        g.health
    );
}

#[test]
fn entity_dies_becomes_corpse() {
    let mut s = setup_empty();
    s.perform_action(&AiAction::FillRect {
        x: 100,
        y: 125,
        w: 20,
        h: 1,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::Spawn {
        kind: "goblin".into(),
        x: 110.0,
        y: 120.0,
    });
    s.step(30);
    s.perform_action(&AiAction::DamageEntity {
        id: 1,
        amount: 100.0,
    });
    s.step(1);
    let entities = s.get_entities();
    let goblin = entities.into_iter().find(|e| e.id == 1);
    assert!(goblin.is_some(), "entity should still exist");
    let g = goblin.unwrap();
    assert!(!g.alive, "entity should be dead after 100 damage");
    assert_eq!(
        g.kind, "Corpse",
        "dead entity should be a corpse, got {}",
        g.kind
    );
}

#[test]
fn entity_on_fire_takes_damage_over_time() {
    let mut s = setup_empty();
    s.perform_action(&AiAction::FillRect {
        x: 100,
        y: 125,
        w: 20,
        h: 1,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::FillRect {
        x: 116,
        y: 123,
        w: 4,
        h: 2,
        material: "lava".into(),
    });
    s.perform_action(&AiAction::Spawn {
        kind: "goblin".into(),
        x: 118.0,
        y: 120.0,
    });
    s.step(20);
    let entities = s.get_entities();
    let goblin = entities.into_iter().find(|e| e.id == 1);
    if let Some(g) = goblin {
        if g.on_fire {
            let hp_after_fire = g.health;
            s.step(30);
            let entities2 = s.get_entities();
            if let Some(g2) = entities2.into_iter().find(|e| e.id == 1) {
                assert!(
                    g2.health < hp_after_fire,
                    "entity on fire should lose more health over time"
                );
            }
        }
    }
}

#[test]
fn entity_blocked_by_stone() {
    let mut s = setup_empty();
    s.perform_action(&AiAction::FillRect {
        x: 100,
        y: 130,
        w: 30,
        h: 1,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::FillRect {
        x: 110,
        y: 126,
        w: 1,
        h: 4,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::Spawn {
        kind: "goblin".into(),
        x: 105.0,
        y: 125.0,
    });
    s.step(30);
    let entities = s.get_entities();
    if let Some(g) = entities.into_iter().find(|e| e.id == 1) {
        assert!(
            g.pos[0] < 110.0,
            "goblin should be blocked by stone wall, got x={}",
            g.pos[0]
        );
    }
}
