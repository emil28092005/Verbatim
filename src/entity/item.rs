#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ItemType {
    Dagger,
    Sword,
    Bow,
    LeatherArmor,
    PlateArmor,
    HealthPotion,
    ManaPotion,
    Food,
    Scroll,
    Shield,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub typ: ItemType,
    pub x: i32,
    pub y: i32,
}

impl Item {
    pub fn new(typ: ItemType, x: i32, y: i32) -> Self {
        Self { typ, x, y }
    }

    pub fn name(&self) -> &'static str {
        match self.typ {
            ItemType::Dagger => "Dagger",
            ItemType::Sword => "Sword",
            ItemType::Bow => "Bow",
            ItemType::LeatherArmor => "Leather Armor",
            ItemType::PlateArmor => "Plate Armor",
            ItemType::Shield => "Shield",
            ItemType::HealthPotion => "Health Potion",
            ItemType::ManaPotion => "Mana Potion",
            ItemType::Food => "Food",
            ItemType::Scroll => "Scroll",
        }
    }

    pub fn display_char(&self) -> char {
        match self.typ {
            ItemType::Dagger => '/',
            ItemType::Sword => '|',
            ItemType::Bow => ')',
            ItemType::LeatherArmor => '[',
            ItemType::PlateArmor => '{',
            ItemType::Shield => 'O',
            ItemType::HealthPotion => '!',
            ItemType::ManaPotion => '!',
            ItemType::Food => '%',
            ItemType::Scroll => '?',
        }
    }

    pub fn color(&self) -> [u8; 3] {
        match self.typ {
            ItemType::Dagger => [200, 200, 200],
            ItemType::Sword => [240, 240, 240],
            ItemType::Bow => [180, 140, 80],
            ItemType::LeatherArmor => [140, 90, 50],
            ItemType::PlateArmor => [180, 180, 190],
            ItemType::Shield => [160, 140, 60],
            ItemType::HealthPotion => [255, 40, 40],
            ItemType::ManaPotion => [60, 60, 255],
            ItemType::Food => [80, 200, 60],
            ItemType::Scroll => [255, 220, 120],
        }
    }

    pub fn is_equipment(&self) -> bool {
        matches!(
            self.typ,
            ItemType::Dagger
                | ItemType::Sword
                | ItemType::Bow
                | ItemType::LeatherArmor
                | ItemType::PlateArmor
                | ItemType::Shield
        )
    }

    pub fn is_weapon(&self) -> bool {
        matches!(self.typ, ItemType::Dagger | ItemType::Sword | ItemType::Bow)
    }

    pub fn is_armor(&self) -> bool {
        matches!(
            self.typ,
            ItemType::LeatherArmor | ItemType::PlateArmor | ItemType::Shield
        )
    }

    pub fn is_consumable(&self) -> bool {
        matches!(
            self.typ,
            ItemType::HealthPotion | ItemType::ManaPotion | ItemType::Food | ItemType::Scroll
        )
    }

    pub fn damage_bonus(&self) -> f32 {
        match self.typ {
            ItemType::Dagger => 3.0,
            ItemType::Sword => 6.0,
            ItemType::Bow => 4.0,
            _ => 0.0,
        }
    }

    pub fn armor_bonus(&self) -> f32 {
        match self.typ {
            ItemType::LeatherArmor => 2.0,
            ItemType::PlateArmor => 5.0,
            ItemType::Shield => 3.0,
            _ => 0.0,
        }
    }

    pub fn heal_amount(&self) -> f32 {
        match self.typ {
            ItemType::HealthPotion => 40.0,
            ItemType::ManaPotion => 20.0,
            ItemType::Food => 15.0,
            _ => 0.0,
        }
    }
}

pub struct ItemManager {
    items: Vec<Item>,
    next_id: u32,
}

impl ItemManager {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            next_id: 0,
        }
    }

    pub fn spawn(&mut self, typ: ItemType, x: i32, y: i32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.items.push(Item::new(typ, x, y));
        id
    }

    pub fn all(&self) -> &[Item] {
        &self.items
    }

    pub fn all_mut(&mut self) -> &mut Vec<Item> {
        &mut self.items
    }

    pub fn remove_at(&mut self, x: i32, y: i32) -> Option<Item> {
        if let Some(idx) = self.items.iter().position(|i| i.x == x && i.y == y) {
            Some(self.items.remove(idx))
        } else {
            None
        }
    }
}
