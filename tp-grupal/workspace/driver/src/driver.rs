use std::{
    cmp::Ordering,
    collections::{HashMap, VecDeque},
    time::Duration,
};

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
    leader::{Leader, Response},
    messages::{
        concu_ride_messages::Initialize,
        driver_messages::{
            Arrived, DriverMessages, DriverState, Election, ElectionACK, EndTrip, InfoACK,
            LeaderData, NewLeader, NoDriverAvailable, OnTheWay, ResetRide, ResponseDrive, SendInfo,
            Travelling,
        },
        gateway_messages::{
            AssignPayee, FreePayment, GatewayMessage, PaymentConfirmation, PaymentState,
            RequestPaymentInfo,
        },
        leader_messages::{DriveOffer, InitDrive, LeaderMessages, LookForDriver},
        passenger_messages::{
            AckEndTrip, OnDrive, PassengerMessages, RequestACK, RequestRide, RideConfirmed,
        },
        tcp_messages::{AssignActor, CloseConnection, ReceivedTcpMessage, SendTcpMessage},
    },
    payment::PaymentStatus,
    ride::RideStatus,
    Id, Peers,
};

use actix::{Actor, Addr, AsyncContext, Context, Handler};
use actix_async_handler::async_handler;
use rand::Rng;

const DRIVER_ROLE: &str = "Driver";

const GATEWAY_TIMEOUT: Duration = Duration::from_secs(3);

const ELECT_TIMEOUT: Duration = Duration::from_secs(3);

const INFO_TIMEOUT: Duration = Duration::from_secs(2);
const INFO_INTERVAL: Duration = Duration::from_secs(4);

const LOOK_FOR_DRIVER_TIMEOUT: Duration = Duration::from_secs(5);
const ACCEPTING_RIDE_TIMEOUT: Duration = Duration::from_secs(5);

const PAYMENT_TIMEOUT: Duration = Duration::from_secs(3);
const TRAVEL_INTERVAL: Duration = Duration::from_secs(4);

/// Represents a driver in the ConcuRide system.
pub struct Driver {
    /// Unique identifier for the driver.
    id: Id,
    /// Current position of the driver.
    position: Coordinate,
    /// Optional identifier of the leader.
    leader: Option<Id>,
    /// Optional data of the leader.
    leader_data: Option<Leader>,
    /// List of peers in the network.
    peers: Peers,
    /// TCP layer address for communication.
    tcp_layer: Addr<TcpLayer<Driver>>,
    /// Indicates if an election is in progress.
    election_in_progress: bool,
    /// Number of acknowledgments received during an election.
    election_acks: usize,
    /// Indicates if information acknowledgments have been received.
    info_acks: bool,
    /// Current status of the driver.
    driver_status: DriverStatus,
    /// Queue of messages to be sent.
    queued_messages: VecDeque<String>,
    /// Accept rate for the driver.
    driver_accept_rate: f64,
}

impl ConcuRideActor for Driver {
    /// Returns the unique identifier for the driver in the format "Driver {id}".
    fn id(&self) -> String {
        format!("{} {}", DRIVER_ROLE, self.id)
    }
}

impl TCPConcuRideActor for Driver {}

impl TCPConcuRideActorToEvelope<Driver> for TCPConcuRideActorContext<Driver> {}

