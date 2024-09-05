use std::collections::HashMap;

pub struct PlayerStats {
    pub total: usize,
    pub weapons: HashMap<String, usize>,
}

impl PlayerStats {
    pub fn new(weapon: &String) -> Self {
        Self {
            total: 1,
            weapons: vec![(weapon.to_string(), 1)].into_iter().collect(),
        }
    }

    pub fn add_death(&mut self, weapon: &String) {
        self.total += 1;
        if let Some(count) = self.weapons.get_mut(weapon) {
            *count += 1;
        } else {
            self.weapons.insert(weapon.to_string(), 1);
        }
    }
}
