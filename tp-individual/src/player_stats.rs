use std::collections::HashMap;

pub type PlayerWeaponStats = HashMap<String, usize>;

#[derive(Eq, PartialEq)]
pub struct PlayerStats {
    pub deaths_count: usize,
    pub weapons: PlayerWeaponStats,
}

impl PartialOrd for PlayerStats {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.deaths_count.cmp(&other.deaths_count))
    }
}

impl Ord for PlayerStats {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.deaths_count.cmp(&other.deaths_count)
    }
}

impl PlayerStats {
    pub fn new() -> Self {
        Self {
            deaths_count: 0,
            weapons: HashMap::new(),
        }
    }

    pub fn add_death(&mut self, weapon: Option<String>) {
        self.deaths_count += 1;
        if let Some(weapon) = weapon {
            *self.weapons.entry(weapon).or_insert(0) += 1;
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.deaths_count += other.deaths_count;
        for (weapon, count) in other.weapons.into_iter() {
            *self.weapons.entry(weapon).or_insert(0) += count;
        }
    }
}
