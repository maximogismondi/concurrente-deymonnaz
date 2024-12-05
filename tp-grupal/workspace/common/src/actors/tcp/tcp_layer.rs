use actix::{
    fut::wrap_future, Actor, ActorContext, Addr, AsyncContext, Context, Handler, StreamHandler,
};
use actix_async_handler::async_handler;

use tokio::{
    io::{split, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};
use tokio_stream::wrappers::LinesStream;

use crate::{
    actors::{
        actor_type::{ActorType, ACTOR_TYPE_SIZE},
        concu_ride_actor::ConcuRideActor,
    },
    messages::{
        concu_ride_messages::{Disconnect, Initialize},
        tcp_messages::{
            AssignActor, CloseConnection, NewConnection, ReceivedTcpMessage, SendTcpMessage,
        },
    },
    Id, Peers, ID_SIZE,
};
use std::collections::HashMap;

use super::{
    tcp_connection::TcpConnection, TCPConcuRideActor, TCPConcuRideActorContext,
    TCPConcuRideActorToEvelope,
};

const TCP_LAYER_ROLE: &str = "TCP Layer";

/// Represents the TCP layer in the ConcuRide system.
pub struct TcpLayer<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    /// The identifier of the actor connected to the TCP layer.
    pub id: Id,
    /// The type of the actor.
    pub actor_type: ActorType,
    /// A map of connections associated with their IDs.
    pub connections: HashMap<ActorType, HashMap<Id, Addr<TcpConnection<A>>>>,
    /// A map of peers in the network.
    pub peers: Peers,
    /// The actor address.
    pub actor: Option<Addr<A>>,
}

impl<A> ConcuRideActor for TcpLayer<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    fn id(&self) -> String {
        format!("{} {}", TCP_LAYER_ROLE, self.id)
    }
}

impl<A> TcpLayer<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    /// Creates a new `TcpLayer` instance.
    pub fn new(id: Id, actor_type: ActorType, peers: Peers) -> Self {
        Self {
            id,
            actor_type,
            connections: HashMap::new(),
            peers,
            actor: None,
        }
    }

    /// Serialize Id and ActorType to bytes
    /// The first 4 bytes are the Id and the last byte is the ActorType
    fn serialize_id_and_actor_type(&self) -> [u8; ID_SIZE + ACTOR_TYPE_SIZE] {
        let id_as_bytes = self.id.to_be_bytes();
        let actor_type_as_bytes = self.actor_type.to_be_bytes();

        let mut bytes = Vec::new();

        for byte in id_as_bytes.iter() {
            bytes.push(*byte);
        }

        for byte in actor_type_as_bytes.iter() {
            bytes.push(*byte);
        }

        bytes.try_into().unwrap_or_default()
    }

    /// Deserializes Id and ActorType from bytes
    /// The first 4 bytes are the Id and the last byte is the ActorType
    fn deserialize_id_and_actor_type(bytes: [u8; ID_SIZE + ACTOR_TYPE_SIZE]) -> (Id, ActorType) {
        let id = Id::from_be_bytes(bytes[..ID_SIZE].try_into().unwrap_or_default());
        let actor_type = ActorType::from_be_bytes(
            bytes[ID_SIZE..ID_SIZE + ACTOR_TYPE_SIZE]
                .try_into()
                .unwrap_or_default(),
        );

        (id, actor_type)
    }
}

impl<A> Actor for TcpLayer<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    type Context = Context<Self>;
}

impl<A> Handler<AssignActor<A>> for TcpLayer<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    type Result = ();

    fn handle(&mut self, msg: AssignActor<A>, _ctx: &mut Self::Context) -> Self::Result {
        self.actor = Some(msg.actor);
    }
}

impl<A> Handler<NewConnection<A>> for TcpLayer<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    type Result = ();

    fn handle(&mut self, msg: NewConnection<A>, _ctx: &mut Self::Context) -> Self::Result {
        self.log(&format!(
            "➕ New connection from {} with id {}",
            msg.actor_type, msg.id
        ));
        msg.connection.do_send(Initialize {});
        self.connections
            .entry(msg.actor_type)
            .or_default()
            .insert(msg.id, msg.connection);
    }
}

impl<A> Handler<CloseConnection> for TcpLayer<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    type Result = ();

    fn handle(&mut self, msg: CloseConnection, _ctx: &mut Self::Context) -> Self::Result {
        self.log(&format!(
            "⚠️ Closing connection with {} with id {}",
            msg.actor_type, msg.id
        ));
        if let Some(connections) = self.connections.get_mut(&msg.actor_type) {
            connections.remove(&msg.id);
        }
        self.actor
            .as_ref()
            .expect("Actor not assigned")
            .do_send(msg);
    }
}

