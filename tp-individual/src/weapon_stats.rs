pub struct WeaponStats {
    pub count: usize,
    pub total_distance: f32,
}

impl WeaponStats {
    pub fn new(distance: f32) -> Self {
        Self {
            count: 1,
            total_distance: distance,
        }
    }

    pub fn add_death(&mut self, distance: f32) {
        self.count += 1;
        self.total_distance += distance;
    }
}
