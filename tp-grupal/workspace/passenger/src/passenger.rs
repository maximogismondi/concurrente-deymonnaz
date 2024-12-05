use std::collections::HashSet;
use std::time::Duration;

use common::{
    actors::{
        actor_type::ActorType,
        concu_ride_actor::ConcuRideActor,
        tcp::{
            tcp_layer::TcpLayer, TCPConcuRideActor, TCPConcuRideActorContext,
            TCPConcuRideActorToEvelope,
        },
    },
    coordinate::Coordinate,
    driver_status::DriverStatus,
    messages::{
        concu_ride_messages::{Disconnect, Initialize},
        driver_messages::{
            Arrived, DriverMessages, DriverState, EndTrip, NoDriverAvailable, OnTheWay,
        },
        gateway_messages::{
            EnablePay, GatewayMessage, PaymentRequest, PaymentState, ReturnPayment,
        },
        passenger_messages::{
            AckEndTrip, OnDrive, PassengerMessages, RequestACK, RequestRide, RideConfirmed,
        },
        tcp_messages::{AssignActor, CloseConnection, ReceivedTcpMessage, SendTcpMessage},
    },
    payment::PaymentStatus,
    ride::Ride,
    Id, Peers,
};

use actix::{Actor, Addr, AsyncContext, Context, Handler};

use crate::passenger_status::PassengerStatus;
use rand::seq::SliceRandom;

const PASSENGER_ROLE: &str = "Passenger";

const MIN_WAITING_TIME: Duration = Duration::from_secs(2);
const REQUEST_INTERVAL: Duration = Duration::from_secs(3);
const GATEWAY_TIMEOUT: Duration = Duration::from_secs(5);

const DISCONNECTION_INTERVAL: Duration = Duration::from_secs(2);

const FEE: usize = 60;
/// Represents a passenger in the ConcuRide system.
pub struct Passenger {
    /// Unique identifier for the passenger.
    id: Id,
    /// Current position of the passenger.
    position: Coordinate,
    /// Destination coordinate of the passenger.
    destination: Coordinate,
    /// List of peers in the network.
    peers: Peers,
    /// TCP layer address for communication.
    tcp_layer: Addr<TcpLayer<Passenger>>,
    /// Optional identifier of the assigned driver.
    driver_assigned: Option<Id>,
    /// Current status of the passenger.
    passenger_status: PassengerStatus,
    /// Set of drivers that have been asked for a ride.
    asked_drivers: HashSet<Id>,
}

impl ConcuRideActor for Passenger {
    /// Returns the unique identifier for the passenger in the format "Passenger {id}".
    fn id(&self) -> String {
        format!("{} {}", PASSENGER_ROLE, self.id)
    }
}

impl TCPConcuRideActor for Passenger {}

impl TCPConcuRideActorToEvelope<Passenger> for TCPConcuRideActorContext<Passenger> {}

impl Passenger {
    /// Creates a new `Passenger` instance.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the passenger.
    /// * `origin` - Starting coordinate of the passenger.
    /// * `destination` - Destination coordinate of the passenger.
    /// * `peers` - List of peers in the network.
    pub fn new(id: Id, origin: Coordinate, destination: Coordinate, peers: Peers) -> Self {
        let tcp_layer = TcpLayer::new(id, ActorType::Passenger, peers.clone()).start();

        Self {
            id,
            position: origin,
            destination,
            peers,
            tcp_layer,
            driver_assigned: None,
            passenger_status: PassengerStatus::Init,
            asked_drivers: HashSet::new(),
        }
    }

    /// Disconnects the passenger from the network.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the actor.
    fn disconnect(&self, ctx: &mut Context<Self>) {
        self.log("⚠️ Disconnecting from the network");
        self.tcp_layer.do_send(Disconnect {});
        ctx.run_later(DISCONNECTION_INTERVAL, |_, _| {
            std::process::exit(0);
        });
    }

    /// Sends a message to another actor.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send.
    /// * `to_id` - The identifier of the recipient actor.
    /// * `to_actor_type` - The type of the recipient actor.
    fn send_message(&self, message: String, to_id: Id, to_actor_type: ActorType) {
        self.tcp_layer.do_send(SendTcpMessage {
            to_id,
            to_actor_type,
            content: message,
        });
    }

    /// Selects a random driver from the list of available drivers.
    ///
    /// # Returns
    ///
    /// An optional identifier of the selected driver.
    fn select_random_driver(&self) -> Option<Id> {
        let mut rng = rand::thread_rng();

        if let Some(drivers) = self.peers.get(&ActorType::Driver) {
            let available_drivers: Vec<_> = drivers
                .keys()
                .filter(|id| !self.asked_drivers.contains(id))
                .cloned()
                .collect();

            available_drivers.choose(&mut rng).cloned()
        } else {
            None
        }
    }

