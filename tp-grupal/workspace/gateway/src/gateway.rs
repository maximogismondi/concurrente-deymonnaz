use actix::{Actor, Addr, AsyncContext, Context, Handler};
use common::{
    actors::{
        actor_type::ActorType,
        concu_ride_actor::ConcuRideActor,
        tcp::{
            tcp_layer::TcpLayer, TCPConcuRideActor, TCPConcuRideActorContext,
            TCPConcuRideActorToEvelope,
        },
    },
    messages::{
        concu_ride_messages::Initialize,
        gateway_messages::{
            AssignPayee, EnablePay, FreePayment, GatewayMessage, PaymentConfirmation,
            PaymentRequest, PaymentState, RequestPaymentInfo, ReturnPayment,
        },
        tcp_messages::{AssignActor, CloseConnection, ReceivedTcpMessage, SendTcpMessage},
    },
    payment::{Payment, PaymentStatus},
    Id,
};

use rand::Rng;
use std::collections::HashMap;

const GATEWAY_ROLE: &str = "Gateway";

/// Represents a gateway in the ConcuRide system.
pub(crate) struct Gateway {
    /// A map of payments associated with their IDs.
    pub payments: HashMap<Id, Payment>,
    /// TCP layer address for communication.
    pub tcp_layer: Addr<TcpLayer<Gateway>>,
    /// The rate at which the gateway accepts payments.
    pub gateway_accept_rate: f64,
}

impl ConcuRideActor for Gateway {
    /// Returns the unique identifier for the gateway in the format "Gateway 1".
    fn id(&self) -> String {
        format!("{} 1", GATEWAY_ROLE)
    }
}

impl TCPConcuRideActor for Gateway {}

impl TCPConcuRideActorToEvelope<Gateway> for TCPConcuRideActorContext<Gateway> {}

impl Gateway {
    /// Creates a new `Gateway` instance.
    ///
    /// # Arguments
    ///
    /// * `peers` - A map of peers in the network.
    ///
    /// # Returns
    ///
    /// A new `Gateway` instance.
    pub fn new(peers: HashMap<ActorType, HashMap<Id, String>>, gateway_accept_rate: f64) -> Self {
        let tcp_layer = TcpLayer::new(1, ActorType::Gateway, peers).start();

        Self {
            payments: HashMap::new(),
            tcp_layer,
            gateway_accept_rate,
        }
    }

    /// Determines whether the gateway should accept a payment.
    ///
    /// # Returns
    ///
    /// `true` if the payment should be accepted, `false` otherwise.
    fn should_accept_payment(&self) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() < self.gateway_accept_rate
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
}

impl Actor for Gateway {
    type Context = Context<Self>;
}

// Payment handling

impl Handler<PaymentRequest> for Gateway {
    type Result = ();

    fn handle(&mut self, msg: PaymentRequest, _ctx: &mut Context<Self>) {
        self.log(&format!(
            "❔ Verifying payment from {} for ${}",
            msg.payer_id, msg.amount
        ));

        let mut payment = Payment {
            payer: msg.payer_id,
            payee: None,
            amount: msg.amount,
            status: PaymentStatus::Pending,
        };

        if self.should_accept_payment() {
            self.log(&format!("✅ Payment from {} accepted", msg.payer_id));
            payment.status = PaymentStatus::Accepted;

            self.payments.insert(msg.payer_id, payment);
        } else {
            self.log(&format!("⛔ Payment from {} rejected", msg.payer_id));
            payment.status = PaymentStatus::Rejected
        };

        let message = PaymentState {
            payer_id: payment.payer,
            status: payment.status,
        }
        .to_string();

        self.send_message(message, msg.payer_id, ActorType::Passenger);
    }
}

impl Handler<ReturnPayment> for Gateway {
    type Result = ();

    fn handle(&mut self, msg: ReturnPayment, _ctx: &mut Context<Self>) {
        self.log(&format!("⬅️ Returning payment to {}", msg.payer_id));
        if let Some(payment) = self.payments.get_mut(&msg.payer_id) {
            match payment.status {
                PaymentStatus::Accepted => {
                    if msg.restart_payment_approval {
                        payment.status = PaymentStatus::Refunded;
                    } else {
                        payment.status = PaymentStatus::RefundedWithoutRebooking;
                    }

                    let message = PaymentState {
                        payer_id: msg.payer_id,
                        status: payment.status,
                    }
                    .to_string();
                    self.send_message(message, msg.payer_id, ActorType::Passenger);
                }
                PaymentStatus::Paid => self.log("❌ Error: Payment already accepted"),
                _ => self.log("❌ Error: Payment not accepted"),
            }
        } else {
            self.log("❌ Error: Payment not found");
        }
    }
}

impl Handler<AssignPayee> for Gateway {
    type Result = ();

    fn handle(&mut self, msg: AssignPayee, _ctx: &mut Context<Self>) {
        self.log(&format!(
            "✅ Asigning payee driver {} to passenger {}",
            msg.payee_id, msg.payer_id
        ));
        match self.payments.get_mut(&msg.payer_id) {
            Some(payment) => payment.payee = Some(msg.payee_id),
            None => self.log("❌ Error: Payment not found"),
        }
    }
}

