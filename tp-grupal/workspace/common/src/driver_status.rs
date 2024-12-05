use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::ride::Ride;

/// Represents the status of a driver in the ConcuRide system.
///
/// # Variants
///
/// - `Free`: The driver is free.
/// - `AcceptingRide`: The driver is accepting a ride.
/// - `Busy`: The driver is busy with a ride.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriverStatus {
    Free,
    AcceptingRide(Ride),
    Busy(Ride),
}

impl DriverStatus {
    /// Checks if the driver is free.
    pub fn is_free(&self) -> bool {
        matches!(self, DriverStatus::Free)
    }
}

impl fmt::Display for DriverStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DriverStatus::Free => write!(f, "Free"),
            DriverStatus::AcceptingRide(ride) => write!(f, "AcceptingRide {}", ride),
            DriverStatus::Busy(ride) => write!(f, "Busy {}", ride),
        }
    }
}

impl FromStr for DriverStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        match parts[0] {
            "Free" => Ok(DriverStatus::Free),
            "AcceptingRide" => {
                let ride = parts[1..].join(" ");
                let ride = ride.parse::<Ride>().unwrap_or(Ride::default());
                Ok(DriverStatus::AcceptingRide(ride))
            }
            "Busy" => {
                let ride = parts[1..].join(" ");
                let ride = ride.parse::<Ride>().unwrap_or(Ride::default());
                Ok(DriverStatus::Busy(ride))
            }
            _ => Err("Invalid DriverStatus".to_string()),
        }
    }
}
