use std::collections::HashMap;
use std::fmt;

use actix::Message;

use crate::{
    coordinate::Coordinate, driver_status::DriverStatus, leader::Response, ride::Ride, Id,
};

pub enum DriverMessages {
    DriverState(DriverState),
    Election(Election),
    NewLeader(NewLeader),
    ElectionACK(ElectionACK),
    InfoACK(InfoACK),
    SendInfo(SendInfo),

    ResposeDrive(ResponseDrive),
    Travelling(Travelling),
    Arrived(Arrived),
    EndTrip(EndTrip),
    ResetRide(ResetRide),
    OnTheWay(OnTheWay),
    NoDriverAvailable(NoDriverAvailable),
    LeaderData(LeaderData),
}

const DRIVER_STATE: &str = "DriverState";
const ELECTION: &str = "Election";
const NEW_LEADER: &str = "NewLeader";
const INFO_ACK: &str = "InfoACK";
const ELECTION_ACK: &str = "ElectionACK";
const SEND_INFO: &str = "SendInfo";

const RESPONSE_DRIVE: &str = "ResponseDrive";
const ON_THE_WAY: &str = "OnTheWay";
const TRAVELLING: &str = "Travelling";
const ARRIVED: &str = "Arrived";
const END_TRIP: &str = "EndTrip";
const RESET_RIDE: &str = "ResetRide";
const NO_DRIVER_AVAILABLE: &str = "NoDriverAvailable";
const LEADER_DATA: &str = "LeaderData";

impl DriverMessages {
    pub fn from_string(msg: String) -> Result<Self, String> {
        let mut parts = msg.split_whitespace();
        let msg_type = match parts.next() {
            Some(msg) => msg,
            None => return Err("Invalid message format: missing message type".to_string()),
        };

        let content = parts.collect::<Vec<&str>>().join(" ");

        match msg_type {
            DRIVER_STATE => Ok(DriverMessages::DriverState(DriverState::from_string(
                content,
            ))),
            ELECTION => Ok(DriverMessages::Election(Election::from_string(content))),
            NEW_LEADER => Ok(DriverMessages::NewLeader(NewLeader::from_string(content))),
            INFO_ACK => Ok(DriverMessages::InfoACK(InfoACK::from_string(content))),
            ELECTION_ACK => Ok(DriverMessages::ElectionACK(ElectionACK::from_string(
                content,
            ))),
            SEND_INFO => Ok(DriverMessages::SendInfo(SendInfo::from_string(content))),

            RESPONSE_DRIVE => Ok(DriverMessages::ResposeDrive(ResponseDrive::from_string(
                content,
            ))),
            ON_THE_WAY => Ok(DriverMessages::OnTheWay(OnTheWay::from_string(content))),
            TRAVELLING => Ok(DriverMessages::Travelling(Travelling::from_string(content))),
            ARRIVED => Ok(DriverMessages::Arrived(Arrived::from_string(content))),
            END_TRIP => Ok(DriverMessages::EndTrip(EndTrip::from_string(content))),
            RESET_RIDE => Ok(DriverMessages::ResetRide(ResetRide::from_string(content))),
            NO_DRIVER_AVAILABLE => Ok(DriverMessages::NoDriverAvailable(
                NoDriverAvailable::from_string(content),
            )),
            LEADER_DATA => Ok(DriverMessages::LeaderData(LeaderData::from_string(
                &content,
            ))),

            _ => Err("Invalid message type".to_string()),
        }
    }
}

/// Represents the state of a driver.
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct DriverState {
    /// The ID of the driver.
    pub id: Id,
    /// The position of the driver.
    pub position: Coordinate,
    /// The status of the driver.
    pub status: DriverStatus,
}

impl DriverState {
    fn from_string(msg: String) -> Self {
        let mut parts = msg.split_whitespace();
        let id = parts.next().unwrap_or("0").parse::<Id>().unwrap_or(0);
        let position = parts
            .next()
            .unwrap_or("")
            .parse::<Coordinate>()
            .unwrap_or_default();
        let status = parts
            .collect::<Vec<&str>>()
            .join(" ")
            .parse::<DriverStatus>()
            .unwrap_or(DriverStatus::Free);

        Self {
            id,
            position,
            status,
        }
    }
}
impl fmt::Display for DriverState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} {} {} {}",
            DRIVER_STATE, self.id, self.position, self.status
        )
    }
}

/// Represents a message indicating the driver is on the way.
#[derive(Message)]
#[rtype(result = "()")]
pub struct OnTheWay {
    /// The ID of the driver.
    pub driver_id: Id,
    /// The time it will take to arrive.
    pub time_to_arrive: usize,
}

impl OnTheWay {
    pub fn from_string(msg: String) -> Self {
        let mut parts = msg.split_whitespace();
        let driver_id = parts
            .next()
            .and_then(|part| part.parse::<Id>().ok())
            .unwrap_or(0);
        let time_to_arrive = parts
            .next()
            .and_then(|part| part.parse::<usize>().ok())
            .unwrap_or(0);

        Self {
            driver_id,
            time_to_arrive,
        }
    }
}
impl fmt::Display for OnTheWay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} {} {}",
            ON_THE_WAY, self.driver_id, self.time_to_arrive
        )
    }
}

/// Represents a message to start an election for a new leader.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Election {
    /// The ID of a driver.
    pub id: Id,
}

impl Election {
    fn from_string(msg: String) -> Self {
        let id = msg.trim().parse::<Id>().unwrap_or(0);

        Self { id }
    }
}
impl fmt::Display for Election {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}", ELECTION, self.id)
    }
}

