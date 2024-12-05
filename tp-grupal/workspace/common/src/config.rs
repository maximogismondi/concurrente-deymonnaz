use serde::Deserialize;
use std::{collections::HashMap, fs};

use crate::{actors::actor_type::ActorType, coordinate::Coordinate, Id, Peers};

/// Represents the configuration of a driver in the ConcuRide system.
#[derive(Debug, Deserialize, Clone)]
pub struct DriverConfig {
    /// The identifier of the driver.
    pub id: Id,
    /// The address of the driver.
    pub address: String,
    /// The starting position of the driver.
    pub position: Coordinate,
}

/// Represents the configuration of a passenger in the ConcuRide system.
#[derive(Debug, Deserialize, Clone)]
pub struct PassengerConfig {
    /// The identifier of the passenger.
    pub id: Id,
    /// The address of the passenger.
    pub address: String,
    /// The starting position of the passenger.
    pub origin: Coordinate,
    /// The destination position of the passenger.
    pub destination: Coordinate,
}

/// Represents the configuration of a gateway in the ConcuRide system.
#[derive(Debug, Deserialize, Clone)]
pub struct GatewayConfig {
    /// The identifier of the gateway.
    pub id: Id,
    /// The address of the gateway.
    pub address: String,
}

/// Custom trait to ensure types have an `id`
pub trait HasId {
    fn id(&self) -> Id;
}

impl HasId for DriverConfig {
    fn id(&self) -> Id {
        self.id
    }
}

impl HasId for PassengerConfig {
    fn id(&self) -> Id {
        self.id
    }
}

impl HasId for GatewayConfig {
    fn id(&self) -> Id {
        self.id
    }
}

/// Represents the configuration of the ConcuRide system.
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "array_to_map")]
    drivers: HashMap<Id, DriverConfig>,
    #[serde(deserialize_with = "array_to_map")]
    passengers: HashMap<Id, PassengerConfig>,
    #[serde(deserialize_with = "array_to_map")]
    gateways: HashMap<Id, GatewayConfig>,
    gateway_accept_rate: f64,
    driver_accept_rate: f64,
}

impl Config {
    /// Creates a new `Config` instance.
    pub fn from_file(path: &str) -> Result<Self, String> {
        let json_data = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let config: Config = serde_json::from_str(&json_data).map_err(|e| e.to_string())?;

        Ok(config)
    }

    /// Returns a map of peers in the network.
    ///
    /// # Returns
    ///
    /// A map of peers in the network.
    pub fn peers(&self) -> Peers {
        let mut peers = HashMap::new();

        let mut drivers = HashMap::new();
        for (id, driver) in &self.drivers {
            drivers.insert(*id, driver.address.clone());
        }
        peers.insert(ActorType::Driver, drivers);

        let mut passengers = HashMap::new();
        for (id, passenger) in &self.passengers {
            passengers.insert(*id, passenger.address.clone());
        }
        peers.insert(ActorType::Passenger, passengers);

        let mut gateways = HashMap::new();
        for (id, gateway) in &self.gateways {
            gateways.insert(*id, gateway.address.clone());
        }
        peers.insert(ActorType::Gateway, gateways);

        peers
    }

    /// Returns the configuration of a driver.
    pub fn driver(&self, id: Id) -> Option<DriverConfig> {
        self.drivers.get(&id).cloned()
    }

    /// Returns the configuration of a passenger.
    pub fn passenger(&self, id: Id) -> Option<PassengerConfig> {
        self.passengers.get(&id).cloned()
    }

    /// Returns the configuration of a gateway.
    pub fn gateway(&self, id: Id) -> Option<GatewayConfig> {
        self.gateways.get(&id).cloned()
    }

    /// Returns the gateway accept rate.
    pub fn gateway_accept_rate(&self) -> f64 {
        self.gateway_accept_rate
    }

    /// Returns the driver accept rate.
    pub fn driver_accept_rate(&self) -> f64 {
        self.driver_accept_rate
    }
}

/// Custom deserialization function to convert arrays to HashMaps
fn array_to_map<'de, D, T>(deserializer: D) -> Result<HashMap<Id, T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de> + HasId,
{
    let vec: Vec<T> = Deserialize::deserialize(deserializer)?;
    let map = vec
        .into_iter()
        .map(|item| {
            let id = item.id();
            (id, item)
        })
        .collect();
    Ok(map)
}