impl Driver {
    /// Creates a new `Driver` instance.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the driver.
    /// * `initial_position` - Initial position of the driver.
    /// * `peers` - List of peers in the network.
    ///
    /// # Returns
    ///
    /// A new `Driver` instance.
    pub fn new(
        id: Id,
        initial_position: Coordinate,
        peers: Peers,
        driver_accept_rate: f64,
    ) -> Self {
        let tcp_layer = TcpLayer::new(id, ActorType::Driver, peers.clone()).start();

        Self {
            id,
            position: initial_position,
            leader: None,
            leader_data: None,
            peers,
            tcp_layer,
            election_in_progress: false,
            election_acks: 0,
            info_acks: false,
            driver_status: DriverStatus::Free,
            queued_messages: VecDeque::new(),
            driver_accept_rate,
        }
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

    /// Broadcasts the new leader to all drivers.
    fn broadcast_new_leader(&mut self) {
        self.log("🛜 Broadcasting new leader");
        if let Some(actor_driver) = self.peers.get(&ActorType::Driver) {
            for id in actor_driver.keys() {
                let message = NewLeader { id: self.id }.to_string();
                self.send_message(message, *id, ActorType::Driver);
            }
        }
    }

    /// Sends an election acknowledgment to a driver.
    ///
    /// # Arguments
    ///
    /// * `from_id` - The identifier of the driver to send the acknowledgment to.
    fn send_election_ack(&mut self, from_id: Id) {
        if from_id == self.id {
            return;
        }

        self.log(&format!("✅ Sending ACK to driver {}", from_id));

        let message = ElectionACK {}.to_string();
        self.send_message(message, from_id, ActorType::Driver);
    }

    /// Sends a message to the leader.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send.
    /// * `ctx` - The context of the actor.
    fn send_message_to_leader(&mut self, message: String, ctx: &mut Context<Self>) {
        if let Some(leader) = self.leader {
            self.send_message(message, leader, ActorType::Driver);
        } else {
            self.queue_message(message);
            ctx.address().do_send(Election { id: self.id });
        }
    }

    /// Sends the driver status to the leader.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the actor.
    fn send_status_to_leader(&mut self, ctx: &mut Context<Self>) {
        if let Some(leader) = self.leader {
            let message = DriverState {
                id: self.id,
                position: self.position.clone(),
                status: self.driver_status.clone(),
            };
            self.send_message(message.to_string(), leader, ActorType::Driver);

            self.info_acks = false;
            ctx.run_later(INFO_TIMEOUT, |actor, _ctx| {
                if !actor.info_acks {
                    actor.log("Leader didn't ACK info message");
                    _ctx.address().do_send(Election { id: actor.id });
                }
            });
        } else {
            ctx.address().do_send(Election { id: self.id });
        }
    }

    /// Queues a message to be sent later.
    ///
    /// # Arguments
    ///
    /// * `content` - The content of the message to queue.
    fn queue_message(&mut self, content: String) {
        self.queued_messages.push_back(content);
    }

    /// Sends all queued messages to the leader.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of the actor.
    fn send_queued_messages(&mut self, ctx: &mut Context<Self>) {
        while let Some(content) = self.queued_messages.pop_front() {
            self.send_message_to_leader(content, ctx);
        }
    }

    /// Determines whether the driver should accept a ride.
    ///
    /// # Returns
    ///
    /// `true` if the ride should be accepted, `false` otherwise.
    fn should_accept_ride(&self) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() < self.driver_accept_rate
    }

    /// Determines the direction to move from origin to destination.
    ///
    /// # Arguments
    ///
    /// * `origin` - The origin coordinate.
    /// * `destination` - The destination coordinate.
    ///
    /// # Returns
    ///
    /// The direction to move.
    fn direction_to(&self, origin: i32, destination: i32) -> i32 {
        match origin.cmp(&destination) {
            Ordering::Less => 1,
            Ordering::Equal => 0,
            Ordering::Greater => -1,
        }
    }

    /// Generates a random direction for the driver to move.
    ///
    /// # Returns
    ///
    /// A new `Coordinate` representing the new position.
    fn random_direction(&self) -> Coordinate {
        let mut rng = rand::thread_rng();
        let prob = rng.gen_range(0..4);
        match prob {
            0 => Coordinate::new(self.position.x + 1, self.position.y),
            1 => Coordinate::new(self.position.x - 1, self.position.y),
            2 => Coordinate::new(self.position.x, self.position.y + 1),
            3 => Coordinate::new(self.position.x, self.position.y - 1),
            _ => Coordinate::new(self.position.x, self.position.y),
        }
    }

    /// Advances the driver towards the destination.
    ///
    /// # Arguments
    ///
    /// * `destination` - The destination coordinate.
    fn advance_to(&mut self, destination: Coordinate) {
        if self.position.x != destination.x {
            self.position.x += self.direction_to(self.position.x, destination.x);
        } else if self.position.y != destination.y {
            self.position.y += self.direction_to(self.position.y, destination.y);
        }
    }