    /// Calculates the fee for the ride based on the distance.
    ///
    /// # Returns
    ///
    /// The calculated fee.
    fn calculate_fee(&self) -> usize {
        self.position.distance(&self.destination) * FEE
    }

    /// Calculates the maximum waiting time based on the estimated time.
    ///
    /// # Arguments
    ///
    /// * `estimated_time` - The estimated time in seconds.
    ///
    /// # Returns
    ///
    /// The maximum waiting time as a `Duration`.
    fn max_waiting_time(&self, estimated_time: usize) -> Duration {
        match estimated_time {
            0 => MIN_WAITING_TIME,
            _ => Duration::from_secs(estimated_time as u64 * 2),
        }
    }

    /// Creates a new ride instance.
    ///
    /// # Returns
    ///
    /// A new `Ride` instance.
    fn new_ride(&self) -> Ride {
        Ride::new(self.id, self.position.clone(), self.destination.clone())
    }

    /// Retrieves the identifier of the gateway.
    ///
    /// # Returns
    ///
    /// The identifier of the gateway.
    fn gateway_id(&self) -> Id {
        *self
            .peers
            .get(&ActorType::Gateway)
            .expect("Gateway not found")
            .keys()
            .next()
            .expect("Gateway not found")
    }
}

impl Actor for Passenger {
    type Context = Context<Self>;
}

// Ride

impl Handler<PaymentState> for Passenger {
    type Result = ();

    fn handle(&mut self, msg: PaymentState, ctx: &mut Self::Context) -> Self::Result {
        match msg.status {
            PaymentStatus::Refunded => {
                self.log("⬅️ Payment refunded");
                let fee = self.calculate_fee();

                self.log(&format!("❔ Requesting payment approval for ${}", fee));

                self.asked_drivers.clear();
                let message = PaymentRequest {
                    payer_id: self.id,
                    amount: fee,
                }
                .to_string();
                self.send_message(message, self.gateway_id(), ActorType::Gateway);
            }
            PaymentStatus::RefundedWithoutRebooking => {
                self.log("⬅️ Payment refunded");
                self.disconnect(ctx);
            }

            PaymentStatus::Rejected => {
                self.log("⛔ Payment rejected");
                self.disconnect(ctx)
            }
            PaymentStatus::Accepted => {
                self.log("✅ Payment accepted");
                ctx.address().do_send(RequestRide {
                    ride: self.new_ride(),
                });
            }
            _ => {
                self.log("❌ Error receiving the payment status");
            }
        }
    }
}

impl Handler<RequestRide> for Passenger {
    type Result = ();

    fn handle(&mut self, msg: RequestRide, ctx: &mut Self::Context) -> Self::Result {
        if let Some(id_driver) = self.select_random_driver() {
            self.log(&format!(
                "🔍 Passenger {} asks for a ride from {} to {} to driver {}",
                self.id, msg.ride.from, msg.ride.to, id_driver
            ));

            self.passenger_status = PassengerStatus::AskingDrive;
            self.asked_drivers.insert(id_driver);

            let message = msg.to_string();
            self.send_message(message, id_driver, ActorType::Driver);

            ctx.run_later(REQUEST_INTERVAL, move |actor, ctx| {
                if actor.passenger_status == PassengerStatus::AskingDrive {
                    actor.log("⌛ No response received, trying another driver.");
                    ctx.address().do_send(msg);
                }
            });
        } else {
            ctx.address().do_send(NoDriverAvailable {});
        }
    }
}

impl Handler<RequestACK> for Passenger {
    type Result = ();
    fn handle(&mut self, _: RequestACK, ctx: &mut Self::Context) -> Self::Result {
        self.passenger_status = PassengerStatus::WaitingForAssignation;
        self.log("✅ Drivers are accepting the ride");

        ctx.run_later(REQUEST_INTERVAL, move |actor, ctx| {
            if actor.passenger_status == PassengerStatus::WaitingForAssignation {
                actor.log("⌛ No response received, trying another driver.");
                ctx.address().do_send(RequestRide {
                    ride: actor.new_ride(),
                });
            }
        });
    }
}

impl Handler<NoDriverAvailable> for Passenger {
    type Result = ();
    fn handle(&mut self, _: NoDriverAvailable, ctx: &mut Self::Context) -> Self::Result {
        self.log("⛔ No drivers enabled now. Requesting refund");
        let message = ReturnPayment {
            payer_id: self.id,
            restart_payment_approval: false,
        };
        self.send_message(message.to_string(), self.gateway_id(), ActorType::Gateway);

        ctx.run_later(GATEWAY_TIMEOUT, move |actor, ctx| {
            actor.log(
                "⌛ Gateway is taking too long to respond, the refund will be processed later.",
            );
            actor.disconnect(ctx);
        });
    }
}

