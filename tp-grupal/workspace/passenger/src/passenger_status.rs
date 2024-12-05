/// Represents the various statuses a passenger can have during their journey.
///
/// # Variants
///
/// - `Init`: Initial state of the passenger.
/// - `AskingDrive`: Passenger is requesting a drive.
/// - `WaitingForAssignation`: Passenger is waiting for a driver to be assigned.
/// - `WaitingForDriverConfirmation`: Passenger is waiting for the driver to confirm the ride.
/// - `WaitingDriver`: Passenger is waiting for the driver to arrive.
/// - `OnRide`: Passenger is currently on the ride.
/// - `Arrived`: Passenger has arrived at the destination.
#[derive(PartialEq)]
pub enum PassengerStatus {
    Init,
    AskingDrive,
    WaitingForAssignation,
    WaitingForDriverConfirmation,
    WaitingDriver,
    OnRide,
    Arrived,
}
