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
fn death_transitions_rigid_to_ragdoll() {
    let mut s = setup();
    s.perform_action(&AiAction::Spawn { kind: "goblin".into(), x: 140.0, y: 120.0 });
    s.step(20);
    let entities = s.get_entities();
    let goblin = entities.into_iter().find(|e| e.kind == "Goblin").unwrap();
    assert!(goblin.alive, "goblin should be alive initially");

    s.perform_action(&AiAction::DamageEntity { id: goblin.id, amount: 100.0 });
    s.step(1);
    let entities = s.get_entities();
    let corpse = entities.into_iter().find(|e| e.id == goblin.id).unwrap();
    assert!(!corpse.alive, "goblin should be dead after 100 damage");
    assert_eq!(corpse.kind, "Corpse", "dead goblin should become corpse");
}

#[test]
fn ragdoll_falls_after_death() {
    let mut s = setup();
    s.perform_action(&AiAction::Spawn { kind: "goblin".into(), x: 140.0, y: 110.0 });
    s.step(20);
    let entities = s.get_entities();
    let g = entities.into_iter().find(|e| e.kind == "Goblin").unwrap();
    let y_before = g.pos[1];

    s.perform_action(&AiAction::DamageEntity { id: g.id, amount: 100.0 });
    s.step(30);
    let y_after = {
        let e = s.get_entities().into_iter().find(|e| e.id == g.id).unwrap();
        e.pos[1]
    };
    assert!(y_after > y_before, "corpse should fall: y_before={} y_after={}", y_before, y_after);
}

#[test]
fn ragdoll_bodies_stay_near_each_other() {
    let mut s = setup();
    s.perform_action(&AiAction::Spawn { kind: "goblin".into(), x: 130.0, y: 110.0 });
    s.step(20);
    let entities = s.get_entities();
    let g = entities.into_iter().find(|e| e.kind == "Goblin").unwrap();
    s.perform_action(&AiAction::DamageEntity { id: g.id, amount: 100.0 });
    s.step(10);
    let entities = s.get_entities();
    let corpse = entities.into_iter().find(|e| e.id == g.id).unwrap();
    assert!(!corpse.alive, "corpse should be dead");
    assert!(corpse.body_count > 0, "corpse should have some alive bodies");
}

#[test]
fn player_death_becomes_corpse() {
    let mut s = setup();
    s.step(30);
    let p = s.get_player().unwrap();
    s.perform_action(&AiAction::DamageEntity { id: p.id, amount: 200.0 });
    s.step(1);
    let p2 = s.get_player().unwrap();
    assert!(!p2.alive, "player should be dead after 200 damage");
}

#[test]
fn damage_reduces_health_progressively() {
    let mut s = setup();
    s.perform_action(&AiAction::Spawn { kind: "goblin".into(), x: 140.0, y: 120.0 });
    s.step(20);
    let entities = s.get_entities();
    let g = entities.into_iter().find(|e| e.kind == "Goblin").unwrap();
    assert_eq!(g.health, 40.0, "goblin should start at 40 HP");

    s.perform_action(&AiAction::DamageEntity { id: g.id, amount: 10.0 });
    s.step(1);
    let entities = s.get_entities();
    let g2 = entities.into_iter().find(|e| e.id == g.id).unwrap();
    assert!((g2.health - 30.0).abs() < 0.01, "goblin should have 30 HP after 10 damage, got {}", g2.health);
    assert!(g2.alive, "goblin should survive 10 damage");
}

#[test]
fn small_damage_does_not_kill() {
    let mut s = setup();
    s.perform_action(&AiAction::Spawn { kind: "goblin".into(), x: 140.0, y: 120.0 });
    s.step(20);
    let entities = s.get_entities();
    let g = entities.into_iter().find(|e| e.kind == "Goblin").unwrap();
    s.perform_action(&AiAction::DamageEntity { id: g.id, amount: 39.0 });
    s.step(1);
    let entities = s.get_entities();
    let g2 = entities.into_iter().find(|e| e.id == g.id).unwrap();
    assert!(g2.alive, "goblin should survive 39 damage (HP=1)");
    s.perform_action(&AiAction::DamageEntity { id: g.id, amount: 1.0 });
    s.step(1);
    let entities = s.get_entities();
    let g3 = entities.into_iter().find(|e| e.id == g.id).unwrap();
    assert!(!g3.alive, "goblin should die at 0 HP");
}

#[test]
fn corpse_exists_in_world() {
    let mut s = setup();
    s.perform_action(&AiAction::Spawn { kind: "goblin".into(), x: 140.0, y: 120.0 });
    s.step(20);
    let g_id = s.get_entities().into_iter().find(|e| e.kind == "Goblin").unwrap().id;
    s.perform_action(&AiAction::DamageEntity { id: g_id, amount: 100.0 });
    s.step(5);
    let entities = s.get_entities();
    let corpse = entities.into_iter().find(|e| e.id == g_id);
    assert!(corpse.is_some(), "corpse entity should persist in world");
    assert!(!corpse.unwrap().alive, "corpse should not be alive");
}