impl Handler<RideConfirmed> for Passenger {
    type Result = ();

    fn handle(&mut self, msg: RideConfirmed, ctx: &mut Self::Context) -> Self::Result {
        self.log(&format!(
            "✅ Driver {} has accepted the ride",
            msg.driver_id
        ));
        self.driver_assigned = Some(msg.driver_id);
        self.passenger_status = PassengerStatus::WaitingForDriverConfirmation;

        ctx.run_later(REQUEST_INTERVAL, move |actor, ctx| {
            if actor.passenger_status == PassengerStatus::WaitingForDriverConfirmation {
                actor.log("⌛ No response received, trying another driver.");
                ctx.address().do_send(RequestRide {
                    ride: actor.new_ride(),
                });
            }
        });
    }
}

impl Handler<OnTheWay> for Passenger {
    type Result = ();

    fn handle(&mut self, msg: OnTheWay, ctx: &mut Self::Context) -> Self::Result {
        self.log(&format!("🚗 Driver {} in on the way", msg.driver_id));
        self.passenger_status = PassengerStatus::WaitingDriver;
        self.driver_assigned = Some(msg.driver_id);

        let max_waiting_time = self.max_waiting_time(msg.time_to_arrive);

        ctx.run_later(max_waiting_time, move |actor, _ctx| {
            if let Some(driver_id) = actor.driver_assigned {
                if driver_id == msg.driver_id
                    && actor.passenger_status == PassengerStatus::WaitingDriver
                {
                    actor.log("⌛ Driver is taking too long to arrive, trying another driver.");
                    let message = ReturnPayment {
                        payer_id: actor.id,
                        restart_payment_approval: true,
                    };
                    actor.send_message(message.to_string(), actor.gateway_id(), ActorType::Gateway);
                }
            }
        });
    }
}

impl Handler<Arrived> for Passenger {
    type Result = ();

    fn handle(&mut self, msg: Arrived, ctx: &mut Self::Context) -> Self::Result {
        self.log(&format!(
            "✅ Getting into the car of driver {}",
            msg.driver_id
        ));
        let message = OnDrive {
            passenger_id: self.id,
        }
        .to_string();
        self.send_message(message, msg.driver_id, ActorType::Driver);
        self.passenger_status = PassengerStatus::OnRide;

        let max_waiting_time = self.max_waiting_time(msg.time_to_arrive);
        ctx.run_later(max_waiting_time, move |actor, _ctx| {
            if let Some(driver_id) = actor.driver_assigned {
                if driver_id == msg.driver_id && actor.passenger_status == PassengerStatus::OnRide {
                    actor.log("⌛ Driver is taking too long to arrive, request refund");
                    let message = ReturnPayment {
                        payer_id: actor.id,
                        restart_payment_approval: true,
                    };
                    actor.send_message(message.to_string(), actor.gateway_id(), ActorType::Gateway);
                }
            }
        });
    }
}

impl Handler<EndTrip> for Passenger {
    type Result = ();

    fn handle(&mut self, msg: EndTrip, ctx: &mut Self::Context) -> Self::Result {
        self.log(&format!(
            "✅ I have arrived to {}. Thanks driver {}",
            self.destination, msg.driver_id
        ));

        let message = AckEndTrip {}.to_string();
        self.send_message(message, msg.driver_id, ActorType::Driver);

        self.passenger_status = PassengerStatus::Arrived;

        let message = EnablePay { payer_id: self.id }.to_string();
        self.send_message(message, self.gateway_id(), ActorType::Gateway);

        self.disconnect(ctx);
    }
}

// Periodic

impl Handler<DriverState> for Passenger {
    type Result = ();

