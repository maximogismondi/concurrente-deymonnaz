use core::fmt;

use actix::Message;

use crate::{payment::PaymentStatus, Id};

const PAYMENT_REQUEST: &str = "PaymentRequest";
const REQUEST_PAYMENT_INFO: &str = "RequestPaymentInfo";
const RETURN_PAYMENT: &str = "ReturnPayment";
const ASIGN_PAYEE: &str = "AsignPayee";
const ENABLE_PAY: &str = "EnablePay";
const PAYMENT_STATE: &str = "PaymentState";
const PAYMENT_CONFIRMATION: &str = "PaymentConfirmation";
const FREE_PAYMENT: &str = "FreePayment";

pub enum GatewayMessage {
    PaymentRequest(PaymentRequest),
    RequestPaymentInfo(RequestPaymentInfo),
    ReturnPayment(ReturnPayment),
    AsignPayee(AssignPayee),
    EnablePay(EnablePay),
    PaymentState(PaymentState),
    PaymentConfirmation(PaymentConfirmation),
    FreePayment(FreePayment),
}

impl GatewayMessage {
    pub fn from_string(msg: String) -> Result<Self, String> {
        let mut parts = msg.split_whitespace();
        let msg_type = match parts.next() {
            Some(msg) => msg,
            None => return Err("Invalid message format: missing message type".to_string()),
        };

        let content = parts.collect::<Vec<&str>>().join(" ");

        match msg_type {
            PAYMENT_REQUEST => Ok(GatewayMessage::PaymentRequest(PaymentRequest::from_string(
                content,
            ))),
            REQUEST_PAYMENT_INFO => Ok(GatewayMessage::RequestPaymentInfo(
                RequestPaymentInfo::from_string(content),
            )),
            RETURN_PAYMENT => Ok(GatewayMessage::ReturnPayment(ReturnPayment::from_string(
                content,
            ))),
            ASIGN_PAYEE => Ok(GatewayMessage::AsignPayee(AssignPayee::from_string(
                content,
            ))),
            ENABLE_PAY => Ok(GatewayMessage::EnablePay(EnablePay::from_string(content))),
            PAYMENT_STATE => Ok(GatewayMessage::PaymentState(PaymentState::from_string(
                content,
            ))),
            PAYMENT_CONFIRMATION => Ok(GatewayMessage::PaymentConfirmation(
                PaymentConfirmation::from_string(content),
            )),
            FREE_PAYMENT => Ok(GatewayMessage::FreePayment(FreePayment::from_string(
                content,
            ))),
            _ => Err("Invalid message type".to_string()),
        }
    }
}

/// Represents a payment request to the gateway.
#[derive(Message)]
#[rtype(result = "()")]
pub struct PaymentRequest {
    /// The ID of the payer.
    pub payer_id: Id,
    /// The amount to be paid.
    pub amount: usize,
}

impl PaymentRequest {
    pub fn from_string(msg: String) -> Self {
        let mut parts = msg.split_whitespace();
        let payer_id = parts.next().unwrap_or("0").parse::<Id>().unwrap_or(0);
        let amount = parts.next().unwrap_or("0").parse::<usize>().unwrap_or(0);

        Self { payer_id, amount }
    }
}

impl fmt::Display for PaymentRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{} {} {}", PAYMENT_REQUEST, self.payer_id, self.amount)
    }
}

/// Represents a request for payment information to the gateway from the leader.
#[derive(Message)]
#[rtype(result = "()")]
pub struct RequestPaymentInfo {
    /// The ID of the payer.
    pub payer_id: Id,
    /// The ID of the requester (Leader Driver).
    pub requester_id: Id,
}

impl RequestPaymentInfo {
    pub fn from_string(msg: String) -> Self {
        let mut parts = msg.split_whitespace();
        let payer_id = parts.next().unwrap_or("0").parse::<Id>().unwrap_or(0);
        let requester_id = parts.next().unwrap_or("0").parse::<Id>().unwrap_or(0);

        Self {
            payer_id,
            requester_id,
        }
    }
}

