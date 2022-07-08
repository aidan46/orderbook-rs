use super::{order::Order, OrderId, Side};
use anyhow::{bail, Result};
use std::collections::{BinaryHeap, HashSet};

#[derive(Debug)]
pub struct BookSide {
    pub orders: BinaryHeap<Order>,
}

impl BookSide {
    #[must_use]
    pub fn new() -> BookSide {
        BookSide {
            orders: BinaryHeap::new(),
        }
    }

    fn insert(&mut self, order: Order) {
        self.orders.push(order);
    }
}

impl Default for BookSide {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct OrderBook {
    pub asks: BookSide,
    pub bids: BookSide,
    pub active: HashSet<OrderId>,
}

impl OrderBook {
    #[must_use]
    pub fn new() -> OrderBook {
        OrderBook {
            asks: BookSide::default(),
            bids: BookSide::default(),
            active: HashSet::new(),
        }
    }

    /// # Errors
    /// Returns [`Err`] if the `OrderId` is already present
    pub fn try_insert(&mut self, order: Order) -> Result<()> {
        match order.side {
            Side::Ask => self.try_insert_ask(order),
            Side::Bid => self.try_insert_bid(order),
        }
    }

    /// # Errors
    /// Returns [`Err`] if the `OrderId` is not present
    pub fn try_remove(&mut self, order_id: OrderId) -> Result<()> {
        if !self.active.remove(&order_id) {
            bail!("Order with OrderId {order_id} not found!");
        }
        Ok(())
    }

    fn try_insert_ask(&mut self, order: Order) -> Result<()> {
        if !self.active.insert(order.order_id) {
            bail!("Order with OrderId {} is already present!", order.order_id);
        }
        self.asks.insert(order);
        Ok(())
    }

    fn try_insert_bid(&mut self, order: Order) -> Result<()> {
        if !self.active.insert(order.order_id) {
            bail!("Order with OrderId {} is already present!", order.order_id);
        }
        self.bids.insert(order);
        Ok(())
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::OrderBook;
    use crate::models::{order::Order, Side};

    #[test]
    fn insert_order() {
        // Setup
        let mut ob = OrderBook::default();
        let o1 = Order::default();

        // Act + assert
        assert!(ob.try_insert(o1).is_ok());
    }

    #[test]
    fn delete_order() {
        // Setup
        let mut ob = OrderBook::default();
        let order_id = 1;
        let o1 = Order {
            order_id,
            ..Order::default()
        };

        assert!(ob.try_insert(o1).is_ok());

        // Act + assert
        assert!(ob.try_remove(order_id).is_ok());
        assert!(ob.try_remove(order_id).is_err());
    }

    #[test]
    fn duplicate_order_id() {
        // Setup
        let mut ob = OrderBook::default();
        let o1 = Order {
            order_id: 1,
            price: 69,
            side: Side::Ask,
            ..Order::default()
        };
        let o2 = Order {
            order_id: 1,
            price: 69,
            side: Side::Bid,
            ..Order::default()
        };

        // Act + assert
        assert!(ob.try_insert(o1).is_ok());
        assert!(ob.try_insert(o2).is_err());
    }
}
