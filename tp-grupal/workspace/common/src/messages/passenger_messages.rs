use actix::Message;
use std::fmt;

use crate::{coordinate::Coordinate, ride::Ride, Id};

const REQUEST_RIDE: &str = "RequestRide";
const REQUEST_ACK: &str = "RequestAck";
const RIDE_CONFIRMED: &str = "RideConfirmed";
const ON_DRIVE: &str = "OnDrive";
const ACK_END_TRIP: &str = "AckEndTrip";
const NEW_REQUEST_DRIVE: &str = "NewRequestRide";

pub enum PassengerMessages {
    RequestRide(RequestRide),
    RequestACK(RequestACK),
    RideConfirmed(RideConfirmed),
    OnDrive(OnDrive),
    AckEndTrip(AckEndTrip),
}

impl PassengerMessages {
    pub fn from_string(msg: String) -> Result<Self, String> {
        let mut parts = msg.split_whitespace();
        let msg_type = match parts.next() {
            Some(msg) => msg,
            None => return Err("Invalid message format: missing message type".to_string()),
        };

        let content = parts.collect::<Vec<&str>>().join(" ");

        match msg_type {
            REQUEST_RIDE => Ok(PassengerMessages::RequestRide(RequestRide::from_string(
                content,
            ))),
            REQUEST_ACK => Ok(PassengerMessages::RequestACK(RequestACK::from_string(
                content,
            ))),
            RIDE_CONFIRMED => Ok(PassengerMessages::RideConfirmed(
                RideConfirmed::from_string(content),
            )),
            ON_DRIVE => Ok(PassengerMessages::OnDrive(OnDrive::from_string(content))),
            ACK_END_TRIP => Ok(PassengerMessages::AckEndTrip(AckEndTrip::from_string(
                content,
            ))),
            _ => Err("Invalid message type".to_string()),
        }
    }
}

/// Message to request a ride.
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct RequestRide {
    pub ride: Ride,
}

impl RequestRide {
    pub fn from_string(msg: String) -> Self {
        let ride = msg.parse::<Ride>().unwrap_or(Ride::default());

        Self { ride }
    }
}
impl fmt::Display for RequestRide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}", REQUEST_RIDE, self.ride)
    }
}

/// Message to acknowledge a request.
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct RequestACK {}

impl RequestACK {
    pub fn from_string(_msg: String) -> Self {
        Self {}
    }
}

impl fmt::Display for RequestACK {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", REQUEST_ACK)
    }
}

/// Message to confirm a ride.
#[derive(Message)]
#[rtype(result = "()")]
pub struct RideConfirmed {
    pub driver_id: Id,
}

impl RideConfirmed {
    pub fn from_string(msg: String) -> Self {
        let mut parts = msg.split_whitespace();
        let driver_id = parts
            .next()
            .and_then(|part| part.parse::<Id>().ok())
            .unwrap_or(0);

        Self { driver_id }
    }
}
impl fmt::Display for RideConfirmed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}", RIDE_CONFIRMED, self.driver_id)
    }
}

/// Message to indicate that the passenger is on a drive.
#[derive(Message)]
#[rtype(result = "()")]
pub struct OnDrive {
    pub passenger_id: Id,
}

/// Message to acknowledge the end of a trip.
#[derive(Message)]
#[rtype(result = "()")]
pub struct AckEndTrip {}

impl AckEndTrip {}

impl AckEndTrip {
    pub fn from_string(_msg: String) -> Self {
        Self {}
    }
}
impl fmt::Display for AckEndTrip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", ACK_END_TRIP)
    }
}
impl OnDrive {
    pub fn from_string(msg: String) -> Self {
        let mut parts = msg.split_whitespace();
        let passenger_id = parts
            .next()
            .and_then(|part| part.parse::<Id>().ok())
            .unwrap_or(0);

        Self { passenger_id }
    }
}
impl fmt::Display for OnDrive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}", ON_DRIVE, self.passenger_id)
    }
}

/// Message to request a new ride.
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct NewRequestRide {
    pub id: Id,
    pub from: Coordinate,
    pub to: Coordinate,
}

impl NewRequestRide {
    pub fn from_string(msg: String) -> Self {
        let mut parts = msg.split_whitespace();
        let id = parts
            .next()
            .and_then(|part| part.parse::<Id>().ok())
            .unwrap_or(0);
        let from = parts
            .next()
            .and_then(|part| part.parse::<Coordinate>().ok())
            .unwrap_or(Coordinate::new(0, 0));
        let to = parts
            .next()
            .and_then(|part| part.parse::<Coordinate>().ok())
            .unwrap_or(Coordinate::new(0, 0));

        Self { id, from, to }
    }
}
impl fmt::Display for NewRequestRide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} {} {} {}",
            NEW_REQUEST_DRIVE, self.id, self.from, self.to
        )
    }
}
