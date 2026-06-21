use verbatim::ai::{AiAction, GameSession};

fn setup() -> GameSession {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(80, 80, 80, 80);
    s.perform_action(&AiAction::FillRect {
        x: 80,
        y: 130,
        w: 80,
        h: 5,
        material: "stone".into(),
    });
    s
}

#[test]
fn projectile_travels_and_deals_damage() {
    let mut s = setup();
    let player_pos = s.get_player().unwrap().pos;
    s.perform_action(&AiAction::Spawn {
        kind: "goblin".into(),
        x: player_pos[0] + 15.0,
        y: player_pos[1],
    });
    s.step(10);
    let goblin = s
        .get_entities()
        .into_iter()
        .find(|e| e.kind == "Goblin")
        .unwrap();
    let hp_before = goblin.health;
    s.perform_action(&AiAction::Shoot {
        dir_x: 1.0,
        dir_y: 0.0,
    });
    s.step(10);
    let goblin_after = s
        .get_entities()
        .into_iter()
        .find(|e| e.kind == "Goblin")
        .unwrap();
    assert!(
        goblin_after.health < hp_before,
        "goblin should take projectile damage: {} -> {}",
        hp_before,
        goblin_after.health
    );
}

#[test]
fn projectile_stops_on_solid_cell() {
    let mut s = setup();
    let player_pos = s.get_player().unwrap().pos;
    let wall_x = player_pos[0] as i32 + 8;
    s.perform_action(&AiAction::FillRect {
        x: wall_x,
        y: player_pos[1] as i32 - 2,
        w: 4,
        h: 4,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::Shoot {
        dir_x: 1.0,
        dir_y: 0.0,
    });
    s.step(10);
    let cell = s.get_cell(wall_x, player_pos[1] as i32);
    assert_eq!(
        cell.material, "stone",
        "projectile should not destroy stone wall"
    );
    let state = s.get_state();
    let projectile_count = state
        .entities
        .iter()
        .filter(|e| e.kind == "Projectile")
        .count();
    assert_eq!(
        projectile_count, 0,
        "projectile should be destroyed after hitting wall"
    );
}

#[test]
fn fireball_ignites_wood() {
    let mut s = setup();
    let player_pos = s.get_player().unwrap().pos;
    s.perform_action(&AiAction::FillRect {
        x: player_pos[0] as i32 + 5,
        y: player_pos[1] as i32 - 2,
        w: 6,
        h: 4,
        material: "wood".into(),
    });
    s.perform_action(&AiAction::ToggleFireball);
    s.perform_action(&AiAction::Shoot {
        dir_x: 1.0,
        dir_y: 0.0,
    });
    s.step(15);
    let fire_count = s.count_material_in_region(
        player_pos[0] as i32 + 4,
        player_pos[1] as i32 - 3,
        10,
        8,
        "fire",
    );
    assert!(
        fire_count > 0,
        "fireball should ignite wood, got {} fire cells",
        fire_count
    );
}

#[test]
fn corpse_decomposes_into_flesh_cells() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect {
        x: 100,
        y: 130,
        w: 40,
        h: 5,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::FillRect {
        x: 100,
        y: 100,
        w: 1,
        h: 30,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::FillRect {
        x: 139,
        y: 100,
        w: 1,
        h: 30,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::Spawn {
        kind: "goblin".into(),
        x: 110.0,
        y: 120.0,
    });
    s.step(10);
    let goblin = s
        .get_entities()
        .into_iter()
        .find(|e| e.kind == "Goblin")
        .unwrap();
    s.perform_action(&AiAction::DamageEntity {
        id: goblin.id,
        amount: 100.0,
    });
    s.step(100);
    let corpse = s
        .get_entities()
        .into_iter()
        .find(|e| e.kind == "Corpse")
        .unwrap();
    let _pos = corpse.pos;
    let after = s.count_material_in_region(100, 120, 40, 20, "flesh");
    assert!(
        after > 0,
        "corpse should decompose into flesh cells, got {}",
        after
    );
}

#[test]
fn ui_hud_shows_player_hp() {
    let mut s = setup();
    let mut ui = verbatim::ui::UiLayer::new();
    let player = s.game.entities.all()[0].clone();
    ui.draw_hud(
        80,
        25,
        Some(&player),
        s.tick(),
        verbatim::world::cell::MaterialId::Sand,
        0,
        0,
        1,
        &s.game.player,
        60.0,
    );
    assert!(ui.get(0, 8).is_some(), "HUD should draw HP label");
}
