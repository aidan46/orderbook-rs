use crate::{
    OrderId, Price, Qty, {Order, PriceLevel},
};
use anyhow::{bail, Result};
use std::collections::{hash_map::Entry, HashMap};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Ask,
    Bid,
}

pub(super) struct BookSide {
    price_levels: HashMap<Price, PriceLevel>,
    orders: HashMap<OrderId, Order>,
}

impl BookSide {
    /// Constructor function
    pub(super) fn new() -> Self {
        Self {
            price_levels: HashMap::new(),
            orders: HashMap::new(),
        }
    }

    /// Function insert new order into the `BookSide`
    pub(super) fn insert(&mut self, order: &Order, id: OrderId) {
        match self.price_levels.entry(order.price) {
            Entry::Vacant(new_price_lvl) => {
                let mut price_lvl = PriceLevel::new();
                price_lvl.insert(order, id);
                new_price_lvl.insert(price_lvl);
            }
            Entry::Occupied(mut price_lvl) => {
                price_lvl.get_mut().insert(order, id);
            }
        }
        self.orders.insert(id, *order);
    }

    /// Function removes order with given `OrderId`
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the order with given `OrderId` is not present
    pub(super) fn remove(&mut self, id: OrderId) -> Result<()> {
        match self.orders.remove(&id) {
            Some(order) => match self.price_levels.get_mut(&order.price) {
                Some(price_level) => price_level.remove(id),
                None => bail!(
                    "Order with OrderId {id} on PriceLevel {} was not found",
                    order.price
                ),
            },
            None => bail!("Order with OrderId {id} is not present"),
        }
    }

    /// Function gets the best price for the given `Side`
    ///
    /// Returns [`None`] if there are no orders on given side
    pub(super) fn get_best_price(&self) -> Option<Price> {
        todo!()
    }

    /// Function gets the total quantity at the given `Price` and `Side` combination
    pub(super) fn get_total_qty(&self, price: Price) -> Option<Qty> {
        self.price_levels.get(&price).map(PriceLevel::get_total_qty)
    }

    /// Function drains orders on the given `Price` and `Side` combination up to the given `Qty`
    ///
    /// Returns [`Some`] with orders and total collected `Qty`
    /// Returns [`None`] if there are no orders on the given `Side` and `Price` combination
    pub(super) fn get_orders_till_qty(
        &mut self,
        price: Price,
        qty: Qty,
    ) -> Option<(Vec<Order>, Qty)> {
        match self.price_levels.get_mut(&price) {
            Some(price_level) => price_level.get_orders_till_qty(qty),
            None => None,
        }
    }
}

impl Default for BookSide {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use crate::{BookSide, Order, OrderId, Side};

    #[test]
    fn book_side_insert() {
        // Setup
        let mut bs = BookSide::default();
        let price = 69;
        let qty = 420;
        let side = Side::Ask;
        let order = Order { price, qty, side };
        let id: OrderId = 1;

        // Act
        bs.insert(&order, id);

        // Assert
        assert!(bs.orders.contains_key(&id));
        assert!(bs.price_levels.contains_key(&price));
        assert_eq!(bs.get_total_qty(price), Some(qty));
    }

    #[test]
    fn book_side_remove() {
        // Setup
        let mut bs = BookSide::default();
        let price = 69;
        let qty = 420;
        let side = Side::Ask;
        let order = Order { price, qty, side };
        let id: OrderId = 1;

        bs.insert(&order, id);

        // Act
        assert!(bs.remove(id).is_ok());
        assert!(!bs.orders.contains_key(&id));
    }

    #[test]
    fn book_side_remove_unknown_id() {
        // Setup
        let mut bs = BookSide::default();
        let id: OrderId = 1;

        // Act
        let ret = bs.remove(id);

        // Assert
        assert!(ret.is_err());
    }
}
