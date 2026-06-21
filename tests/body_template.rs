use verbatim::entity::body_template::{BodyPart, BodyTemplate};

#[test]
fn player_template_has_correct_shape() {
    let t = BodyTemplate::humanoid_player();
    assert_eq!(t.name, "player");
    assert!(
        t.parts.len() >= 30,
        "player should have 30+ parts, got {}",
        t.parts.len()
    );
    assert_eq!(t.half_w, 4.0);
    assert_eq!(t.half_h, 6.0);
}

#[test]
fn goblin_template_has_correct_shape() {
    let t = BodyTemplate::humanoid_goblin();
    assert_eq!(t.name, "goblin");
    assert!(
        t.parts.len() >= 30,
        "goblin should have 30+ parts, got {}",
        t.parts.len()
    );
}

#[test]
fn boulder_template_has_correct_shape() {
    let t = BodyTemplate::boulder();
    assert_eq!(t.name, "boulder");
    assert_eq!(t.parts.len(), 16);
    assert!(t.parts.iter().all(|p| p.label == "rock"));
}

#[test]
fn template_json_roundtrip() {
    let t = BodyTemplate::humanoid_player();
    let json = t.to_json();
    let t2 = BodyTemplate::from_json(&json).expect("parse");
    assert_eq!(t2.name, t.name);
    assert_eq!(t2.parts.len(), t.parts.len());
    assert_eq!(t2.half_w, t.half_w);
    assert_eq!(t2.half_h, t.half_h);
}

#[test]
fn preview_ascii_not_empty() {
    let t = BodyTemplate::humanoid_player();
    let preview = t.preview_ascii();
    assert!(!preview.is_empty());
    assert!(preview.contains("O"));
}

#[test]
fn custom_template_works() {
    let t = BodyTemplate {
        name: "snake".to_string(),
        half_w: 3.0,
        half_h: 0.5,
        radius: 0.5,
        parts: vec![
            BodyPart {
                x: -3.0,
                y: 0.0,
                color: [100, 200, 50, 255],
                label: "body".into(),
            },
            BodyPart {
                x: -2.0,
                y: 0.0,
                color: [100, 200, 50, 255],
                label: "body".into(),
            },
            BodyPart {
                x: -1.0,
                y: 0.0,
                color: [100, 200, 50, 255],
                label: "body".into(),
            },
            BodyPart {
                x: 0.0,
                y: 0.0,
                color: [100, 200, 50, 255],
                label: "body".into(),
            },
            BodyPart {
                x: 1.0,
                y: 0.0,
                color: [100, 200, 50, 255],
                label: "body".into(),
            },
            BodyPart {
                x: 2.0,
                y: 0.0,
                color: [100, 200, 50, 255],
                label: "body".into(),
            },
            BodyPart {
                x: 3.0,
                y: 0.0,
                color: [80, 180, 40, 255],
                label: "head".into(),
            },
        ],
        constraints: vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6)],
    };
    assert_eq!(t.parts.len(), 7);
    let json = t.to_json();
    let t2 = BodyTemplate::from_json(&json).expect("parse");
    assert_eq!(t2.parts.len(), 7);
    assert_eq!(t2.name, "snake");
}

#[test]
fn template_apply_to_entity() {
    use verbatim::entity::entity::{Entity, EntityKind};
    let mut e = Entity::new(0, EntityKind::Goblin);
    let t = BodyTemplate::humanoid_goblin();
    t.apply_to(&mut e, 100.0, 100.0);
    assert!(
        e.bodies.len() >= 30,
        "goblin should have 30+ bodies, got {}",
        e.bodies.len()
    );
    assert_eq!(e.cx, 100.0);
    assert_eq!(e.cy, 100.0);
    assert!(!e.constraints.is_empty());
}
