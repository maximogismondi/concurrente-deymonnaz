use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{coordinate::Coordinate, driver_status::DriverStatus, ride::Ride, Id};

const MAX_DISTANCE: usize = 30;

/// Represents the leader in the ConcuRide system.
#[derive(Default)]
pub struct Leader {
    /// A map of active rides associated with their IDs.
    pub active_rides: HashMap<Id, Ride>,
    /// A map of driver states associated with their IDs.
    pub drivers_states: HashMap<Id, (Coordinate, DriverStatus)>,
    /// A map of asked drivers associated with their IDs.
    pub asked_drivers: HashMap<Id, HashMap<Id, Response>>,
}

/// Represents the response status of a driver.
///
/// # Variants
///
/// - `Pending`: The driver has not responded yet.
/// - `Accept`: The driver has accepted the ride.
/// - `Reject`: The driver has rejected the ride.
#[derive(Clone, Deserialize, Serialize)]
pub enum Response {
    Pending,
    Accept,
    Reject,
}

impl Leader {
    /// Creates a new `Leader` instance.
    ///
    /// # Returns
    ///
    /// A new `Leader` instance.
    pub fn new() -> Self {
        let active_rides = HashMap::new();
        let position_drivers = HashMap::new();
        let asked_drivers = HashMap::new();

        Self {
            active_rides,
            drivers_states: position_drivers,
            asked_drivers,
        }
    }

    /// Searches for the nearest available driver for a given ride.
    ///
    /// # Arguments
    ///
    /// * `ride` - The ride for which to search for a driver.
    ///
    /// # Returns
    ///
    /// An optional identifier of the nearest available driver.
    pub fn search_nearest_driver(&self, ride: Ride) -> Option<Id> {
        self.drivers_states
            .iter()
            .filter(|(_, (_, status))| status.is_free())
            .filter(|(id, _)| {
                self.asked_drivers
                    .get(&ride.passenger_id)
                    .map(|asked| !asked.contains_key(id))
                    .unwrap_or(true)
            })
            .map(|(id, (coord, _))| (*id, ride.from.distance(coord)))
            .filter(|(_, distance)| *distance <= MAX_DISTANCE)
            .min_by(|(_, dist1), (_, dist2)| dist1.cmp(dist2))
            .map(|(id, _)| id)
    }
}
