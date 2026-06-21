use verbatim::ai::AiAction;
use verbatim::ai::GameSession;

fn setup() -> GameSession {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(90, 90, 50, 50);
    s
}

#[test]
fn fire_dies_over_time() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect {
        x: 100,
        y: 120,
        w: 10,
        h: 1,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::SetCell {
        x: 105,
        y: 119,
        material: "fire".into(),
    });
    s.step(60);
    assert_ne!(
        s.get_cell(105, 119).material,
        "fire",
        "fire should die after 60 ticks"
    );
}

#[test]
fn fire_ignites_wood() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect {
        x: 100,
        y: 120,
        w: 10,
        h: 1,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::SetCell {
        x: 104,
        y: 119,
        material: "wood".into(),
    });
    s.perform_action(&AiAction::SetCell {
        x: 105,
        y: 119,
        material: "fire".into(),
    });
    s.step(20);
    assert_ne!(
        s.get_cell(104, 119).material,
        "wood",
        "wood should be ignited by fire"
    );
}

#[test]
fn fire_ignites_grass() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect {
        x: 100,
        y: 120,
        w: 10,
        h: 1,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::FillRect {
        x: 100,
        y: 119,
        w: 8,
        h: 1,
        material: "grass".into(),
    });
    s.perform_action(&AiAction::SetCell {
        x: 100,
        y: 119,
        material: "fire".into(),
    });
    s.step(30);
    let grass_left = s.count_material_in_region(99, 118, 10, 3, "grass");
    assert_eq!(grass_left, 0, "fire should spread through grass");
}

#[test]
fn fire_ignites_flesh() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect {
        x: 100,
        y: 120,
        w: 10,
        h: 1,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::SetCell {
        x: 104,
        y: 119,
        material: "flesh".into(),
    });
    s.perform_action(&AiAction::SetCell {
        x: 105,
        y: 119,
        material: "fire".into(),
    });
    s.step(30);
    assert_ne!(
        s.get_cell(104, 119).material,
        "flesh",
        "flesh should be ignited by fire"
    );
}

#[test]
fn smoke_rises() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect {
        x: 100,
        y: 120,
        w: 10,
        h: 1,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::FillRect {
        x: 100,
        y: 110,
        w: 10,
        h: 1,
        material: "stone".into(),
    });
    for y in 114..118 {
        for x in 104..107 {
            s.perform_action(&AiAction::SetCell {
                x,
                y,
                material: "smoke".into(),
            });
        }
    }
    s.step(15);
    let smoke_above = s.count_material_in_region(100, 111, 10, 3, "smoke");
    assert!(smoke_above > 0, "smoke should rise toward ceiling");
}

#[test]
fn smoke_dissipates_over_time() {
    let mut s = setup();
    s.perform_action(&AiAction::SetCell {
        x: 105,
        y: 105,
        material: "smoke".into(),
    });
    s.step(120);
    let smoke_left = s.count_material_in_region(100, 100, 10, 10, "smoke");
    assert_eq!(smoke_left, 0, "smoke should dissipate after 120 ticks");
}

#[test]
fn steam_condenses_to_water() {
    let mut s = setup();
    // Closed container so steam/water cannot drift into inactive chunks.
    for y in 116..125 {
        s.perform_action(&AiAction::SetCell {
            x: 100,
            y,
            material: "stone".into(),
        });
        s.perform_action(&AiAction::SetCell {
            x: 109,
            y,
            material: "stone".into(),
        });
    }
    for x in 100..110 {
        s.perform_action(&AiAction::SetCell {
            x,
            y: 116,
            material: "stone".into(),
        });
        s.perform_action(&AiAction::SetCell {
            x,
            y: 124,
            material: "stone".into(),
        });
    }
    for y in 117..124 {
        for x in 101..109 {
            s.perform_action(&AiAction::SetCell {
                x,
                y,
                material: "steam".into(),
            });
        }
    }
    s.step(200);
    let water_or_steam = s.count_material_in_region(100, 116, 10, 9, "water")
        + s.count_material_in_region(100, 116, 10, 9, "steam");
    assert!(
        water_or_steam > 0,
        "steam should condense to water or remain steam: water+steam={}",
        water_or_steam
    );
    let water_count = s.count_material_in_region(100, 116, 10, 9, "water");
    assert!(
        water_count > 0,
        "some steam should have condensed to water by now: water={}",
        water_count
    );
}

#[test]
fn steam_rises() {
    let mut s = setup();
    s.perform_action(&AiAction::FillRect {
        x: 100,
        y: 130,
        w: 10,
        h: 1,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::FillRect {
        x: 100,
        y: 100,
        w: 10,
        h: 1,
        material: "stone".into(),
    });
    s.perform_action(&AiAction::SetCell {
        x: 105,
        y: 125,
        material: "steam".into(),
    });
    s.step(20);
    let steam_above = s.count_material_in_region(100, 105, 10, 10, "steam");
    assert!(steam_above > 0, "steam should rise upward");
}

#[test]
fn grass_is_solid() {
    let mut s = setup();
    s.perform_action(&AiAction::SetCell {
        x: 105,
        y: 110,
        material: "grass".into(),
    });
    let cell = s.get_cell(105, 110);
    assert!(cell.is_solid, "grass should be solid");
}

#[test]
fn dirt_is_solid() {
    let mut s = setup();
    s.perform_action(&AiAction::SetCell {
        x: 105,
        y: 110,
        material: "dirt".into(),
    });
    let cell = s.get_cell(105, 110);
    assert!(cell.is_solid, "dirt should be solid");
}

#[test]
fn stone_is_static() {
    let mut s = setup();
    s.perform_action(&AiAction::SetCell {
        x: 105,
        y: 110,
        material: "stone".into(),
    });
    s.step(20);
    assert_eq!(
        s.get_cell(105, 110).material,
        "stone",
        "stone should not move"
    );
}

#[test]
fn wood_is_static() {
    let mut s = setup();
    s.perform_action(&AiAction::SetCell {
        x: 105,
        y: 110,
        material: "wood".into(),
    });
    s.step(20);
    assert_eq!(
        s.get_cell(105, 110).material,
        "wood",
        "wood should not move"
    );
}

#[test]
fn bone_is_static() {
    let mut s = setup();
    s.perform_action(&AiAction::SetCell {
        x: 105,
        y: 110,
        material: "bone".into(),
    });
    s.step(20);
    assert_eq!(
        s.get_cell(105, 110).material,
        "bone",
        "bone should not move"
    );
}

#[test]
fn lava_initial_temp_is_high() {
    let mut s = setup();
    s.perform_action(&AiAction::SetCell {
        x: 105,
        y: 110,
        material: "lava".into(),
    });
    let cell = s.get_cell(105, 110);
    assert!(
        cell.temp > 1000.0,
        "lava should start very hot, got {}°C",
        cell.temp
    );
}

#[test]
fn water_initial_temp_is_room() {
    let mut s = setup();
    s.perform_action(&AiAction::SetCell {
        x: 105,
        y: 110,
        material: "water".into(),
    });
    let cell = s.get_cell(105, 110);
    assert!(
        cell.temp < 50.0,
        "water should start at room temp, got {}°C",
        cell.temp
    );
}
