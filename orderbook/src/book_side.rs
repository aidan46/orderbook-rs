use crate::{Order, OrderId, Price, PriceLevel, Qty};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::Not;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Ask or Bid
pub enum Side {
    /// Ordered in ascending order
    Ask,
    /// Ordered in descending order
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

pub(super) struct BookSide {
    price_levels: HashMap<Price, PriceLevel>,
    map: HashMap<OrderId, Order>,
    side: Side,
    prices: Vec<Price>,
}

impl BookSide {
    /// Constructor function
    pub(super) fn new(side: Side) -> Self {
        Self {
            price_levels: HashMap::new(),
            map: HashMap::new(),
            side,
            prices: Vec::new(),
        }
    }

    /// Function insert new order into the `BookSide`
    pub(super) fn insert(&mut self, order: &Order) {
        let id = order.id;
        match self.price_levels.entry(order.price) {
            Entry::Vacant(new_price_lvl) => {
                let mut price_lvl = PriceLevel::new();
                price_lvl.insert(order);
                new_price_lvl.insert(price_lvl);
                self.prices.push(order.price);
                match self.side {
                    Side::Bid => self.prices.sort_by(Ord::cmp),
                    Side::Ask => self.prices.sort_by(|a, b| b.cmp(a)),
                }
            }
            Entry::Occupied(mut price_lvl) => {
                price_lvl.get_mut().insert(order);
            }
        }
        self.map.insert(id, *order);
    }

    /// Function removes order with given `OrderId`
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the order with given `OrderId` is not present
    pub(super) fn remove(&mut self, id: OrderId) {
        if let Some(order) = self.map.remove(&id) {
            if let Some(price_level) = self.price_levels.get_mut(&order.price) {
                price_level.remove(id);
                if price_level.get_total_qty() == 0 {
                    self.prices.retain(|&p| p != order.price);
                }
            }
        }
    }

    /// Function gets the best price for the given `Side`
    ///
    /// Returns [`None`] if there are no orders on given side
    pub(super) fn get_best_price(&self) -> Option<&Price> {
        self.prices.first()
    }

    /// Function gets the total quantity at the given `Price` and `Side` combination
    pub(super) fn get_total_qty(&self, price: Price) -> Option<Qty> {
        self.price_levels.get(&price).map(PriceLevel::get_total_qty)
    }

    /// Function drains orders on the given `Price` and `Side` combination up to the given `Qty`
    ///
    /// Returns [`Some`] with map and total collected `Qty`
    /// Returns [`None`] if there are no map on the given `Side` and `Price` combination
    pub(super) fn get_orders_till_qty(
        &mut self,
        price: Price,
        qty: Qty,
    ) -> Option<(Vec<Order>, Qty)> {
        match self
            .price_levels
            .get_mut(&price)
            .map(|price_level| price_level.get_orders_till_qty(qty))
        {
            Some((orders, total_qty)) => {
                orders.iter().for_each(|order| {
                    self.map.remove(&order.id);
                });
                Some((orders, total_qty))
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{BookSide, Order, OrderId, Side};

    #[test]
    fn insert() {
        // Setup
        let side = Side::Ask;
        let mut bs = BookSide::new(side);
        let price = 69;
        let qty = 420;
        let id: OrderId = 1;
        let order = Order {
            price,
            qty,
            side,
            id,
        };

        // Act
        bs.insert(&order);

        // Assert
        assert!(bs.map.contains_key(&id));
        assert!(bs.price_levels.contains_key(&price));
        assert_eq!(bs.get_total_qty(price), Some(qty));
        assert_eq!(bs.prices.len(), 1);
        let best_price = bs.prices.first().unwrap();
        assert_eq!(*best_price, price);
    }

    #[test]
    fn remove() {
        // Setup
        let side = Side::Ask;
        let mut bs = BookSide::new(side);
        let price = 69;
        let qty = 420;
        let id: OrderId = 1;
        let order = Order {
            price,
            qty,
            side,
            id,
        };

        bs.insert(&order);
        bs.remove(id);

        // Act
        assert!(!bs.map.contains_key(&id));
        assert!(bs.prices.is_empty());
    }

    #[test]
    fn get_best_price_ask() {
        // Setup
        let side = Side::Ask;
        let mut bs = BookSide::new(side);
        // First order
        let price = 69;
        let qty = 420;
        let id: OrderId = 1;
        let o1 = Order {
            price,
            qty,
            side,
            id,
        };
        bs.insert(&o1);

        // Second order
        let price = 70;
        let id: OrderId = 2;
        let o2 = Order {
            price,
            qty,
            side,
            id,
        };
        bs.insert(&o2);

        // Act
        let best_price = bs.get_best_price();

        // Assert
        assert_eq!(best_price, Some(&o2.price));
    }

    #[test]
    fn get_best_price_bid() {
        // Setup
        let side = Side::Bid;
        let mut bs = BookSide::new(side);
        // First order
        let price = 69;
        let qty = 420;
        let id: OrderId = 1;
        let o1 = Order {
            price,
            qty,
            side,
            id,
        };
        bs.insert(&o1);

        // Second order
        let price = 70;
        let id: OrderId = 2;
        let o2 = Order {
            price,
            qty,
            side,
            id,
        };
        bs.insert(&o2);

        // Act
        let best_price = bs.get_best_price();

        // Assert
        assert_eq!(best_price, Some(&o1.price));
    }

    #[test]
    fn get_total_qty() {
        // Setup
        let side = Side::Bid;
        let mut bs = BookSide::new(side);
        // First order
        let price = 69;
        let qty = 420;
        let id: OrderId = 1;
        let o1 = Order {
            price,
            qty,
            side,
            id,
        };
        bs.insert(&o1);

        // Second order
        let id: OrderId = 2;
        let o2 = Order {
            price,
            qty,
            side,
            id,
        };
        bs.insert(&o2);

        // Act
        let total_qty = bs.get_total_qty(price);

        // Assert
        assert_eq!(total_qty, Some(qty * 2));
    }

    #[test]
    // Function tested in `PriceLevel`
    fn get_till_qty() {
        // Setup
        let side = Side::Bid;
        let mut bs = BookSide::new(side);
        // First order
        let price = 69;
        let qty = 420;
        let id: OrderId = 1;
        let o1 = Order {
            price,
            qty,
            side,
            id,
        };
        bs.insert(&o1);

        // Second order
        let id_2: OrderId = 2;
        let o2 = Order {
            price,
            qty,
            side,
            id: id_2,
        };
        bs.insert(&o2);

        // Act
        let res = bs.get_orders_till_qty(price, qty * 2);
        assert!(res.is_some());
        let (items, total_qty) = res.unwrap();

        // Assert
        assert_eq!(items.len(), 2);
        assert_eq!(total_qty, qty * 2);

        // First item
        let item = items.first().unwrap();
        assert_eq!(item.qty, qty);
        assert!(!bs.map.contains_key(&id));

        // First item
        let item = items.get(1).unwrap();
        assert_eq!(item.qty, qty);
        assert!(!bs.map.contains_key(&id_2));
    }
}