#[allow(clippy::unused_unit)]
#[async_handler]
impl<A> Handler<SendTcpMessage> for TcpLayer<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    type Result = ();

    async fn handle(&mut self, msg: SendTcpMessage, _ctx: &mut Self::Context) {
        if msg.to_actor_type == self.actor_type && msg.to_id == self.id {
            self.actor
                .as_ref()
                .expect("Actor not assigned")
                .do_send(ReceivedTcpMessage {
                    from_id: self.id,
                    from_actor_type: self.actor_type.clone(),
                    content: msg.content,
                });
        } else if let Some(addr) = self
            .connections
            .get(&msg.to_actor_type)
            .and_then(|map| map.get(&msg.to_id))
        {
            addr.do_send(SendTcpMessage {
                to_id: msg.to_id,
                to_actor_type: msg.to_actor_type,
                content: msg.content,
            });
        } else if let Some(tcp_address) = self
            .peers
            .get(&msg.to_actor_type)
            .and_then(|map| map.get(&msg.to_id))
            .cloned()
        {
            let bytes = self.serialize_id_and_actor_type();

            let result_stream = async move {
                let mut stream = match TcpStream::connect(tcp_address).await {
                    Ok(stream) => stream,
                    Err(_) => return Err(()),
                };

                match stream.write_all(&bytes).await {
                    Ok(_) => {}
                    Err(_) => return Err(()),
                }
                Ok(stream)
            }
            .await;

            let stream = match result_stream {
                Ok(stream) => stream,
                Err(_) => {
                    self.log(&format!(
                        "⚠️ Could not connect to {} with id {}",
                        msg.to_actor_type, msg.to_id
                    ));
                    return;
                }
            };

            let actor = self.actor.as_ref().expect("Driver not assigned").clone();
            let my_address = _ctx.address().clone();

            let connection = TcpConnection::create(|ctx| {
                let (read, write_half) = split(stream);
                TcpConnection::add_stream(LinesStream::new(BufReader::new(read).lines()), ctx);
                TcpConnection::new(
                    self.id,
                    self.actor_type.clone(),
                    msg.to_id,
                    msg.to_actor_type.clone(),
                    write_half,
                    actor,
                    my_address,
                )
            });

            _ctx.address().do_send(NewConnection {
                id: msg.to_id,
                actor_type: msg.to_actor_type.clone(),
                connection: connection.clone(),
            });

            connection.do_send(msg.clone());
        } else {
            self.log(&format!("⚠️ Peer {} not found", msg.to_id));
        }
    }
}

impl<A> Handler<Disconnect> for TcpLayer<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    type Result = ();

    fn handle(&mut self, _msg: Disconnect, ctx: &mut Self::Context) -> Self::Result {
        self.log("⚠️ Disconnecting");
        for connections in self.connections.values() {
            for connection in connections.values() {
                connection.do_send(Disconnect {});
            }
        }
        ctx.stop();
    }
}

impl<A> Handler<Initialize> for TcpLayer<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    type Result = ();

    fn handle(&mut self, _msg: Initialize, ctx: &mut Self::Context) -> Self::Result {
        let my_tcp_address = self
            .peers
            .get(&self.actor_type)
            .expect("No TCP address assigned")
            .get(&self.id)
            .expect("No TCP address assigned")
            .clone();

        let my_address = ctx.address().clone();
        let my_id = self.id;
        let my_actor_type = self.actor_type.clone();

        let actor = self.actor.clone().expect("Actor not assigned");

        let future = async move {
            let listener = TcpListener::bind(my_tcp_address)
                .await
                .expect("Failed to bind");

            while let Ok((mut stream, _addr)) = listener.accept().await {
                let mut buf = [0u8; ID_SIZE + ACTOR_TYPE_SIZE];

                match stream.read_exact(&mut buf).await {
                    Ok(_) => {}
                    Err(_) => {
                        eprintln!("Error reading id from peer");
                        continue;
                    }
                }

                let (new_id, new_actor_type) = Self::deserialize_id_and_actor_type(buf);

                let connection = TcpConnection::create(|ctx| {
                    let (read, write_half) = split(stream);
                    TcpConnection::add_stream(LinesStream::new(BufReader::new(read).lines()), ctx);
                    TcpConnection::new(
                        my_id,
                        my_actor_type.clone(),
                        new_id,
                        new_actor_type.clone(),
                        write_half,
                        actor.clone(),
                        my_address.clone(),
                    )
                });

                my_address.do_send(NewConnection {
                    id: new_id,
                    actor_type: new_actor_type,
                    connection,
                });
            }
        };

        ctx.spawn(wrap_future(future));
    }
}
