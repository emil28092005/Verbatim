use verbatim::ai::AiAction;
use verbatim::ai::GameSession;
use verbatim::entity::{ItemManager, ItemType};

fn setup() -> GameSession {
    let mut s = GameSession::new_seeded(42);
    s.init_empty();
    s.clear_area(90, 90, 50, 50);
    s
}

#[test]
fn item_picked_up_when_near_player() {
    let mut s = setup();
    let pos = s.get_player().unwrap().pos;
    let x = pos[0] as i32;
    let y = pos[1] as i32;
    s.game.items.spawn(ItemType::Sword, x, y);
    assert_eq!(s.game.player.inventory.len(), 0);
    s.step(1);
    assert_eq!(s.game.player.inventory.len(), 1);
    assert_eq!(s.game.player.inventory[0].typ, ItemType::Sword);
}

#[test]
fn equipping_weapon_adds_damage_bonus() {
    let mut s = setup();
    let pos = s.get_player().unwrap().pos;
    s.game
        .items
        .spawn(ItemType::Sword, pos[0] as i32, pos[1] as i32);
    s.step(1);
    s.game.use_item(0);
    assert_eq!(s.game.player.weapon.as_ref().unwrap().typ, ItemType::Sword);
    let bonus = s.game.player.weapon.as_ref().unwrap().damage_bonus();
    assert!(bonus > 0.0, "equipped weapon should provide damage bonus");
}

#[test]
fn equipping_armor_reduces_contact_damage() {
    let mut s = setup();
    s.perform_action(&AiAction::Spawn {
        kind: "goblin".into(),
        x: 120.0,
        y: 120.0,
    });
    let pos = s.get_player().unwrap().pos;
    s.game
        .items
        .spawn(ItemType::PlateArmor, pos[0] as i32, pos[1] as i32);
    s.step(1);
    s.game.use_item(0);
    let armor = s.game.player.armor.as_ref().unwrap().armor_bonus();
    assert!(armor > 0.0, "plate armor should provide armor bonus");
}

#[test]
fn consumable_heals_player() {
    let mut s = setup();
    let pos = s.get_player().unwrap().pos;
    s.game
        .items
        .spawn(ItemType::HealthPotion, pos[0] as i32, pos[1] as i32);
    s.step(1);
    let id = s.get_player().unwrap().id;
    s.perform_action(&AiAction::DamageEntity { id, amount: 50.0 });
    let health_before = s.get_player().unwrap().health;
    s.game.use_item(0);
    let health_after = s.get_player().unwrap().health;
    assert!(health_after > health_before, "health potion should heal");
}

#[test]
fn dropped_item_returns_to_world() {
    let mut s = setup();
    let pos = s.get_player().unwrap().pos;
    s.game
        .items
        .spawn(ItemType::Dagger, pos[0] as i32, pos[1] as i32);
    s.step(1);
    s.game.drop_item(0);
    assert_eq!(s.game.player.inventory.len(), 0);
    let count = s
        .game
        .items
        .all()
        .iter()
        .filter(|i| i.typ == ItemType::Dagger)
        .count();
    assert_eq!(count, 1, "dropped item should exist in world");
}

#[test]
fn item_manager_spawns_and_removes() {
    let mut mgr = ItemManager::new();
    let id = mgr.spawn(ItemType::Food, 100, 100);
    assert_eq!(id, 0);
    assert_eq!(mgr.all().len(), 1);
    let removed = mgr.remove_at(100, 100);
    assert!(removed.is_some());
    assert_eq!(mgr.all().len(), 0);
}
