use actix::{Addr, Message};

use crate::{
    actors::{
        actor_type::ActorType,
        tcp::{
            tcp_connection::TcpConnection, TCPConcuRideActor, TCPConcuRideActorContext,
            TCPConcuRideActorToEvelope,
        },
    },
    Id,
};

/// Message to send data over TCP
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct SendTcpMessage {
    /// The identifier of the sender.
    pub to_id: Id,
    /// The type of the sender.
    pub to_actor_type: ActorType,
    /// The content of the message.
    pub content: String,
}

/// Message to process data received from TCP
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct ReceivedTcpMessage {
    /// The identifier of the sender.
    pub from_id: Id,
    /// The type of the sender.
    pub from_actor_type: ActorType,
    /// The content of the message.
    pub content: String,
}

/// Message to assign an actor to a connection
#[derive(Message)]
#[rtype(result = "()")]
pub struct AssignActor<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    /// The identifier of the actor.
    pub actor: Addr<A>,
}

/// Message to create a new connection
#[derive(Message)]
#[rtype(result = "()")]
pub struct NewConnection<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    /// The identifier of the connection.
    pub id: Id,
    /// The type of the actor.
    pub actor_type: ActorType,
    /// The TCP connection.
    pub connection: Addr<TcpConnection<A>>,
}

/// Message to close a connection
#[derive(Message)]
#[rtype(result = "()")]
pub struct CloseConnection {
    /// The identifier of the connection.
    pub id: Id,
    /// The type of the actor.
    pub actor_type: ActorType,
}
