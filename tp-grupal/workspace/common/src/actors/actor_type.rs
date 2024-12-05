use std::fmt;

pub const ACTOR_TYPE_SIZE: usize = 1;

/// Represents the type of an actor.
///
/// # Variants
///
/// - `Driver`: The actor is a driver.
/// - `Passenger`: The actor is a passenger.
/// - `Gateway`: The actor is a gateway.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ActorType {
    Driver,
    Passenger,
    Gateway,
}

impl ActorType {
    /// Converts the `ActorType` to a big-endian byte array.
    pub fn to_be_bytes(&self) -> [u8; ACTOR_TYPE_SIZE] {
        match self {
            ActorType::Driver => [0],
            ActorType::Passenger => [1],
            ActorType::Gateway => [2],
        }
    }

    /// Converts a big-endian byte array to an `ActorType`.
    pub fn from_be_bytes(bytes: [u8; ACTOR_TYPE_SIZE]) -> Self {
        match bytes {
            [0] => ActorType::Driver,
            [1] => ActorType::Passenger,
            [2] => ActorType::Gateway,
            _ => panic!("Invalid actor type"),
        }
    }
}

impl fmt::Display for ActorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActorType::Driver => write!(f, "Driver"),
            ActorType::Passenger => write!(f, "Passenger"),
            ActorType::Gateway => write!(f, "Gateway"),
        }
    }
}
