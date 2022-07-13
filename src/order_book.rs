use crate::{
    OrderId, Price, Qty, {BookSide, Sequencer, Side},
};
use anyhow::{bail, Result};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Order {
    pub price: Price,
    pub qty: Qty,
    pub side: Side,
}

pub struct OrderBook {
    asks: BookSide,
    bids: BookSide,
    orders: HashMap<OrderId, Order>,
    sequencer: Sequencer,
}

impl OrderBook {
    /// Constructor function
    #[must_use]
    pub fn new() -> Self {
        Self {
            asks: BookSide::new(),
            bids: BookSide::new(),
            orders: HashMap::new(),
            sequencer: Sequencer::new(),
        }
    }

    /// Function insert a new [`Order`] into the [`OrderBook`]
    ///
    /// Example:
    /// ```
    /// use ob::{OrderBook, Order, Side};
    /// let mut ob = OrderBook::new();
    /// let price = 69;
    /// let qty = 420;
    /// let side = Side::Ask;
    /// let order = Order {
    ///     price,
    ///     qty,
    ///     side
    /// };
    ///
    /// ob.insert(order);
    ///
    /// ```
    pub fn insert(&mut self, order: Order) -> OrderId {
        let id = self.sequencer.get_next_id();
        match order.side {
            Side::Ask => self.asks.insert(&order, id),
            Side::Bid => self.bids.insert(&order, id),
        };
        self.orders.insert(id, order);
        id
    }

    /// Function removes an [`Order`] according to an `OrderId`
    ///
    /// # Errors
    /// Returns [`Err`] if the order with the given `OrderId` is not present
    ///
    /// Example:
    /// ```
    /// use ob::{OrderBook};
    ///
    /// let mut ob = OrderBook::new();
    /// let id = 69;
    ///
    /// match ob.remove(id) {
    ///     Ok(()) => (), // Order Removed!
    ///     Err(e) => (), // OrderId not found!
    /// }
    /// ```
    pub fn remove(&mut self, id: OrderId) -> Result<()> {
        match self.orders.remove(&id) {
            Some(order) => match order.side {
                Side::Ask => self.asks.remove(id),
                Side::Bid => self.bids.remove(id),
            },
            None => bail!("Order with OrderId {id} is not present"),
        }
    }

    /// Function gets the best price for the given `Side`
    ///
    /// Returns [`Some`] `Price` on success
    ///
    /// Returns [`None`] if there are no orders on given side
    #[must_use]
    pub fn get_best_price(&self, side: Side) -> Option<Price> {
        match side {
            Side::Ask => self.asks.get_best_price(),
            Side::Bid => self.bids.get_best_price(),
        }
    }

    /// Function gets the total quantity at the given `Price` and `Side` combination
    ///
    /// Returns [`Some`] `Qty` on success
    ///
    /// Returns [`None`] if there are no orders on given `Side` and `Price` combination
    #[must_use]
    pub fn get_total_qty(&self, price: Price, side: Side) -> Option<Qty> {
        match side {
            Side::Ask => self.asks.get_total_qty(price),
            Side::Bid => self.bids.get_total_qty(price),
        }
    }

    /// Function drains orders on the given `Price` and `Side` combination up to the given `Qty`
    ///
    /// Returns [`Some`] [`Vec`] of [`Order`] and total collected `Qty`
    ///
    /// Returns [`None`] if there are no orders on the given `Side` and `Price` combination
    pub fn get_orders_till_qty(
        &mut self,
        price: Price,
        side: Side,
        qty: Qty,
    ) -> Option<(Vec<Order>, Qty)> {
        match side {
            Side::Ask => self.asks.get_orders_till_qty(price, qty),
            Side::Bid => self.bids.get_orders_till_qty(price, qty),
        }
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use crate::{Order, OrderBook, Side};

    #[test]
    fn order_book_insert() {
        // Setup
        let mut ob = OrderBook::default();
        let price = 69;
        let qty = 420;
        let side = Side::Ask;
        let order = Order { price, qty, side };

        // Act
        let order_id = ob.insert(order);

        // Assert
        assert_eq!(order_id, 1);
        assert!(ob.orders.contains_key(&order_id));
    }

    #[test]
    fn order_book_remove() {
        // Setup
        let mut ob = OrderBook::default();
        let price = 69;
        let qty = 420;
        let side = Side::Ask;
        let order = Order { price, qty, side };

        let id = ob.insert(order);

        // Act
        let res = ob.remove(id);

        // Assert
        assert!(res.is_ok());
        assert!(!ob.orders.contains_key(&id));
    }
}
