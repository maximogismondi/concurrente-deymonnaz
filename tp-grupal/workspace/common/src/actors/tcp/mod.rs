use actix::{dev::ToEnvelope, Actor, Handler};

use crate::messages::tcp_messages::{CloseConnection, ReceivedTcpMessage};

use super::concu_ride_actor::ConcuRideActor;

pub mod tcp_connection;
pub mod tcp_layer;

pub trait TCPConcuRideActor:
    ConcuRideActor + Actor + Handler<ReceivedTcpMessage> + Handler<CloseConnection>
{
}

pub type TCPConcuRideActorContext<A> = <A as Actor>::Context;

pub trait TCPConcuRideActorToEvelope<A>:
    ToEnvelope<A, ReceivedTcpMessage> + ToEnvelope<A, CloseConnection>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: ToEnvelope<A, ReceivedTcpMessage> + ToEnvelope<A, CloseConnection>,
{
}