impl Handler<EnablePay> for Gateway {
    type Result = ();

    fn handle(&mut self, msg: EnablePay, _ctx: &mut Context<Self>) {
        self.log(&format!(
            "✅ Enabling payment for passenger {}",
            msg.payer_id
        ));
        match self.payments.get_mut(&msg.payer_id) {
            Some(payment) => payment.status = PaymentStatus::Paid,
            None => {
                self.log("❌ Error: Payment not found");
                return;
            }
        }

        match self.payments.get(&msg.payer_id) {
            Some(payment) => {
                if let Some(payee) = payment.payee {
                    self.log(&format!(
                        "💵 Sending payment confirmation to driver {} for ${}",
                        payee, payment.amount
                    ));
                    let message = PaymentConfirmation {
                        amount: payment.amount,
                    }
                    .to_string();
                    self.send_message(message, payee, ActorType::Driver);
                } else {
                    self.log("❌ Error: No payee asigned");
                }
            }
            None => self.log("❌ Error: Payment not found"),
        }
    }
}

impl Handler<RequestPaymentInfo> for Gateway {
    type Result = ();

    fn handle(&mut self, msg: RequestPaymentInfo, _ctx: &mut Context<Self>) {
        self.log(&format!(
            "❔ Asked info of Passenger {} from Driver {}",
            msg.payer_id, msg.requester_id
        ));

        if msg.payer_id == 0 {
            self.log("❌ Error: Invalid ID");
            return;
        }

        match self.payments.get(&msg.payer_id) {
            Some(payment) => {
                let message = PaymentState {
                    payer_id: payment.payer,
                    status: payment.status,
                }
                .to_string();
                self.send_message(message, msg.requester_id, ActorType::Driver);

                self.log(&format!(
                    "✅ Sent info of Passenger {} to Driver {}",
                    msg.payer_id, msg.requester_id
                ));
            }
            None => {
                self.log("❌ Error: Payment not found");
            }
        }
    }
}

impl Handler<FreePayment> for Gateway {
    type Result = ();

    fn handle(&mut self, msg: FreePayment, _ctx: &mut Context<Self>) {
        self.log(&format!(
            "⚠️ Passenger {} didn't enable payment",
            msg.payer_id
        ));
        if let Some(payment) = self.payments.get_mut(&msg.payer_id) {
            payment.status = PaymentStatus::Paid;
            let message = PaymentConfirmation {
                amount: payment.amount,
            }
            .to_string();
            if let Some(payee) = payment.payee {
                self.send_message(message, payee, ActorType::Driver);
            } else {
                self.log("❌ Error: Payee not assigned");
            }
        } else {
            self.log("❌ Error: Payment not found");
        }
    }
}

// Close Connection

impl Handler<CloseConnection> for Gateway {
    type Result = ();

    fn handle(&mut self, msg: CloseConnection, _ctx: &mut Context<Self>) {
        self.log(&format!(
            "⚠️ Closing connection with {} {}",
            msg.actor_type, msg.id
        ));
    }
}

// Communication

impl Handler<ReceivedTcpMessage> for Gateway {
    type Result = ();

    fn handle(&mut self, msg: ReceivedTcpMessage, ctx: &mut Context<Self>) {
        let message = GatewayMessage::from_string(msg.content.clone());

        match message {
            Ok(GatewayMessage::PaymentRequest(payment_request)) => {
                ctx.address().do_send(payment_request)
            }
            Ok(GatewayMessage::RequestPaymentInfo(request_ride_info)) => {
                ctx.address().do_send(request_ride_info)
            }
            Ok(GatewayMessage::ReturnPayment(return_payment)) => {
                ctx.address().do_send(return_payment)
            }
            Ok(GatewayMessage::AsignPayee(asign_payee)) => ctx.address().do_send(asign_payee),
            Ok(GatewayMessage::EnablePay(enable_pay)) => ctx.address().do_send(enable_pay),
            Ok(GatewayMessage::PaymentState(_)) => {
                self.log("❌ Error: Gateway should not receive PaymentState message")
            }
            Ok(GatewayMessage::PaymentConfirmation(_)) => {
                self.log("❌ Error: Gateway should not receive Payment message")
            }
            Ok(GatewayMessage::FreePayment(free_payment)) => ctx.address().do_send(free_payment),
            Err(e) => self.log(&format!(
                "❌ Error parsing message: {}, Message: {}",
                e, msg.content
            )),
        }
    }
}

// Initialization

impl Handler<Initialize> for Gateway {
    type Result = ();

    fn handle(&mut self, _msg: Initialize, ctx: &mut Context<Self>) {
        self.tcp_layer.do_send(AssignActor {
            actor: ctx.address().clone(),
        });

        self.tcp_layer.do_send(Initialize {});
        self.log("✅ Gateway initialized");
    }
}
