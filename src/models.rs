use std::{
    ops::Not,
    time::{SystemTime, UNIX_EPOCH},
};

pub type InstrumentId = i32;
pub type OrderId = u32;
pub type Price = i32;
pub type Qty = u32;

// Side + `Not` impl
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Side {
    Ask,
    Bid,
}

impl Not for Side {
    type Output = Side;
    fn not(self) -> Self::Output {
        match self {
            Side::Ask => Side::Bid,
            Side::Bid => Side::Ask,
        }
    }
}

// Order
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Order {
    pub order_id: Option<OrderId>,
    pub price: Price,
    pub qty: Qty,
    pub side: Side,
    pub instrument_id: InstrumentId,
    pub ts: u64,
}

impl Order {
    #[must_use]
    /// # Panics
    /// Returns an [`Err`] if current time is later than `UNIX_EPOCH`
    pub fn new(price: Price, qty: Qty, side: Side, instrument_id: InstrumentId) -> Order {
        Order {
            order_id: None,
            price,
            qty,
            side,
            instrument_id,
            ts: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn assign_order_id(&mut self, order_id: OrderId) {
        self.order_id = Some(order_id);
    }
}

#[cfg(test)]
mod test {
    use super::Side;

    #[test]
    fn not_side_impl() {
        let ask = Side::Ask;
        let bid = Side::Bid;

        assert_eq!(!ask, Side::Bid);
        assert_eq!(!bid, Side::Ask);
    }
}
