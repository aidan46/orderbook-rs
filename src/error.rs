#![allow(clippy::module_name_repetitions)]
use crate::OrderId;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum OrderBookError {
    #[error("OrderId not found")]
    UnknownId(OrderId),
    #[error("Duplicate OrderId {0}")]
    DuplicateOrderId(OrderId),
}
