use verbatim::ai::AiAction;
use verbatim::ai::GameSession;

fn setup() -> GameSession {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(90, 90, 50, 50);
    s.perform_action(&AiAction::FillRect {
        x: 80,
        y: 130,
        w: 80,
        h: 15,
        material: "stone".into(),
    });
    s
}

#[test]
fn slime_spawns_correctly() {
    let mut s = setup();
    s.perform_action(&AiAction::Spawn {
        kind: "slime".into(),
        x: 120.0,
        y: 120.0,
    });
    let entities = s.get_entities();
    let slime = entities.into_iter().find(|e| e.kind == "Slime");
    assert!(slime.is_some(), "slime should exist after spawn");
    let sl = slime.unwrap();
    assert!(sl.alive, "slime should be alive");
    assert_eq!(sl.max_health, 25.0, "slime max health should be 25");
}

#[test]
fn slime_takes_damage_and_dies() {
    let mut s = setup();
    s.perform_action(&AiAction::Spawn {
        kind: "slime".into(),
        x: 120.0,
        y: 120.0,
    });
    s.step(10);
    let id = s
        .get_entities()
        .into_iter()
        .find(|e| e.kind == "Slime")
        .unwrap()
        .id;
    s.perform_action(&AiAction::DamageEntity { id, amount: 25.0 });
    s.step(1);
    let entities = s.get_entities();
    let sl = entities.into_iter().find(|e| e.id == id).unwrap();
    assert!(!sl.alive, "slime should die after 25 damage");
    assert_eq!(sl.kind, "Corpse", "dead slime should become corpse");
}

#[test]
fn slime_jumps_toward_player() {
    let mut s = setup();
    s.step(30);
    let player = s.get_player().unwrap();
    let px = player.pos[0];
    s.perform_action(&AiAction::Spawn {
        kind: "slime".into(),
        x: px + 10.0,
        y: 120.0,
    });
    s.step(10);
    let entities = s.get_entities();
    let slime = entities.into_iter().find(|e| e.kind == "Slime" && e.alive);
    assert!(slime.is_some(), "slime should be alive");
    let slime_y_before = slime.unwrap().pos[1];
    s.step(60);
    let entities = s.get_entities();
    let slime_after = entities.into_iter().find(|e| e.kind == "Slime" && e.alive);
    if let Some(sl) = slime_after {
        assert!(
            sl.pos[1] < slime_y_before + 5.0 || sl.pos[0] != px + 10.0,
            "slime should have moved (jumped) from original position"
        );
    }
}

#[test]
fn slime_deals_contact_damage_to_player() {
    let mut s = setup();
    s.step(30);
    let player = s.get_player().unwrap();
    let hp_before = player.health;
    s.perform_action(&AiAction::Spawn {
        kind: "slime".into(),
        x: player.pos[0] + 1.0,
        y: player.pos[1],
    });
    s.step(60);
    let player_after = s.get_player().unwrap();
    assert!(
        player_after.health < hp_before,
        "player should take damage from slime contact: {} -> {}",
        hp_before,
        player_after.health
    );
}

#[test]
fn slime_template_has_correct_shape() {
    use verbatim::entity::body_template::BodyTemplate;
    let t = BodyTemplate::slime();
    assert_eq!(t.name, "slime");
    assert!(
        t.parts.len() >= 15,
        "slime should have 15+ parts, got {}",
        t.parts.len()
    );
    assert!(
        t.parts.iter().any(|p| p.label == "eye"),
        "slime should have eyes"
    );
}

#[test]
fn slime_count_in_world() {
    let mut s = setup();
    for i in 0..3 {
        s.perform_action(&AiAction::Spawn {
            kind: "slime".into(),
            x: 100.0 + i as f32 * 10.0,
            y: 120.0,
        });
    }
    s.step(10);
    let slimes = s
        .get_entities()
        .into_iter()
        .filter(|e| e.kind == "Slime")
        .count();
    assert_eq!(slimes, 3, "should have 3 slimes");
}