    /// Calculates the arrival time to the destination.
    ///
    /// # Arguments
    ///
    /// * `destination` - The destination coordinate.
    ///
    /// # Returns
    ///
    /// The arrival time in seconds.
    fn arrival_time(&self, destination: Coordinate) -> usize {
        self.position.distance(&destination) * TRAVEL_INTERVAL.as_secs() as usize
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

impl Actor for Driver {
    type Context = Context<Self>;
}

// Election

#[async_handler]
impl Handler<Election> for Driver {
    type Result = ();

    fn handle(&mut self, msg: Election, _ctx: &mut Self::Context) -> Self::Result {
        match msg.id.cmp(&self.id) {
            Ordering::Less => {
                self.log(&format!("🔎 Received election from driver {}", msg.id));
            }
            Ordering::Greater => {
                self.log(&format!(
                    "❌ Received election from non expected driver {}",
                    msg.id
                ));
                return;
            }
            _ => {}
        }

        if self.election_in_progress {
            self.send_election_ack(msg.id);
            return;
        }

        self.log("🔎 Starting election");
        self.election_in_progress = true;
        self.election_acks = 0;

        if let Some(leader) = self.leader {
            if leader != self.id {
                self.leader = None;
            }
        }

        self.send_election_ack(msg.id);

        if let Some(peer_map) = self.peers.get(&ActorType::Driver) {
            for id in peer_map.keys() {
                if *id > self.id {
                    self.log(&format!("🔎 Sending election message to {}", id));
                    let message = Election { id: self.id }.to_string();
                    self.send_message(message, *id, ActorType::Driver);
                }
            }
        }

        _ctx.run_later(ELECT_TIMEOUT, |actor, _ctx| {
            if actor.election_in_progress {
                if actor.election_acks == 0 {
                    actor.log("📪 No election ACKs received");
                    actor.broadcast_new_leader();
                } else {
                    actor.log(&format!(
                        "✅ Received {} election ACKs, waiting for new leader",
                        actor.election_acks
                    ));
                }
            }
        });
    }
}

impl Handler<ElectionACK> for Driver {
    type Result = ();

    fn handle(&mut self, _msg: ElectionACK, _ctx: &mut Self::Context) -> Self::Result {
        self.log("✅ Received Election ACK");
        self.election_acks += 1;
    }
}

impl Handler<NewLeader> for Driver {
    type Result = ();

    fn handle(&mut self, msg: NewLeader, ctx: &mut Self::Context) -> Self::Result {
        if msg.id == self.id {
            self.log("👑 I'm the new leader");

            if self.leader_data.is_none() {
                self.leader_data = Some(Leader::new());
            }
        } else {
            self.log(&format!("👑 Driver {} is the new leader", msg.id));

            if let Some(leader_data) = self.leader_data.as_ref() {
                self.log(&format!("📦 Passing leader data to new leader {}", msg.id));

                let message = LeaderData {
                    active_rides: leader_data.active_rides.clone(),
                    drivers_states: leader_data.drivers_states.clone(),
                    asked_drivers: leader_data.asked_drivers.clone(),
                }
                .to_string();
                self.send_message(message, msg.id, ActorType::Driver);
            }

            self.leader_data = None;
        }
        self.leader = Some(msg.id);
        self.election_in_progress = false;
        self.send_queued_messages(ctx);
        self.send_status_to_leader(ctx);
    }
}

// Leader

impl Handler<DriverState> for Driver {
    type Result = ();

    fn handle(&mut self, msg: DriverState, _ctx: &mut Self::Context) -> Self::Result {
        self.log(&format!(
            "📌 Driver {} is at position {}",
            msg.id, msg.position
        ));

        if let Some(leader) = self.leader_data.as_mut() {
            if let DriverStatus::Busy(ride) = msg.status.clone() {
                leader.active_rides.insert(ride.passenger_id, ride);
            }

            leader
                .drivers_states
                .insert(msg.id, (msg.position, msg.status));
        }

        let message = InfoACK {}.to_string();
        self.send_message(message, msg.id, ActorType::Driver);
    }
}

impl Handler<LookForDriver> for Driver {
    type Result = ();

    fn handle(&mut self, msg: LookForDriver, ctx: &mut Self::Context) -> Self::Result {
        if self.leader_data.is_some() {
            match msg.ride.status {
                RideStatus::Requested => {
                    self.log(&format!(
                        "❔ Checking if the payment was accepted for passenger {}",
                        msg.ride.passenger_id
                    ));
                }
                RideStatus::LookingForDriver => {
                    self.log(&format!(
                        "🔍 Looking for driver for passenger {}",
                        msg.ride.passenger_id
                    ));
                }
                _ => {
                    self.log("❌ Error: Ride status not allowed");
                    return;
                }
            }
        } else {
            self.log("⏩ I'm not the leader. Sending message to the leader");

            let message = msg.to_string();
            self.send_message_to_leader(message, ctx);
        }

        let mut ride = msg.ride.clone();
        if let Some(leader) = self.leader_data.as_mut() {
            leader
                .active_rides
                .insert(msg.ride.passenger_id, msg.ride.clone());

            if ride.status == RideStatus::Requested {
                let message = RequestPaymentInfo {
                    payer_id: msg.ride.passenger_id,
                    requester_id: self.id,
                }
                .to_string();

                self.send_message(message, self.gateway_id(), ActorType::Gateway);
                ctx.run_later(GATEWAY_TIMEOUT, move |actor, ctx| {
                    if let Some(leader) = &actor.leader_data {
                        if let Some(ride) = leader.active_rides.get(&msg.ride.passenger_id) {
                            if ride.status == RideStatus::Requested {
                                ctx.address().do_send(msg);
                            }
                        }
                    }
                });
                return;
            }

            ride.status = RideStatus::LookingForDriver;

            if let Some(nearest_driver) = leader.search_nearest_driver(ride.clone()) {
                leader
                    .asked_drivers
                    .entry(msg.ride.passenger_id)
                    .or_insert_with(HashMap::new)
                    .insert(nearest_driver, Response::Pending);

                self.log(&format!("❔ I offer the trip to {}", nearest_driver));

                let message = DriveOffer { ride }.to_string();
                self.send_message(message, nearest_driver, ActorType::Driver);

                ctx.run_later(LOOK_FOR_DRIVER_TIMEOUT, move |actor, ctx| {
                    if let Some(leader) = actor.leader_data.as_mut() {
                        if let Some(ride) = leader.asked_drivers.get(&msg.ride.passenger_id) {
                            if let Some(Response::Pending) = ride.get(&nearest_driver) {
                                actor.log(&format!(
                                    "⛔ Driver {} don't accept the ride",
                                    nearest_driver
                                ));
                                ctx.address().do_send(msg);
                            }
                        }
                    }
                });
            } else {
                self.log(&format!(
                    "⛔ No drivers available. Notifying to the passenger {}",
                    msg.ride.passenger_id
                ));
                let message = NoDriverAvailable {}.to_string();
                self.send_message(message, msg.ride.passenger_id, ActorType::Passenger);

                let message = ResetRide {
                    id_passenger: msg.ride.passenger_id,
                };
                ctx.address().do_send(message);
            }
        }
    }
}

impl Handler<PaymentState> for Driver {
    type Result = ();

    fn handle(&mut self, msg: PaymentState, ctx: &mut Self::Context) -> Self::Result {
        self.log(&format!(
            "✅ Payment from passenger {} is {}",
            msg.payer_id, msg.status
        ));

        if let PaymentStatus::Accepted = msg.status {
            if let Some(leader) = self.leader_data.as_mut() {
                if let Some(ride) = leader.active_rides.get_mut(&msg.payer_id) {
                    if ride.status == RideStatus::Requested {
                        ride.status = RideStatus::LookingForDriver;

                        let message = LookForDriver { ride: ride.clone() };
                        ctx.address().do_send(message);
                    }
                } else {
                    self.log("❌ Error: Ride not found");
                }
            } else {
                self.log("⚠️ Only the leader can receive a payment status");
            }
        } else {
            self.log("❌ Error: Should not receive a payment status different from Accepted");
        }
    }
}

impl Handler<ResetRide> for Driver {
    type Result = ();
    fn handle(&mut self, msg: ResetRide, ctx: &mut Self::Context) -> Self::Result {
        self.log(&format!(
            "✅ Deleting the trip of passenger {}",
            msg.id_passenger
        ));
        if let Some(leader) = self.leader_data.as_mut() {
            leader.active_rides.remove(&msg.id_passenger);
            leader.asked_drivers.remove(&msg.id_passenger);
        } else {
            self.log("⏩ Only the leader can reset a ride");
            self.send_message_to_leader(msg.to_string(), ctx);
        }
    }
}

impl Handler<ResponseDrive> for Driver {
    type Result = ();
    fn handle(&mut self, msg: ResponseDrive, ctx: &mut Self::Context) -> Self::Result {
        if let Some(leader) = self.leader_data.as_mut() {
            leader
                .active_rides
                .insert(msg.ride.passenger_id, msg.ride.clone());

            if let Some(driver) = msg.ride.driver {
                if let Some(response) = leader
                    .asked_drivers
                    .get_mut(&msg.ride.passenger_id)
                    .and_then(|drivers| drivers.get_mut(&driver))
                {
                    *response = Response::Accept;
                }

                self.log(&format!(
                    "✅ Driver {} accepted the ride for passenger {}",
                    driver, msg.ride.passenger_id
                ));

                let message = AssignPayee {
                    payer_id: msg.ride.passenger_id,
                    payee_id: driver,
                }
                .to_string();
                self.send_message(message, self.gateway_id(), ActorType::Gateway);

                let message = RideConfirmed { driver_id: driver }.to_string();
                self.send_message(message, msg.ride.passenger_id, ActorType::Passenger);

                let message = InitDrive {
                    ride: msg.ride.clone(),
                }
                .to_string();
                self.send_message(message, driver, ActorType::Driver);
            } else {
                if let Some(response) = leader
                    .asked_drivers
                    .get_mut(&msg.ride.passenger_id)
                    .and_then(|drivers| drivers.get_mut(&msg.driver_id))
                {
                    *response = Response::Reject;
                }

                self.log(&format!(
                    "⛔ Driver rejected the ride for passenger {}",
                    msg.ride.passenger_id
                ));

                let message = LookForDriver {
                    ride: msg.ride.clone(),
                }
                .to_string();

                self.send_message_to_leader(message, ctx);
            }
        }
    }
}

impl Handler<LeaderData> for Driver {
    type Result = ();

    fn handle(&mut self, msg: LeaderData, _ctx: &mut Self::Context) -> Self::Result {
        self.log("📦 Received leader data");

        if let Some(leader) = self.leader_data.as_mut() {
            leader.active_rides.extend(msg.active_rides);
            leader.drivers_states.extend(msg.drivers_states);
            for (key, value) in msg.asked_drivers {
                leader.asked_drivers.insert(key, value);
            }
        }
    }
}

impl Handler<FreePayment> for Driver {
    type Result = ();

    fn handle(&mut self, msg: FreePayment, ctx: &mut Self::Context) -> Self::Result {
        match self.leader {
            Some(leader) if leader == self.id => {
                self.send_message(msg.to_string(), self.gateway_id(), ActorType::Gateway);
            }
            _ => {
                self.send_message_to_leader(msg.to_string(), ctx);
            }
        }
    }
}

// Periodic messages

#[async_handler]
impl Handler<SendInfo> for Driver {
    type Result = ();

    async fn handle(&mut self, _msg: SendInfo, _ctx: &mut Self::Context) -> Self::Result {
        self.log(&format!(
            "📌 I'm at position {} with status {}",
            self.position, self.driver_status
        ));

        match self.leader {
            Some(leader) if leader == self.id => {
                if let Some(leader) = self.leader_data.as_mut() {
                    leader
                        .drivers_states
                        .insert(self.id, (self.position.clone(), self.driver_status.clone()));
                }
            }

            _ => self.send_status_to_leader(_ctx),
        }

        match &self.driver_status {
            DriverStatus::Free => {}
            DriverStatus::AcceptingRide(_) => {}
            DriverStatus::Busy(ride) => {
                let passenger = ride.passenger_id;

                let message = DriverState {
                    id: self.id,
                    position: self.position.clone(),
                    status: self.driver_status.clone(),
                }
                .to_string();
                self.send_message(message, passenger, ActorType::Passenger);
            }
        }

        _ctx.run_later(INFO_INTERVAL, |_, ctx| {
            ctx.address().do_send(SendInfo {});
        });
    }
}

impl Handler<InfoACK> for Driver {
    type Result = ();

    fn handle(&mut self, _msg: InfoACK, _ctx: &mut Self::Context) -> Self::Result {
        self.info_acks = true;
    }
}

impl Handler<Travelling> for Driver {
    type Result = ();

    fn handle(&mut self, _msg: Travelling, ctx: &mut Self::Context) -> Self::Result {
        let advance_to: Coordinate;
        if let DriverStatus::Busy(ride) = &mut self.driver_status {
            match ride.status {
                RideStatus::GoingToPickup => {
                    advance_to = ride.from.clone();
                }
                RideStatus::InProgress => {
                    advance_to = ride.to.clone();
                }
                _ => {
                    advance_to = self.position.clone();
                }
            }
        } else {
            advance_to = self.random_direction();
        }

        self.advance_to(advance_to);

        if let DriverStatus::Busy(ride) = &mut self.driver_status {
            match ride.status {
                RideStatus::GoingToPickup => {
                    if self.position == ride.from {
                        ride.status = RideStatus::WaitingToStart;
                    }
                }
                RideStatus::InProgress => {
                    if self.position == ride.to {
                        ride.status = RideStatus::Completed;
                    }
                }
                _ => {}
            }
        }

        if let DriverStatus::Busy(ride) = &self.driver_status {
            match ride.status {
                RideStatus::WaitingToStart => {
                    let arriving_time = self.arrival_time(ride.to.clone());
                    let message = Arrived {
                        driver_id: self.id,
                        time_to_arrive: arriving_time,
                    }
                    .to_string();
                    self.send_message(message, ride.passenger_id, ActorType::Passenger);

                    ctx.run_later(INFO_INTERVAL, |actor, ctx| {
                        if let DriverStatus::Busy(ride) = &mut actor.driver_status {
                            if ride.status == RideStatus::WaitingToStart {
                                let message = FreePayment {
                                    payer_id: ride.passenger_id,
                                }
                                .to_string();
                                actor.send_message_to_leader(message, ctx);
                            }
                        }
                    });
                }
                RideStatus::Completed => {
                    let message = EndTrip { driver_id: self.id }.to_string();
                    self.send_message(message, ride.passenger_id, ActorType::Passenger);

                    ctx.run_later(INFO_INTERVAL, |actor, ctx| {
                        if let DriverStatus::Busy(ride) = &mut actor.driver_status {
                            if ride.status == RideStatus::Completed {
                                let message = FreePayment {
                                    payer_id: ride.passenger_id,
                                }
                                .to_string();
                                actor.send_message_to_leader(message, ctx);
                            }
                        }
                    });
                }
                _ => {}
            }
        }

        ctx.run_later(TRAVEL_INTERVAL, move |actor, _ctx| {
            actor.tcp_layer.do_send(SendTcpMessage {
                to_id: actor.id,
                to_actor_type: ActorType::Driver,
                content: Travelling {}.to_string(),
            });
        });
    }
}

// Ride

impl Handler<RequestRide> for Driver {
    type Result = ();
    fn handle(&mut self, msg: RequestRide, ctx: &mut Self::Context) -> Self::Result {
        self.log(&format!(
            "🔍 Request for ride from passenger {}",
            msg.ride.passenger_id
        ));

        let message = RequestACK {}.to_string();
        self.send_message(message, msg.ride.passenger_id, ActorType::Passenger);

        let message = LookForDriver { ride: msg.ride }.to_string();
        self.send_message_to_leader(message, ctx);
    }
}

impl Handler<DriveOffer> for Driver {
    type Result = ();
    fn handle(&mut self, msg: DriveOffer, ctx: &mut Self::Context) -> Self::Result {
        self.log(&format!(
            "❔ Offer for ride from passenger {}",
            msg.ride.passenger_id
        ));

        let mut ride = msg.ride;

        if let DriverStatus::Free = self.driver_status {
            if self.should_accept_ride() {
                ride.driver = Some(self.id);
                self.driver_status = DriverStatus::AcceptingRide(ride.clone());

                let offered_ride = ride.clone();

                self.log("✅ I'm accepting the ride");

                ctx.run_later(ACCEPTING_RIDE_TIMEOUT, move |actor, ctx| {
                    if let DriverStatus::AcceptingRide(accepted_ride) = &actor.driver_status {
                        if accepted_ride == &offered_ride {
                            actor.driver_status = DriverStatus::Free;
                            actor.log("⌛ Timeout accepting the ride");

                            let message = ResetRide {
                                id_passenger: offered_ride.passenger_id,
                            }
                            .to_string();
                            actor.send_message_to_leader(message, ctx);
                            actor.send_status_to_leader(ctx);
                        }
                    }
                });
            } else {
                self.log("⛔ I'm not accepting the ride");
            }
        } else {
            self.log("⛔ I can't accept the ride");
        }

        let message = ResponseDrive {
            driver_id: self.id,
            ride,
        }
        .to_string();
        self.send_message_to_leader(message, ctx);
    }
}

impl Handler<InitDrive> for Driver {
    type Result = ();
    fn handle(&mut self, msg: InitDrive, ctx: &mut Self::Context) -> Self::Result {
        self.log(&format!(
            "✅ Start communication with the passenger {}",
            msg.ride.passenger_id
        ));

        let arriving = self.position.distance(&msg.ride.from) * TRAVEL_INTERVAL.as_secs() as usize;

        let message = OnTheWay {
            driver_id: self.id,
            time_to_arrive: arriving,
        }
        .to_string();
        self.send_message(message, msg.ride.passenger_id, ActorType::Passenger);

        self.log(&format!("🚗 Arriving in {} seconds", arriving));

        let mut ride = msg.ride.clone();
        ride.status = RideStatus::GoingToPickup;
        self.driver_status = DriverStatus::Busy(ride);

        self.send_status_to_leader(ctx);
    }
}

impl Handler<OnDrive> for Driver {
    type Result = ();
    fn handle(&mut self, _msg: OnDrive, ctx: &mut Self::Context) -> Self::Result {
        self.log("✅ The passenger got into the car. Start the trip");

        let arrival_time = match &self.driver_status {
            DriverStatus::Busy(ride) => self.arrival_time(ride.to.clone()),
            _ => {
                self.log("❌ An error occurred, the driver was not aware of the trip");
                return;
            }
        };

        match &mut self.driver_status {
            DriverStatus::Busy(ride) => {
                ride.status = RideStatus::InProgress;
                self.log(&format!(
                    "🚗 Arriving destination in {} seconds",
                    arrival_time
                ));
            }
            _ => self.log("❌ An error occurred, the driver was not aware of the trip"),
        }

        self.send_status_to_leader(ctx);
    }
}

impl Handler<AckEndTrip> for Driver {
    type Result = ();

    fn handle(&mut self, _msg: AckEndTrip, ctx: &mut Self::Context) -> Self::Result {
        let completed_ride = match &self.driver_status {
            DriverStatus::Busy(ride) => ride.clone(),
            _ => {
                self.log("✅ Payment recieved. Have a nice day!");
                return;
            }
        };

        ctx.run_later(PAYMENT_TIMEOUT, move |actor, _ctx| {
            if let DriverStatus::Busy(actual_ride) = &actor.driver_status {
                if actual_ride == &completed_ride {
                    actor.log("⌛ Payment timeout.");
                    let message = FreePayment {
                        payer_id: completed_ride.passenger_id,
                    }
                    .to_string();
                    actor.send_message_to_leader(message, _ctx);
                }
            }
        });
    }
}

impl Handler<PaymentConfirmation> for Driver {
    type Result = ();

    fn handle(&mut self, msg: PaymentConfirmation, ctx: &mut Self::Context) -> Self::Result {
        self.log(&format!("💵 ${} payment received", msg.amount));

        let ride = match &self.driver_status {
            DriverStatus::Busy(ride) => ride,
            _ => {
                self.log("❌ Error: Trip end error");
                return;
            }
        }
        .clone();

        self.driver_status = DriverStatus::Free;

        let message = ResetRide {
            id_passenger: ride.passenger_id,
        }
        .to_string();
        self.send_message_to_leader(message, ctx);
        self.send_status_to_leader(ctx);
    }
}

// Close Connection

impl Handler<CloseConnection> for Driver {
    type Result = ();

    fn handle(&mut self, msg: CloseConnection, ctx: &mut Self::Context) -> Self::Result {
        self.log(&format!(
            "⚠️ Closing connection with {} {}",
            msg.actor_type, msg.id
        ));

        match msg.actor_type {
            ActorType::Driver => {
                if let Some(leader) = self.leader {
                    if leader == msg.id {
                        ctx.address().do_send(Election { id: self.id });
                    }
                } else {
                    ctx.address().do_send(Election { id: self.id });
                }
            }
            ActorType::Passenger => {
                if let DriverStatus::Busy(ride) = &self.driver_status {
                    if ride.passenger_id == msg.id {
                        match ride.status {
                            RideStatus::Requested => {}
                            RideStatus::LookingForDriver => {}
                            RideStatus::Completed => {}
                            _ => {
                                self.log("❔ The passenger disconnected. Requesting payment");

                                let message = FreePayment {
                                    payer_id: ride.passenger_id,
                                }
                                .to_string();
                                self.send_message_to_leader(message, ctx);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

// Communication

impl Handler<ReceivedTcpMessage> for Driver {
    type Result = ();

    fn handle(&mut self, msg: ReceivedTcpMessage, ctx: &mut Self::Context) -> Self::Result {
        if let Ok(driver_message) = DriverMessages::from_string(msg.content.clone()) {
            match driver_message {
                DriverMessages::DriverState(driver_state) => {
                    ctx.address().do_send(driver_state);
                }
                DriverMessages::Election(election) => {
                    ctx.address().do_send(election);
                }
                DriverMessages::NewLeader(coord) => {
                    ctx.address().do_send(coord);
                }
                DriverMessages::InfoACK(info_ack) => {
                    ctx.address().do_send(info_ack);
                }
                DriverMessages::ElectionACK(election_ack) => {
                    ctx.address().do_send(election_ack);
                }
                DriverMessages::SendInfo(send_info) => {
                    ctx.address().do_send(send_info);
                }
                DriverMessages::ResetRide(reset_drive) => {
                    ctx.address().do_send(reset_drive);
                }
                DriverMessages::ResposeDrive(accept_drive) => {
                    ctx.address().do_send(accept_drive);
                }
                DriverMessages::Travelling(travelling) => {
                    ctx.address().do_send(travelling);
                }
                DriverMessages::LeaderData(leader_data) => ctx.address().do_send(leader_data),

                _ => {
                    self.log(&format!(
                        "❌ Error when receiving an unexpected type of message for the driver: {}",
                        msg.content
                    ));
                }
            }
            return;
        } else if let Ok(leader_message) = LeaderMessages::from_string(msg.content.clone()) {
            match leader_message {
                LeaderMessages::LookForDriver(look_for_driver) => {
                    ctx.address().do_send(look_for_driver)
                }
                LeaderMessages::DriveOffer(drive_offer) => ctx.address().do_send(drive_offer),
                LeaderMessages::InitDrive(init_drive) => ctx.address().do_send(init_drive),
            }

            return;
        } else if let Ok(passanger_message) = PassengerMessages::from_string(msg.content.clone()) {
            match passanger_message {
                PassengerMessages::RequestRide(request_ride) => {
                    ctx.address().do_send(request_ride);
                }
                PassengerMessages::OnDrive(ondrive) => {
                    ctx.address().do_send(ondrive);
                }
                PassengerMessages::AckEndTrip(ack_end_trip) => ctx.address().do_send(ack_end_trip),
                _ => {
                    self.log("❌ Error when receiving an unexpected type message for the driver");
                }
            }
            return;
        } else if let Ok(gateway_message) = GatewayMessage::from_string(msg.content.clone()) {
            match gateway_message {
                GatewayMessage::PaymentState(payment_state) => ctx.address().do_send(payment_state),
                GatewayMessage::PaymentConfirmation(payment_confirmation) => {
                    ctx.address().do_send(payment_confirmation)
                }
                GatewayMessage::FreePayment(free_payment) => ctx.address().do_send(free_payment),
                _ => self.log(&format!(
                    "❌ Error when receiving an unexpected type message for the driver: {}",
                    msg.content
                )),
            }
            return;
        }

        self.log("❌ Error parsing message");
    }
}

// Initialization

impl Handler<Initialize> for Driver {
    type Result = ();

    fn handle(&mut self, _msg: Initialize, ctx: &mut Self::Context) -> Self::Result {
        ctx.address().do_send(SendInfo {});
        ctx.address().do_send(Travelling {});

        self.tcp_layer.do_send(AssignActor {
            actor: ctx.address().clone(),
        });

        self.tcp_layer.do_send(Initialize {});
    }
}
