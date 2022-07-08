use super::{OrderId, Price, Qty, Side};
use std::{
    cmp::Ordering,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct Order {
    pub order_id: OrderId,
    pub price: Price,
    pub qty: Qty,
    pub side: Side,
    pub ts: u64,
}

impl Order {
    #[must_use]
    /// # Panics
    /// Returns an [`Err`] if current time is later than `UNIX_EPOCH`
    pub fn new(order_id: OrderId, price: Price, qty: Qty, side: Side) -> Order {
        Order {
            order_id,
            price,
            qty,
            side,
            ts: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

impl Default for Order {
    fn default() -> Self {
        Self::new(1, 69, 420, Side::Ask)
    }
}

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        match self.price.cmp(&other.price) {
            Ordering::Less | Ordering::Greater => false,
            Ordering::Equal => self.ts == other.ts,
        }
    }
}

impl Eq for Order {}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.price.cmp(&other.price) {
            Ordering::Less => match self.side {
                Side::Ask => Ordering::Greater,
                Side::Bid => Ordering::Less,
            },
            Ordering::Greater => match self.side {
                Side::Ask => Ordering::Less,
                Side::Bid => Ordering::Greater,
            },
            Ordering::Equal => self.ts.cmp(&other.ts),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{super::Side, Order};

    #[test]
    fn order_index_ordering_ask() {
        let o1 = Order {
            price: 69,
            side: Side::Ask,
            ..Order::default()
        };
        let o2 = Order {
            price: 70,
            side: Side::Ask,
            ..Order::default()
        };

        assert!(o1 > o2);
        assert!(o1 != o2);
    }

    #[test]
    fn order_index_ordering_bid() {
        let o1 = Order {
            price: 69,
            side: Side::Bid,
            ..Order::default()
        };
        let o2 = Order {
            price: 70,
            side: Side::Bid,
            ..Order::default()
        };

        assert!(o1 < o2);
        assert!(o1 != o2);
    }
}
