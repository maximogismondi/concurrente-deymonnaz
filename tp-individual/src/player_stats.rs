use std::collections::HashMap;

use crate::deaths::Death;

pub type PlayerWeaponStats = HashMap<String, usize>;

pub struct PlayerStats {
    pub total: usize,
    pub weapons: PlayerWeaponStats,
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

pub fn player_stats_from_deaths(deaths: &Vec<Death>) -> HashMap<String, PlayerStats> {
    deaths.iter().fold(HashMap::new(), |mut acc, death| {
        if let Some(stats) = acc.get_mut(&death.killer_name) {
            stats.add_death(&death.killed_by);
        } else {
            acc.insert(
                death.killer_name.clone(),
                PlayerStats::new(&death.killed_by),
            );
        }

        acc
    })
}

pub fn get_top_killers(
    player_stats: HashMap<String, PlayerStats>,
    player_count: usize,
    weapon_count: usize,
) -> HashMap<String, PlayerStats> {
    let mut player_names: Vec<&String> = player_stats.keys().collect();

    player_names.sort_by(|a, b| {
        let a = player_stats.get(*a).unwrap().total;
        let b = player_stats.get(*b).unwrap().total;
        b.cmp(&a)
    });

    let most_lethal_players: HashMap<String, PlayerStats> = player_names
        .iter()
        .take(player_count)
        .map(|name| {
            let stats = player_stats.get(*name).unwrap();
            let total = stats.total;

            let weapons: PlayerWeaponStats = get_top_weapons(stats.weapons.clone(), weapon_count);

            (name.to_string(), PlayerStats { total, weapons })
        })
        .collect();

    most_lethal_players
}

fn get_top_weapons(weapons_stats: PlayerWeaponStats, weapon_count: usize) -> PlayerWeaponStats {
    let mut weapon_names: Vec<&String> = weapons_stats.keys().collect();

    weapon_names.sort_by(|a, b| {
        let a = weapons_stats.get(*a).unwrap();
        let b = weapons_stats.get(*b).unwrap();
        b.cmp(&a)
    });

    let most_lethal_weapons: PlayerWeaponStats = weapon_names
        .iter()
        .take(weapon_count)
        .map(|weapon| (weapon.to_string(), *weapons_stats.get(*weapon).unwrap()))
        .collect();

    most_lethal_weapons
}