impl fmt::Display for RequestPaymentInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "{} {} {}",
            REQUEST_PAYMENT_INFO, self.payer_id, self.requester_id
        )
    }
}

/// Represents a message to return a payment to the passenger
#[derive(Message)]
#[rtype(result = "()")]
pub struct ReturnPayment {
    /// The ID of the payer.
    pub payer_id: Id,
    /// If the payment approval should be restarted or not.
    pub restart_payment_approval: bool,
}

impl ReturnPayment {
    pub fn from_string(msg: String) -> Self {
        let mut parts = msg.split_whitespace();
        let payer_id = parts.next().unwrap_or("0").parse::<Id>().unwrap_or(0);
        let restart_payment_approval = parts
            .next()
            .unwrap_or("false")
            .parse::<bool>()
            .unwrap_or(false);
        Self {
            payer_id,
            restart_payment_approval,
        }
    }
}

impl fmt::Display for ReturnPayment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "{} {} {}",
            RETURN_PAYMENT, self.payer_id, self.restart_payment_approval
        )
    }
}

/// Represents a message to assign a payee to a payer.
#[derive(Message)]
#[rtype(result = "()")]
pub struct AssignPayee {
    /// The ID of the payer.
    pub payer_id: Id,
    /// The ID of the payee.
    pub payee_id: Id,
}

impl AssignPayee {
    pub fn from_string(msg: String) -> Self {
        let mut parts = msg.split_whitespace();
        let payer_id = parts.next().unwrap_or("0").parse::<Id>().unwrap_or(0);
        let payee_id = parts.next().unwrap_or("0").parse::<Id>().unwrap_or(0);

        Self { payer_id, payee_id }
    }
}

impl fmt::Display for AssignPayee {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{} {} {}", ASIGN_PAYEE, self.payer_id, self.payee_id)
    }
}

/// Represents a message to enable the payment to the driver.
#[derive(Message)]
#[rtype(result = "()")]
pub struct EnablePay {
    /// The ID of the payer.
    pub payer_id: Id,
}

impl EnablePay {
    pub fn from_string(msg: String) -> Self {
        let payer_id = msg.parse::<Id>().unwrap_or(0);
        Self { payer_id }
    }
}

impl fmt::Display for EnablePay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{} {}", ENABLE_PAY, self.payer_id)
    }
}

/// Represents a message to communicate the payment state.
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct PaymentState {
    /// The ID of the payer.
    pub payer_id: Id,
    /// The status of the payment.
    pub status: PaymentStatus,
}

impl PaymentState {
    pub fn from_string(msg: String) -> Self {
        let mut parts = msg.split_whitespace();
        let payer_id = parts.next().unwrap_or("0").parse::<Id>().unwrap_or(0);
        let status = parts
            .next()
            .unwrap_or("")
            .parse::<PaymentStatus>()
            .unwrap_or(PaymentStatus::Rejected);

        Self { payer_id, status }
    }
}

impl fmt::Display for PaymentState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{} {} {}", PAYMENT_STATE, self.payer_id, self.status)
    }
}

/// Represents a message to confirm the payment has been made to the driver.
#[derive(Message)]
#[rtype(result = "()")]
pub struct PaymentConfirmation {
    /// The amount of the payment.
    pub amount: usize,
}

impl PaymentConfirmation {
    pub fn from_string(msg: String) -> Self {
        let amount = msg.parse::<usize>().unwrap_or(0);
        Self { amount }
    }
}

impl fmt::Display for PaymentConfirmation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{} {}", PAYMENT_CONFIRMATION, self.amount)
    }
}

/// Represents a message to free the payment because the passenger didn´t enable it.
#[derive(Message)]
#[rtype(result = "()")]
pub struct FreePayment {
    /// The ID of the payer.
    pub payer_id: Id,
}

impl FreePayment {
    pub fn from_string(msg: String) -> Self {
        let payer_id = msg.parse::<Id>().unwrap_or(0);
        Self { payer_id }
    }
}

impl fmt::Display for FreePayment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{} {}", FREE_PAYMENT, self.payer_id)
    }
}
