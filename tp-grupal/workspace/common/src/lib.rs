use std::collections::HashMap;

use actors::actor_type::ActorType;

pub mod actors;
pub mod config;
pub mod coordinate;
pub mod driver_status;
pub mod leader;
pub mod messages;
pub mod payment;
pub mod ride;

pub type Peers = HashMap<ActorType, HashMap<Id, String>>;

pub type Id = u32;
pub const ID_SIZE: usize = 4;
