use verbatim::ai::AiAction;
use verbatim::ai::GameSession;

fn setup() -> GameSession {
    let mut s = GameSession::new_seeded(42);
    s.init();
    s
}

#[test]
fn descend_requires_stairs() {
    let mut s = setup();
    let start_depth = s.game.depth;
    s.perform_action(&AiAction::Descend);
    assert_eq!(
        s.game.depth, start_depth,
        "descend should not work without stairs"
    );
}

#[test]
fn descend_increases_depth_on_stairs() {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(90, 90, 50, 50);
    let player = s.game.player.center(&s.game.entities);
    let foot_x = player.0 as i32;
    let foot_y = (player.1 + 3.0) as i32;
    s.perform_action(&AiAction::SetCell {
        x: foot_x,
        y: foot_y,
        material: "stairs".into(),
    });
    s.perform_action(&AiAction::SetCell {
        x: foot_x,
        y: foot_y + 1,
        material: "stone".into(),
    });
    s.step(5);
    let before = s.game.depth;
    s.perform_action(&AiAction::Descend);
    assert_eq!(
        s.game.depth,
        before + 1,
        "descend should increase depth when on stairs"
    );
}

#[test]
fn depth_shown_in_hud() {
    let s = setup();
    let player = s.game.entities.all()[0].clone();
    let mut ui = verbatim::ui::UiLayer::new();
    let screen_w = 320;
    let screen_h = 100;
    ui.draw_hud(
        screen_w,
        screen_h,
        Some(&player),
        s.game.tick,
        verbatim::world::cell::MaterialId::Sand,
        0,
        0,
        s.game.depth,
        &s.game.player,
        60.0,
    );
    let stats = format!(
        "LV:{} XP:{} K:{} S:{} D:{} T:{}",
        player.level, player.xp, 0, 0, s.game.depth, s.game.tick
    );
    let stats_w = verbatim::ui::UiLayer::text_width(&stats);
    let stats_x = (screen_w as i32 - stats_w as i32).max(0);
    let y_row3 = screen_h as i32 - 28 + 22;
    let line: String = (stats_x as i32..screen_w as i32)
        .step_by(3)
        .map(|x| ui.get(x, y_row3).map(|c| c.ch).unwrap_or(' '))
        .collect();
    assert!(line.contains("D:1"), "HUD should show depth: {}", line);
}

#[test]
fn stairs_material_exists() {
    let mut s = setup();
    let cell = s.get_cell(0, 0);
    assert_ne!(cell.material, "stairs", "empty corner should not be stairs");
    s.perform_action(&AiAction::SetCell {
        x: 50,
        y: 50,
        material: "stairs".into(),
    });
    let cell = s.get_cell(50, 50);
    assert_eq!(
        cell.material, "stairs",
        "stairs material should be placeable"
    );
}

#[test]
fn cell_stairs_bytes_roundtrip() {
    let cell = verbatim::world::cell::Cell::new(verbatim::world::cell::MaterialId::Stairs);
    let bytes = cell.to_bytes();
    let cell2 = verbatim::world::cell::Cell::from_bytes(&bytes);
    assert_eq!(cell2.material, verbatim::world::cell::MaterialId::Stairs);
}

#[test]
fn material_name_stairs() {
    let mut s = setup();
    let cell = s.get_cell(50, 50);
    assert_eq!(cell.material, "empty");
    s.perform_action(&AiAction::SetCell {
        x: 50,
        y: 50,
        material: "stairs".into(),
    });
    let cell = s.get_cell(50, 50);
    assert_eq!(cell.material, "stairs");
}

#[test]
fn descend_resets_world() {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(90, 90, 50, 50);
    let player = s.game.player.center(&s.game.entities);
    let foot_x = player.0 as i32;
    let foot_y = (player.1 + 3.0) as i32;
    s.perform_action(&AiAction::SetCell {
        x: foot_x,
        y: foot_y,
        material: "stairs".into(),
    });
    s.perform_action(&AiAction::SetCell {
        x: foot_x,
        y: foot_y + 1,
        material: "stone".into(),
    });
    s.step(5);
    let before = s.game.depth;
    s.perform_action(&AiAction::Descend);
    assert_eq!(s.game.depth, before + 1);
    assert!(
        s.game.player.entity(&s.game.entities).is_some(),
        "player should respawn"
    );
    assert!(
        s.game
            .entities
            .all()
            .iter()
            .all(|e| e.alive || e.kind == verbatim::entity::EntityKind::Corpse),
        "old corpses should be gone"
    );
}
