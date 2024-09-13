pub struct WeaponStats {
    pub death_count: usize,
    pub death_count_with_distance: usize,
    pub total_distance: f64,
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
    pub fn new() -> Self {
        Self {
            death_count: 0,
            death_count_with_distance: 0,
            total_distance: 0.0,
        }
    }

    pub fn add_death(&mut self, distance: Option<f64>) {
        self.death_count += 1;
        if let Some(distance) = distance {
            self.death_count_with_distance += 1;
            self.total_distance += distance;
        }
    }

    pub fn merge(&mut self, other: &Self) {
        self.death_count += other.death_count;
        self.death_count_with_distance += other.death_count_with_distance;
        self.total_distance += other.total_distance;
    }
}

#[cfg(test)]
mod tests {
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
}
