use std::collections::HashMap;

pub type PlayerWeaponStats = HashMap<String, usize>;

#[derive(Eq, PartialEq)]
pub struct PlayerStats {
    pub deaths_count: usize,
    pub weapons: PlayerWeaponStats,
}

impl PartialOrd for PlayerStats {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
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

#[cfg(test)]
mod tests {
    use super::*;

    const WEAPON_1: &str = "AK47";
    const WEAPON_2: &str = "M4A4";

    #[test]
    fn test_new() {
        let player_stats = PlayerStats::new();

        assert_eq!(player_stats.deaths_count, 0);
        assert!(player_stats.weapons.is_empty());
    }

    #[test]
    fn test_add_death() {
        let mut player_stats = PlayerStats::new();

        player_stats.add_death(Some(WEAPON_1.to_string()));

        assert_eq!(player_stats.deaths_count, 1);
        assert_eq!(player_stats.weapons.len(), 1);
    }

    #[test]
    fn test_add_two_deaths_same_weapon() {
        let mut player_stats = PlayerStats::new();

        player_stats.add_death(Some(WEAPON_1.to_string()));
        player_stats.add_death(Some(WEAPON_1.to_string()));

        assert_eq!(player_stats.deaths_count, 2);
        assert_eq!(player_stats.weapons.len(), 1);
        assert_eq!(player_stats.weapons.get(WEAPON_1), Some(&2));
    }

    #[test]
    fn test_add_two_deaths_different_weapons() {
        let mut player_stats = PlayerStats::new();

        player_stats.add_death(Some(WEAPON_1.to_string()));
        player_stats.add_death(Some(WEAPON_2.to_string()));

        assert_eq!(player_stats.deaths_count, 2);
        assert_eq!(player_stats.weapons.len(), 2);
        assert_eq!(player_stats.weapons.get(WEAPON_1), Some(&1));
        assert_eq!(player_stats.weapons.get(WEAPON_2), Some(&1));
    }

    #[test]
    fn test_add_death_no_weapon() {
        let mut player_stats = PlayerStats::new();

        player_stats.add_death(None);

        assert_eq!(player_stats.deaths_count, 1);
        assert!(player_stats.weapons.is_empty());
    }

    #[test]
    fn test_merge() {
        let mut player_stats_1 = PlayerStats::new();
        player_stats_1.add_death(Some(WEAPON_1.to_string()));
        player_stats_1.add_death(Some(WEAPON_2.to_string()));

        let mut player_stats_2 = PlayerStats::new();
        player_stats_2.add_death(Some(WEAPON_1.to_string()));
        player_stats_2.add_death(Some(WEAPON_1.to_string()));

        player_stats_1.merge(player_stats_2);

        assert_eq!(player_stats_1.deaths_count, 4);
        assert_eq!(player_stats_1.weapons.len(), 2);
        assert_eq!(player_stats_1.weapons.get(WEAPON_1), Some(&3));
        assert_eq!(player_stats_1.weapons.get(WEAPON_2), Some(&1));
    }
}
