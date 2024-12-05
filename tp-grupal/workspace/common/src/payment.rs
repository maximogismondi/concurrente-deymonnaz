use std::{fmt::Display, str::FromStr};

use crate::Id;

/// Represents a payment in the ConcuRide system.
#[derive(Clone, Copy)]
pub struct Payment {
    /// The identifier of the payer (Passenger).
    pub payer: Id,
    /// The identifier of the payee (Driver).
    pub payee: Option<Id>,
    /// The amount of the payment.
    pub amount: usize,
    /// The status of the payment.
    pub status: PaymentStatus,
}

/// Represents the various statuses a payment can have.
///
/// # Variants
///
/// - `Pending`: The payment is pending.
/// - `Accepted`: The payment has been accepted.
/// - `Rejected`: The payment has been rejected.
/// - `Paid`: The payment has been given to the payee.
/// - `Refunded`: The payment has been refunded.
/// - `RefundedWithoutRebooking`: The payment has been refunded and they won´t rebook another ride.
#[derive(Clone, Copy, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Accepted,
    Rejected,
    Paid,
    Refunded,
    RefundedWithoutRebooking,
}

impl Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentStatus::Pending => write!(f, "Pending"),
            PaymentStatus::Accepted => write!(f, "Accepted"),
            PaymentStatus::Rejected => write!(f, "Rejected"),
            PaymentStatus::Paid => write!(f, "Paid"),
            PaymentStatus::Refunded => write!(f, "Refunded"),
            PaymentStatus::RefundedWithoutRebooking => write!(f, "RefundedWithoutRebooking"),
        }
    }
}

impl FromStr for PaymentStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(PaymentStatus::Pending),
            "Accepted" => Ok(PaymentStatus::Accepted),
            "Rejected" => Ok(PaymentStatus::Rejected),
            "Paid" => Ok(PaymentStatus::Paid),
            "Refunded" => Ok(PaymentStatus::Refunded),
            "RefundedWithoutRebooking" => Ok(PaymentStatus::RefundedWithoutRebooking),
            _ => Err(()),
        }
    }
}
