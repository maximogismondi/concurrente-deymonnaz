use crate::float_calculations::{calculate_average, calculate_percentage};

/// Struct to store the stats of a weapon.
pub struct WeaponStats {
    death_count: usize,
    death_count_with_distance: usize,
    total_distance: f64,
}

impl Eq for WeaponStats {}

impl PartialEq for WeaponStats {
    fn eq(&self, other: &Self) -> bool {
        self.death_count == other.death_count
    }
}

impl PartialOrd for WeaponStats {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WeaponStats {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.death_count.cmp(&other.death_count)
    }
}

impl WeaponStats {
    /// Creates a new `WeaponStats` instance.
    pub fn new() -> Self {
        Self {
            death_count: 0,
            death_count_with_distance: 0,
            total_distance: 0.0,
        }
    }

    /// Adds a death count to the weapon stats.
    /// If the distance is provided, it also increments the death count with distance and adds the distance to the total distance for further calculations.
    pub fn add_death(&mut self, distance: Option<f64>) {
        self.death_count += 1;
        if let Some(distance) = distance {
            self.death_count_with_distance += 1;
            self.total_distance += distance;
        }
    }

    /// Merges the stats of another `WeaponStats` instance into this one.
    pub fn merge(&mut self, other: &Self) {
        self.death_count += other.death_count;
        self.death_count_with_distance += other.death_count_with_distance;
        self.total_distance += other.total_distance;
    }

    /// Returns the stats of the weapon in a JSON format.
    pub fn json_display(&self, total_deaths: usize) -> serde_json::Value {
        serde_json::json!({
            "deaths_percentage": calculate_percentage(self.death_count, total_deaths),
            "average_distance": calculate_average(self.total_distance, self.death_count_with_distance),
        })
    }
}

#[cfg(test)]
mod tests {
    use assert_json_diff::assert_json_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_new() {
        let weapon_stats = WeaponStats::new();

        assert_eq!(weapon_stats.death_count, 0);
        assert_eq!(weapon_stats.death_count_with_distance, 0);
        assert_eq!(weapon_stats.total_distance, 0.0);
    }

    #[test]
    fn test_add_death() {
        let mut weapon_stats = WeaponStats::new();

        weapon_stats.add_death(Some(100.0));
        assert_eq!(weapon_stats.death_count, 1);
        assert_eq!(weapon_stats.death_count_with_distance, 1);
        assert_eq!(weapon_stats.total_distance, 100.0);
    }

    #[test]
    fn test_add_multiple_deaths() {
        let mut weapon_stats = WeaponStats::new();

        weapon_stats.add_death(Some(100.0));
        weapon_stats.add_death(Some(200.0));

        assert_eq!(weapon_stats.death_count, 2);
        assert_eq!(weapon_stats.death_count_with_distance, 2);
        assert_eq!(weapon_stats.total_distance, 300.0);
    }

    #[test]
    fn test_add_no_distance_death() {
        let mut weapon_stats = WeaponStats::new();

        weapon_stats.add_death(None);

        assert_eq!(weapon_stats.death_count, 1);
        assert_eq!(weapon_stats.death_count_with_distance, 0);
        assert_eq!(weapon_stats.total_distance, 0.0);
    }

    #[test]
    fn test_merge() {
        let mut weapon_stats_1 = WeaponStats::new();
        let mut weapon_stats_2 = WeaponStats::new();

        weapon_stats_1.add_death(Some(100.0));
        weapon_stats_1.add_death(None);
        weapon_stats_2.add_death(Some(200.0));

        weapon_stats_1.merge(&weapon_stats_2);

        assert_eq!(weapon_stats_1.death_count, 3);
        assert_eq!(weapon_stats_1.death_count_with_distance, 2);
        assert_eq!(weapon_stats_1.total_distance, 300.0);
    }

    #[test]
    fn test_json_display() {
        let mut weapon_stats = WeaponStats::new();

        weapon_stats.add_death(Some(100.0));
        weapon_stats.add_death(Some(200.0));

        let json = weapon_stats.json_display(2);

        let expected_json = json!({
            "deaths_percentage": 100.0,
            "average_distance": 150.0,
        });

        assert_json_eq!(expected_json, json);
    }
}