    fn handle(&mut self, msg: DriverState, _ctx: &mut Self::Context) -> Self::Result {
        match msg.status {
            DriverStatus::Free => {
                self.log(&format!(
                    "❌ Should not receive a free driver message from driver {}",
                    msg.id
                ));
            }
            DriverStatus::AcceptingRide(_) => {
                self.log(&format!(
                    "❌ Should not receive an accepting ride driver message from driver {}",
                    msg.id
                ));
            }
            DriverStatus::Busy(ride) => match self.passenger_status {
                PassengerStatus::WaitingDriver => {
                    if ride.passenger_id == self.id {
                        self.log(&format!(
                            "🗺️ Driver {} is at {} on the way to pick me up at {}",
                            msg.id, msg.position, ride.from
                        ));
                    } else {
                        self.log(&format!(
                            "❌ Should not receive a busy driver message from driver {}",
                            msg.id
                        ));
                    }
                }
                PassengerStatus::OnRide => {
                    if ride.passenger_id == self.id {
                        self.log(&format!(
                            "🗺️ Both me and driver {} are at {} on the way to {}",
                            msg.id, msg.position, ride.to
                        ));
                        self.position = msg.position;
                    } else {
                        self.log(&format!(
                            "❌ Should not receive a busy driver message from driver {}",
                            msg.id
                        ));
                    }
                }
                _ => {
                    self.log(&format!(
                        "❌ Should not receive a busy driver message from driver {}",
                        msg.id
                    ));
                }
            },
        }
    }
}

// Close Connection

impl Handler<CloseConnection> for Passenger {
    type Result = ();

    fn handle(&mut self, msg: CloseConnection, _ctx: &mut Self::Context) -> Self::Result {
        self.log(&format!(
            "⚠️ Closing connection with {} {}",
            msg.actor_type, msg.id
        ));

        if let ActorType::Driver = msg.actor_type {
            if let Some(driver_id) = self.driver_assigned {
                if driver_id == msg.id {
                    self.log("❔ Driver has disconnected, requesting refund");

                    let message = ReturnPayment {
                        payer_id: self.id,
                        restart_payment_approval: true,
                    };
                    self.send_message(message.to_string(), self.gateway_id(), ActorType::Gateway);
                }
            }
        }
    }
}

// Communication

impl Handler<ReceivedTcpMessage> for Passenger {
    type Result = ();

    fn handle(&mut self, msg: ReceivedTcpMessage, ctx: &mut Self::Context) -> Self::Result {
        if let Ok(passenger_messages) = PassengerMessages::from_string(msg.content.clone()) {
            match passenger_messages {
                PassengerMessages::RequestRide(request_ride) => {
                    ctx.address().do_send(request_ride);
                }
                PassengerMessages::RequestACK(request_ack) => {
                    ctx.address().do_send(request_ack);
                }
                PassengerMessages::RideConfirmed(ride_confirmed) => {
                    ctx.address().do_send(ride_confirmed);
                }
                _ => self.log(&format!(
                    "❌ Error when receiving an unexpected type of message for passenger {:?}",
                    msg.content
                )),
            }
            return;
        }

        if let Ok(driver_message) = DriverMessages::from_string(msg.content.clone()) {
            match driver_message {
                DriverMessages::OnTheWay(on_the_way) => {
                    ctx.address().do_send(on_the_way);
                }
                DriverMessages::Arrived(arrived) => {
                    ctx.address().do_send(arrived);
                }
                DriverMessages::EndTrip(end_trip) => {
                    ctx.address().do_send(end_trip);
                }
                DriverMessages::NoDriverAvailable(no_driver_available) => {
                    ctx.address().do_send(no_driver_available);
                }
                DriverMessages::DriverState(driver_state) => {
                    ctx.address().do_send(driver_state);
                }
                _ => self.log(&format!(
                    "❌ Error when receiving an unexpected type of driver message for the passenger: {:?}",
                    msg.content
                )),
            }
            return;
        }

        if let Ok(gateway_message) = GatewayMessage::from_string(msg.content.clone()) {
            match gateway_message {
                GatewayMessage::PaymentState(payment_state) => ctx.address().do_send(payment_state),
                _ => self.log(
                    "❌ Error receiving an unexpected message from the passenger for the passenger",
                ),
            }
            return;
        }

        self.log(&format!("❌ Error parsing message: {:?}", msg));
    }
}

// Initialization

impl Handler<Initialize> for Passenger {
    type Result = ();

    fn handle(&mut self, _msg: Initialize, ctx: &mut Self::Context) -> Self::Result {
        self.tcp_layer.do_send(AssignActor {
            actor: ctx.address().clone(),
        });

        self.tcp_layer.do_send(Initialize {});

        let fee = self.calculate_fee();
        self.log(&format!("✅ Requesting payment approval for ${}", fee));

        let message = PaymentRequest {
            payer_id: self.id,
            amount: fee,
        }
        .to_string();
        self.send_message(message, self.gateway_id(), ActorType::Gateway);

        ctx.run_later(GATEWAY_TIMEOUT, move |actor, ctx| {
            if actor.passenger_status == PassengerStatus::Init {
                actor.log("⌛ Timeout waiting for gateway connection, disconnecting.");
                actor.disconnect(ctx);
            }
        });
    }
}
