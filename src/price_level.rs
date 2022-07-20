#![allow(unused, clippy::unused_self)]
use crate::{error::OrderBookError, Order, OrderId, Qty};
use std::collections::{HashMap, VecDeque};

pub(super) struct PriceLevel {
    queue: VecDeque<Order>,
    total_qty: Qty,
    orders: HashMap<OrderId, Order>,
}

impl PriceLevel {
    /// Constructor function
    pub(super) fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            total_qty: 0,
            orders: HashMap::new(),
        }
    }

    /// Function inserts new `Order` into `PriceLevel`
    pub(super) fn insert(&mut self, order: &Order, id: OrderId) {
        self.orders.insert(id, *order);
        self.queue.push_back(*order);
        self.total_qty += order.qty;
    }

    /// Function removes `Order` from `PriceLevel`
    pub(super) fn remove(&mut self, id: OrderId) -> Result<(), OrderBookError> {
        match self.orders.remove(&id) {
            Some(order) => {
                self.total_qty -= order.qty;
                // Remove order from VecDeque
                self.queue.retain(|&o| o != order);
                Ok(())
            }
            None => Err(OrderBookError::UnknownId(id)),
        }
    }

    pub(super) fn get_total_qty(&self) -> Qty {
        self.total_qty
    }

    /// Function drains orders on the given `Side` up to the given `Qty`
    ///
    /// Returns [`Some`] with orders and total collected `Qty`
    /// Returns [`None`] if there are no orders on the given `Side` and `Price` combination
    pub(super) fn get_orders_till_qty(&mut self, qty: Qty) -> Option<(Vec<Order>, Qty)> {
        todo!()
    }
}

impl Default for PriceLevel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use crate::{Order, OrderId, PriceLevel, Side};

    #[test]
    fn price_level_insert() {
        // Setup
        let mut pl = PriceLevel::default();
        let price = 69;
        let qty = 420;
        let side = Side::Ask;
        let order = Order { price, qty, side };
        let id: OrderId = 1;

        // Act
        pl.insert(&order, id);

        // Assert
        assert_eq!(pl.total_qty, qty);
        assert_eq!(pl.queue.len(), 1);
        assert!(pl.orders.contains_key(&id));
    }

    #[test]
    fn price_level_remove() {
        // Setup
        let mut pl = PriceLevel::default();
        let price = 69;
        let qty = 420;
        let side = Side::Ask;
        let order = Order { price, qty, side };
        let id: OrderId = 1;

        pl.insert(&order, id);
        // Act
        let ret = pl.remove(id);

        // Assert
        assert!(ret.is_ok());
        assert!(!pl.orders.contains_key(&id));
        assert_eq!(pl.total_qty, 0);
        assert_eq!(pl.queue.len(), 0);
    }

    #[test]
    fn price_level_remove_unknown_id() {
        // Setup
        let mut pl = PriceLevel::default();
        let id: OrderId = 1;

        // Act
        let ret = pl.remove(id);

        // Assert
        assert!(ret.is_err());
    }
}