/// Represents a message indicating a new leader.
#[derive(Message)]
#[rtype(result = "()")]
pub struct NewLeader {
    /// The ID of the new leader.
    pub id: Id,
}

impl NewLeader {
    fn from_string(msg: String) -> Self {
        let id = msg.parse::<Id>().unwrap_or(0);

        Self { id }
    }
}
impl fmt::Display for NewLeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}", NEW_LEADER, self.id)
    }
}

/// Represents a message to send information to the leader.
#[derive(Message)]
#[rtype(result = "()")]
pub struct SendInfo;

impl SendInfo {
    fn from_string(_msg: String) -> Self {
        Self
    }
}

impl fmt::Display for SendInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", SEND_INFO)
    }
}

/// Represents a message indicating an election message was received.
#[derive(Message)]
#[rtype(result = "()")]
pub struct ElectionACK;

impl ElectionACK {
    fn from_string(_msg: String) -> Self {
        Self
    }
}
impl fmt::Display for ElectionACK {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", ELECTION_ACK)
    }
}

/// Represents a message indicating an information was received.
#[derive(Message)]
#[rtype(result = "()")]
pub struct InfoACK;

impl InfoACK {
    fn from_string(_msg: String) -> Self {
        Self
    }
}
impl fmt::Display for InfoACK {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", INFO_ACK)
    }
}

/// Represents a message indicating a driver is responding to a drive request.
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct ResponseDrive {
    /// The ID of the driver.
    pub driver_id: Id,
    /// The ride the driver is responding to.
    pub ride: Ride,
}

impl ResponseDrive {
    pub fn from_string(msg: String) -> Self {
        let mut parts = msg.split_whitespace();
        let driver_id = parts
            .next()
            .and_then(|part| part.parse::<Id>().ok())
            .unwrap_or(0);

        let ride = parts
            .collect::<Vec<&str>>()
            .join(" ")
            .parse::<Ride>()
            .unwrap_or_default();
        Self { driver_id, ride }
    }
}
impl fmt::Display for ResponseDrive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {} {}", RESPONSE_DRIVE, self.driver_id, self.ride)
    }
}

/// Represents a message indicating a driver is travelling.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Travelling {}

impl fmt::Display for Travelling {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", TRAVELLING)
    }
}

impl Travelling {
    pub fn from_string(_: String) -> Self {
        Self {}
    }
}

/// Represents a message indicating a driver has arrived.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Arrived {
    /// The ID of the driver.
    pub driver_id: Id,
    /// The time it took to arrive.
    pub time_to_arrive: usize,
}

impl Arrived {
    pub fn from_string(msg: String) -> Self {
        let mut parts = msg.split_whitespace();
        let driver_id = parts
            .next()
            .and_then(|part| part.parse::<Id>().ok())
            .unwrap_or(0);
        let time_to_arrive = parts
            .next()
            .and_then(|part| part.parse::<usize>().ok())
            .unwrap_or(0);

        Self {
            driver_id,
            time_to_arrive,
        }
    }
}
impl fmt::Display for Arrived {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {} {}", ARRIVED, self.driver_id, self.time_to_arrive)
    }
}

/// Represents a message indicating a driver has ended a trip.
#[derive(Message)]
#[rtype(result = "()")]
pub struct EndTrip {
    /// The ID of the driver.
    pub driver_id: Id,
}

impl fmt::Display for EndTrip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}", END_TRIP, self.driver_id)
    }
}

impl EndTrip {
    pub fn from_string(msg: String) -> Self {
        let mut parts = msg.split_whitespace();
        let driver_id = parts
            .next()
            .and_then(|part| part.parse::<Id>().ok())
            .unwrap_or(0);

        Self { driver_id }
    }
}

/// Represents a message to reset a ride.
#[derive(Message)]
#[rtype(result = "()")]
pub struct ResetRide {
    /// The ID of the passenger.
    pub id_passenger: Id,
}

impl fmt::Display for ResetRide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}", RESET_RIDE, self.id_passenger)
    }
}

impl ResetRide {
    pub fn from_string(msg: String) -> Self {
        let mut parts = msg.split_whitespace();
        let id_passenger = parts
            .next()
            .and_then(|part| part.parse::<Id>().ok())
            .unwrap_or(0);

        Self { id_passenger }
    }
}

/// Represents a message indicating no driver is available.
#[derive(Message)]
#[rtype(result = "()")]
pub struct NoDriverAvailable {}

impl NoDriverAvailable {
    pub fn from_string(_: String) -> Self {
        Self {}
    }
}

impl fmt::Display for NoDriverAvailable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", NO_DRIVER_AVAILABLE)
    }
}

#[derive(Message, Clone, serde::Serialize, serde::Deserialize, Default)]
#[rtype(result = "()")]
pub struct LeaderData {
    #[serde(default)]
    pub active_rides: HashMap<Id, Ride>,
    #[serde(default)]
    pub drivers_states: HashMap<Id, (Coordinate, DriverStatus)>,
    #[serde(default)]
    pub asked_drivers: HashMap<Id, HashMap<Id, Response>>,
}

impl LeaderData {
    pub fn from_string(msg: &str) -> Self {
        serde_json::from_str(msg).unwrap_or_else(|_| {
            eprintln!("Failed to parse LeaderData, using default.");
            LeaderData::default()
        })
    }
}
impl fmt::Display for LeaderData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => writeln!(f, "{} {}", LEADER_DATA, json),
            Err(err) => writeln!(f, "Error serializing LeaderInfo: {}", err),
        }
    }
}
