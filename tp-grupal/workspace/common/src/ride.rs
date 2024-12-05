use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{coordinate::Coordinate, Id};

/// Represents a ride in the ConcuRide system.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Ride {
    /// The identifier of the passenger.
    pub passenger_id: Id,
    /// The starting coordinate of the ride.
    pub from: Coordinate,
    /// The destination coordinate of the ride.
    pub to: Coordinate,
    /// The status of the ride.
    pub status: RideStatus,
    /// The identifier of the driver assigned to the ride.
    pub driver: Option<Id>,
}

/// Represents the various statuses a ride can have.
///
/// # Variants
///
/// - `Requested`: The ride has been requested.
/// - `LookingForDriver`: Looking for a driver to do the drive.
/// - `GoingToPickup`: The driver is going to the pickup location.
/// - `WaitingToStart`: The driver is waiting for the ride to start.
/// - `InProgress`: The ride is in progress.
/// - `Completed`: The ride has been completed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RideStatus {
    Requested,
    LookingForDriver,
    GoingToPickup,
    WaitingToStart,
    InProgress,
    Completed,
}

impl Ride {
    /// Creates a new `Ride` instance.
    pub fn new(id_passenger: Id, from: Coordinate, to: Coordinate) -> Ride {
        Self {
            passenger_id: id_passenger,
            from,
            to,
            driver: None,
            status: RideStatus::Requested,
        }
    }

    /// Assigns a driver to the ride.
    pub fn assign_driver(&mut self, driver: Id) {
        self.driver = Some(driver);
    }
}

impl Default for Ride {
    fn default() -> Self {
        Self {
            passenger_id: 0,
            from: Coordinate::new(0, 0),
            to: Coordinate::new(0, 0),
            driver: None,
            status: RideStatus::Requested,
        }
    }
}

impl std::fmt::Display for Ride {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.driver {
            Some(driver) => write!(
                f,
                "{} {} {} {} {}",
                self.passenger_id, self.from, self.to, self.status, driver
            ),
            None => write!(
                f,
                "{} {} {} {}",
                self.passenger_id, self.from, self.to, self.status
            ),
        }
    }
}

impl FromStr for Ride {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() < 3 {
            return Err("Not enough parts to create a Ride".to_string());
        }

        let id_passenger = parts[0].parse::<Id>().map_err(|e| e.to_string())?;

        let from = parts[1].parse::<Coordinate>().map_err(|e| e.to_string())?;

        let to = parts[2].parse::<Coordinate>().map_err(|e| e.to_string())?;

        let ride_status = parts[3].parse::<RideStatus>().map_err(|e| e.to_string())?;

        let driver = if parts.len() > 4 {
            parts[4].parse::<Id>().ok()
        } else {
            None
        };

        Ok(Ride {
            passenger_id: id_passenger,
            from,
            to,
            status: ride_status,
            driver,
        })
    }
}

impl fmt::Display for RideStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RideStatus::Requested => write!(f, "Requested"),
            RideStatus::LookingForDriver => write!(f, "LookingForDriver"),
            RideStatus::GoingToPickup => write!(f, "GoingToPickup"),
            RideStatus::WaitingToStart => write!(f, "WaitingToStart"),
            RideStatus::InProgress => write!(f, "InProgress"),
            RideStatus::Completed => write!(f, "Completed"),
        }
    }
}

impl FromStr for RideStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Requested" => Ok(RideStatus::Requested),
            "LookingForDriver" => Ok(RideStatus::LookingForDriver),
            "GoingToPickup" => Ok(RideStatus::GoingToPickup),
            "WaitingToStart" => Ok(RideStatus::WaitingToStart),
            "InProgress" => Ok(RideStatus::InProgress),
            "Completed" => Ok(RideStatus::Completed),
            _ => Err("Invalid RideStatus".to_string()),
        }
    }
}
