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

    pub fn display_glyph(&self) -> [char; 2] {
        match self.typ {
            ItemType::Dagger => ['/', ')'],
            ItemType::Sword => ['|', '|'],
            ItemType::Bow => [')', '{'],
            ItemType::LeatherArmor => ['[', 'L'],
            ItemType::PlateArmor => ['{', 'P'],
            ItemType::Shield => ['O', 'S'],
            ItemType::HealthPotion => ['!', 'H'],
            ItemType::ManaPotion => ['!', 'M'],
            ItemType::Food => ['%', 'F'],
            ItemType::Scroll => ['?', 'S'],
        }
    }

    pub fn display_string(&self) -> String {
        let [a, b] = self.display_glyph();
        format!("{}{}", a, b)
    }

    pub fn shape(&self) -> Vec<(i32, i32, [u8; 3])> {
        match self.typ {
            ItemType::Dagger => vec![
                (0, 0, [220, 220, 230]),
                (-1, 1, [120, 80, 40]),
                (0, 1, [120, 80, 40]),
            ],
            ItemType::Sword => vec![
                (0, -2, [235, 235, 245]),
                (0, -1, [240, 240, 250]),
                (-1, 0, [180, 130, 60]),
                (1, 0, [180, 130, 60]),
                (0, 0, [160, 110, 50]),
                (0, 1, [100, 70, 30]),
            ],
            ItemType::Bow => vec![
                (0, -1, [170, 130, 70]),
                (1, 0, [180, 140, 80]),
                (0, 1, [170, 130, 70]),
                (-1, 0, [220, 220, 220]),
            ],
            ItemType::LeatherArmor => vec![
                (-1, -1, [150, 100, 55]),
                (0, -1, [160, 110, 60]),
                (-1, 0, [140, 90, 50]),
                (0, 0, [130, 80, 45]),
            ],
            ItemType::PlateArmor => vec![
                (-1, -1, [190, 195, 205]),
                (0, -1, [200, 205, 215]),
                (-1, 0, [180, 185, 195]),
                (0, 0, [170, 175, 185]),
            ],
            ItemType::Shield => vec![
                (-1, -1, [170, 150, 70]),
                (0, -1, [180, 160, 80]),
                (-1, 0, [160, 140, 60]),
                (0, 0, [150, 130, 50]),
            ],
            ItemType::HealthPotion => vec![
                (0, -1, [60, 40, 30]),
                (0, 0, [220, 30, 30]),
                (-1, 0, [180, 20, 20]),
                (1, 0, [200, 25, 25]),
            ],
            ItemType::ManaPotion => vec![
                (0, -1, [60, 40, 30]),
                (0, 0, [30, 30, 220]),
                (-1, 0, [20, 20, 180]),
                (1, 0, [25, 25, 200]),
            ],
            ItemType::Food => vec![
                (0, 0, [80, 200, 60]),
                (-1, 0, [60, 180, 40]),
                (1, 0, [70, 190, 50]),
            ],
            ItemType::Scroll => vec![
                (-1, 0, [240, 220, 120]),
                (0, 0, [250, 230, 130]),
                (1, 0, [240, 220, 120]),
            ],
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
