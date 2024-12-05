use actix::{Actor, ActorContext, Addr, Context, Handler, StreamHandler};
use actix_async_handler::async_handler;

use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
};

use crate::{
    actors::{actor_type::ActorType, concu_ride_actor::ConcuRideActor},
    messages::{
        concu_ride_messages::{Disconnect, Initialize},
        tcp_messages::{CloseConnection, ReceivedTcpMessage, SendTcpMessage},
    },
    Id,
};

use super::{
    tcp_layer::TcpLayer, TCPConcuRideActor, TCPConcuRideActorContext, TCPConcuRideActorToEvelope,
};

const TCP_CONNECTION_ROLE: &str = "TCP Connection";

/// Represents a TCP connection between two actors.
pub struct TcpConnection<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    /// The identifier of the actor.
    pub my_id: Id,
    /// The type of the actor.
    pub my_actor_type: ActorType,
    /// The identifier of the other actor.
    pub other_id: Id,
    /// The type of the other actor.
    pub other_actor_type: ActorType,
    /// The write half of the TCP stream.
    pub write: Option<WriteHalf<TcpStream>>,
    /// The actor address.
    pub actor: Addr<A>,
    /// The TCP layer address.
    pub tcp_layer: Addr<TcpLayer<A>>,
}

impl<A> TcpConnection<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    /// Creates a new `TcpConnection` instance.
    pub fn new(
        my_id: Id,
        my_actor_type: ActorType,
        other_id: Id,
        other_actor_type: ActorType,
        write: WriteHalf<TcpStream>,
        actor: Addr<A>,
        tcp_layer: Addr<TcpLayer<A>>,
    ) -> Self {
        Self {
            my_id,
            my_actor_type,
            other_id,
            other_actor_type,
            write: Some(write),
            actor,
            tcp_layer,
        }
    }
}

impl<A> ConcuRideActor for TcpConnection<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    fn id(&self) -> String {
        format!(
            "{} {} {} -> {} {}",
            TCP_CONNECTION_ROLE,
            self.my_actor_type,
            self.my_id,
            self.other_actor_type,
            self.other_id
        )
    }
}

impl<A> Actor for TcpConnection<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    type Context = Context<Self>;
}

impl<A> StreamHandler<Result<String, std::io::Error>> for TcpConnection<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    fn handle(&mut self, read: Result<String, std::io::Error>, _ctx: &mut Self::Context) {
        match read {
            Ok(line) => {
                self.actor.do_send(ReceivedTcpMessage {
                    from_id: self.other_id,
                    from_actor_type: self.other_actor_type.clone(),
                    content: line,
                });
            }
            Err(read_err) => {
                eprintln!("Reading error: {}", read_err);
            }
        }
    }

    fn finished(&mut self, ctx: &mut Self::Context) {
        self.tcp_layer.do_send(CloseConnection {
            id: self.other_id,
            actor_type: self.other_actor_type.clone(),
        });
        ctx.stop();
    }
}

#[async_handler]
impl<A> Handler<SendTcpMessage> for TcpConnection<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    type Result = ();

    async fn handle(&mut self, msg: SendTcpMessage, _ctx: &mut Self::Context) -> Self::Result {
        let mut write = self.write.take().expect(
            "I shouldn't be able to get another message before it returns by using AtomicResponse",
        );

        let ret_write = async move {
            match write.write_all(msg.content.as_bytes()).await {
                Ok(_) => {}
                Err(write_err) => {
                    eprintln!("Writing error: {}", write_err);
                }
            }
            write
        }
        .await;
        self.write = Some(ret_write);
    }
}

impl<A> Handler<Disconnect> for TcpConnection<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    type Result = ();

    fn handle(&mut self, _msg: Disconnect, ctx: &mut Self::Context) -> Self::Result {
        self.log("⚠️ Disconnecting");
        ctx.stop();
    }
}

impl<A> Handler<Initialize> for TcpConnection<A>
where
    A: TCPConcuRideActor,
    TCPConcuRideActorContext<A>: TCPConcuRideActorToEvelope<A>,
{
    type Result = ();

    fn handle(&mut self, _msg: Initialize, _ctx: &mut Self::Context) -> Self::Result {}
}
