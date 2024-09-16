pub fn calculate_percentage(count: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (count as f64 / total as f64 * 10000f64).round() / 100f64
    }
}

pub fn calculate_average(distance: f64, count: usize) -> f64 {
    if count == 0 {
        0.0
    } else {
        (distance / count as f64 * 100f64).round() / 100f64
    }
}

#[cfg(test)]
mod tests {
    use crate::float_calculations::{calculate_average, calculate_percentage};

    #[test]
    fn test_calculate_percentage() {
        assert_eq!(calculate_percentage(0, 0), 0.0);
        assert_eq!(calculate_percentage(0, 1), 0.0);
        assert_eq!(calculate_percentage(1, 1), 100.0);
        assert_eq!(calculate_percentage(1, 3), 33.33);
    }

    #[test]
    fn test_calculate_average() {
        assert_eq!(calculate_average(0.0, 0), 0.0);
        assert_eq!(calculate_average(0.0, 1), 0.0);
        assert_eq!(calculate_average(1.0, 1), 1.0);
        assert_eq!(calculate_average(1.0, 3), 0.33);
    }
}
