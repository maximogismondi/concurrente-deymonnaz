use std::fmt;

use actix::Message;

use crate::ride::Ride;

pub enum LeaderMessages {
    LookForDriver(LookForDriver),
    DriveOffer(DriveOffer),
    InitDrive(InitDrive),
}

const LOOK_FOR_DRIVER: &str = "LookForDriver";
const DRIVE_OFFER: &str = "DriveOffer";
const INIT_DRIVE: &str = "InitDrive";

impl LeaderMessages {
    pub fn from_string(msg: String) -> Result<Self, String> {
        let mut parts = msg.split_whitespace();
        let msg_type = match parts.next() {
            Some(msg) => msg,
            None => return Err("Invalid message format: missing message type".to_string()),
        };

        let content = parts.collect::<Vec<&str>>().join(" ");

        match msg_type {
            LOOK_FOR_DRIVER => Ok(LeaderMessages::LookForDriver(LookForDriver::from_string(
                content,
            ))),
            DRIVE_OFFER => Ok(LeaderMessages::DriveOffer(DriveOffer::from_string(content))),
            INIT_DRIVE => Ok(LeaderMessages::InitDrive(InitDrive::from_string(content))),
            _ => Err("Invalid message type".to_string()),
        }
    }
}

/// Message to initialize a drive.
#[derive(Message)]
#[rtype(result = "()")]
pub struct InitDrive {
    /// The ride to initialize.
    pub ride: Ride,
}

impl InitDrive {
    pub fn from_string(msg: String) -> Self {
        let ride = msg.parse::<Ride>().unwrap_or(Ride::default());

        Self { ride }
    }
}

impl fmt::Display for InitDrive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}", INIT_DRIVE, self.ride)
    }
}

/// Message to look for a driver to  do a ride.
#[derive(Message)]
#[rtype(result = "()")]
pub struct LookForDriver {
    pub ride: Ride,
}

impl LookForDriver {
    pub fn from_string(msg: String) -> Self {
        let ride = msg.parse::<Ride>().unwrap_or_default();

        Self { ride }
    }
}
impl fmt::Display for LookForDriver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}", LOOK_FOR_DRIVER, self.ride)
    }
}

/// Message to offer a drive to a driver.
#[derive(Message)]
#[rtype(result = "()")]
pub struct DriveOffer {
    pub ride: Ride,
}
impl DriveOffer {
    pub fn from_string(msg: String) -> Self {
        let ride = msg.parse::<Ride>().unwrap_or_default();

        Self { ride }
    }
}
impl fmt::Display for DriveOffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}", DRIVE_OFFER, self.ride)
    }
}
