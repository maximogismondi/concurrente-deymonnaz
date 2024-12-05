use actix::Message;

/// Represents a message to initialize a ConcuRide actor.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Initialize;

/// Represents a message to disconnect a ConcuRide actor.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect;
