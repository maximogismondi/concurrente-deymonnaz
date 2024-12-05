use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Represents a coordinate in the ConcuRide system.
#[derive(Debug, Deserialize, Clone, PartialEq, Default, Serialize)]
pub struct Coordinate {
    /// The x coordinate.
    pub x: i32,
    /// The y coordinate.
    pub y: i32,
}

impl Coordinate {
    /// Creates a new `Coordinate` instance.
    pub fn new(x: i32, y: i32) -> Coordinate {
        Coordinate { x, y }
    }

    /// Calculates the distance between two coordinates.
    pub fn distance(&self, other: &Coordinate) -> usize {
        ((self.x - other.x).abs() + (self.y - other.y).abs()) as usize
    }
}

impl FromStr for Coordinate {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(';').collect();
        if parts.len() != 2 {
            return Err("Invalid coordinate format".to_string());
        }

        let x;
        let y;

        if let Ok(x_val) = parts[0].parse::<i32>() {
            x = x_val;
        } else {
            return Err("Invalid coordinate format".to_string());
        }

        if let Ok(y_val) = parts[1].parse::<i32>() {
            y = y_val;
        } else {
            return Err("Invalid coordinate format".to_string());
        }

        Ok(Coordinate { x, y })
    }
}

impl std::fmt::Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{};{}", self.x, self.y)
    }
}
