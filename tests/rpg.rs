use verbatim::ai::AiAction;
use verbatim::ai::GameSession;
use verbatim::entity::EntityKind;

fn setup() -> GameSession {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(90, 90, 50, 50);
    s
}

#[test]
fn player_has_stats() {
    let mut s = setup();
    let p = s.get_player().unwrap();
    assert!(p.strength > 0, "player should have strength");
    assert!(p.agility > 0, "player should have agility");
    assert!(p.toughness > 0, "player should have toughness");
    assert!(p.willpower > 0, "player should have willpower");
}

#[test]
fn goblin_has_stats() {
    let mut s = setup();
    s.perform_action(&AiAction::Spawn {
        kind: "goblin".into(),
        x: 120.0,
        y: 120.0,
    });
    let g = s
        .get_entities()
        .into_iter()
        .find(|e| e.kind == "Goblin")
        .unwrap();
    assert!(g.strength > 0, "goblin should have strength");
    assert!(g.toughness > 0, "goblin should have toughness");
}

#[test]
fn max_health_depends_on_toughness() {
    let mut s = setup();
    let p = s.get_player().unwrap();
    let expected = 80.0 + p.toughness as f32 * 5.0 + p.level as f32 * 10.0;
    assert!(
        (p.max_health - expected).abs() < 0.01,
        "max health should be based on toughness and level"
    );
}

#[test]
fn killing_grants_xp() {
    let mut s = setup();
    s.perform_action(&AiAction::Spawn {
        kind: "goblin".into(),
        x: 120.0,
        y: 120.0,
    });
    s.step(10);
    let id = s
        .get_entities()
        .into_iter()
        .find(|e| e.kind == "Goblin")
        .unwrap()
        .id;
    let xp_before = s.get_player().unwrap().xp;
    s.perform_action(&AiAction::DamageEntity { id, amount: 100.0 });
    s.step(1);
    let xp_after = s.get_player().unwrap().xp;
    assert!(xp_after > xp_before, "killing an enemy should grant XP");
}

#[test]
fn xp_accumulation_levels_up() {
    let mut s = setup();
    let p = s.game.player.entity_mut(&mut s.game.entities).unwrap();
    p.add_xp(100);
    assert_eq!(p.level, 2, "100 XP should level up from 1 to 2");
    assert_eq!(p.xp, 0, "XP should be reset after level up");
}

#[test]
fn level_up_increases_max_health() {
    let mut s = setup();
    let before = s.get_player().unwrap().max_health;
    let p = s.game.player.entity_mut(&mut s.game.entities).unwrap();
    p.add_xp(100);
    let after = s.get_player().unwrap().max_health;
    assert!(after > before, "level up should increase max health");
}

#[test]
fn poison_deals_damage_over_time() {
    let mut s = setup();
    let id = s.get_player().unwrap().id;
    s.perform_action(&AiAction::DamageEntity { id, amount: 10.0 });
    let before = s.get_player().unwrap().health;
    if let Some(p) = s.game.player.entity_mut(&mut s.game.entities) {
        p.poisoned = true;
    }
    s.step(10);
    let after = s.get_player().unwrap().health;
    assert!(after < before, "poison should deal damage over time");
}

#[test]
fn bleeding_deals_damage_over_time() {
    let mut s = setup();
    let id = s.get_player().unwrap().id;
    s.perform_action(&AiAction::DamageEntity { id, amount: 10.0 });
    let before = s.get_player().unwrap().health;
    if let Some(p) = s.game.player.entity_mut(&mut s.game.entities) {
        p.bleeding = true;
    }
    s.step(10);
    let after = s.get_player().unwrap().health;
    assert!(after < before, "bleeding should deal damage over time");
}

#[test]
fn frozen_does_not_deal_damage() {
    let mut s = setup();
    let id = s.get_player().unwrap().id;
    s.perform_action(&AiAction::DamageEntity { id, amount: 10.0 });
    let before = s.get_player().unwrap().health;
    if let Some(p) = s.game.player.entity_mut(&mut s.game.entities) {
        p.frozen = true;
    }
    s.step(10);
    let after = s.get_player().unwrap().health;
    assert!(
        (after - before).abs() < 1.0,
        "frozen should not deal damage directly"
    );
}

#[test]
fn status_effects_expire() {
    let mut s = setup();
    if let Some(p) = s.game.player.entity_mut(&mut s.game.entities) {
        p.poisoned = true;
        p.poison_timer = 200;
    }
    s.step(200);
    let p = s.get_player().unwrap();
    assert!(!p.poisoned, "poison should expire after timer");
}

#[test]
fn level_up_heals_to_full() {
    let mut s = setup();
    let p = s.game.player.entity_mut(&mut s.game.entities).unwrap();
    p.health = 10.0;
    p.add_xp(100);
    assert_eq!(p.health, p.max_health, "level up should heal to full");
}

#[test]
fn entity_info_includes_level() {
    let mut s = setup();
    let p = s.get_player().unwrap();
    assert_eq!(p.level, 1, "player should start at level 1");
}

#[test]
fn corpse_does_not_gain_xp() {
    let mut s = setup();
    let player_id = s.get_player().unwrap().id;
    s.perform_action(&AiAction::DamageEntity {
        id: player_id,
        amount: 999.0,
    });
    s.step(1);
    let p = s.game.player.entity_mut(&mut s.game.entities).unwrap();
    let xp_before = p.xp;
    p.add_xp(100);
    assert_eq!(p.xp, xp_before, "dead player should not gain XP");
}
