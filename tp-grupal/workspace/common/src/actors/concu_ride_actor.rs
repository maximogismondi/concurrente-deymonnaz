/// Trait for actors in the ConcuRide system.
pub trait ConcuRideActor {
    /// Returns the unique identifier for the actor.
    fn id(&self) -> String;

    /// Logs a message with the actor's identifier.
    fn log(&self, msg: &str) {
        println!("[{}] {}", self.id(), msg);
    }
}
