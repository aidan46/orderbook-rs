#![allow(clippy::module_name_repetitions)]
use crate::OrderId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrderBookError {
    #[error("OrderId not found")]
    UnknownId(OrderId),
}
