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
    /// Occupied price levels, kept sorted best-first for `side`: descending for
    /// [`Side::Bid`] so the highest bid is at the front, ascending for
    /// [`Side::Ask`] so the lowest ask is at the front.
    ///
    /// Holds exactly the keys of `price_levels`; see [`BookSide::prune_if_empty`].
    ///
    /// `insert` establishes this ordering and `remove` preserves it; the whole
    /// meaning of "best" lives in that pair, and `get_best_price` reads the
    /// front element on the strength of it.
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
                // Restores the best-first invariant on `self.prices`
                match self.side {
                    Side::Bid => self.prices.sort_by(|a, b| b.cmp(a)),
                    Side::Ask => self.prices.sort_by(Ord::cmp),
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
            }
            self.prune_if_empty(order.price);
        }
    }

    /// Drops a price level that holds no quantity, from both `price_levels` and
    /// `prices`.
    ///
    /// The invariant: a price is in `price_levels` if and only if it is in
    /// `prices`, and any level that is present holds a non-zero quantity. Every
    /// site that can decrease a level's quantity ends by calling this, so a
    /// drained level never survives to be found by `Entry::Occupied` on the next
    /// insert at that price, and `get_best_price` and `get_total_qty` can never
    /// disagree about whether a price exists.
    fn prune_if_empty(&mut self, price: Price) {
        if self
            .price_levels
            .get(&price)
            .is_some_and(|price_level| price_level.get_total_qty() == 0)
        {
            self.price_levels.remove(&price);
            self.prices.retain(|&p| p != price);
        }
    }

    /// Function gets the best price for the given `Side`
    ///
    /// Reads the front of `self.prices`, which is sorted best-first for
    /// `self.side`: highest first for [`Side::Bid`], lowest first for
    /// [`Side::Ask`]. See the invariant on the `prices` field.
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
                self.prune_if_empty(price);
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
        assert_eq!(bs.get_best_price(), None);
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
        // Lowest ask is the best ask
        assert_eq!(best_price, Some(&69));
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
        // Highest bid is the best bid
        assert_eq!(best_price, Some(&70));
    }

    #[test]
    fn get_best_price_ask_multiple_levels() {
        // Setup
        let side = Side::Ask;
        let mut bs = BookSide::new(side);
        let qty = 420;
        // Inserted out of order so the sort has to do work
        for (id, price) in [105, 103, 104].into_iter().enumerate() {
            bs.insert(&Order {
                price,
                qty,
                side,
                id: id as OrderId + 1,
            });
        }

        // Act
        let best_price = bs.get_best_price();

        // Assert
        assert_eq!(bs.prices.len(), 3);
        assert_eq!(best_price, Some(&103));
    }

    #[test]
    fn get_best_price_bid_multiple_levels() {
        // Setup
        let side = Side::Bid;
        let mut bs = BookSide::new(side);
        let qty = 420;
        // Inserted out of order so the sort has to do work
        for (id, price) in [100, 102, 101].into_iter().enumerate() {
            bs.insert(&Order {
                price,
                qty,
                side,
                id: id as OrderId + 1,
            });
        }

        // Act
        let best_price = bs.get_best_price();

        // Assert
        assert_eq!(bs.prices.len(), 3);
        assert_eq!(best_price, Some(&102));
    }

    #[test]
    fn get_best_price_after_level_removed() {
        // Setup
        let side = Side::Ask;
        let mut bs = BookSide::new(side);
        let qty = 420;
        // One order per level, so removing an order drains its level
        for (id, price) in [105, 103, 104].into_iter().enumerate() {
            bs.insert(&Order {
                price,
                qty,
                side,
                id: id as OrderId + 1,
            });
        }
        assert_eq!(bs.get_best_price(), Some(&103));

        // Act: drain the best level
        bs.remove(2);

        // Assert: the next level up becomes the best ask
        assert_eq!(bs.prices.len(), 2);
        assert_eq!(bs.get_best_price(), Some(&104));
    }

    #[test]
    fn reinsert_after_level_emptied() {
        // Setup
        let side = Side::Ask;
        let mut bs = BookSide::new(side);
        bs.insert(&Order {
            price: 100,
            qty: 5,
            side,
            id: 1,
        });
        bs.remove(1);

        // The drained level is gone from both maps, not just from `prices`
        assert!(bs.prices.is_empty());
        assert!(bs.price_levels.is_empty());
        assert_eq!(bs.get_best_price(), None);
        assert_eq!(bs.get_total_qty(100), None);

        // Act: refill the same price
        bs.insert(&Order {
            price: 100,
            qty: 3,
            side,
            id: 2,
        });

        // Assert: the price is visible again
        assert_eq!(bs.get_best_price(), Some(&100));
        assert_eq!(bs.get_total_qty(100), Some(3));
    }

    #[test]
    fn repeated_empty_refill_cycles() {
        // Setup
        let side = Side::Bid;
        let mut bs = BookSide::new(side);

        // Act: four empty/refill cycles at the same price
        for id in 1..=4 {
            bs.insert(&Order {
                price: 100,
                qty: 7,
                side,
                id,
            });
            assert_eq!(bs.get_best_price(), Some(&100));
            assert_eq!(bs.get_total_qty(100), Some(7));

            bs.remove(id);
            assert_eq!(bs.get_best_price(), None);
            assert_eq!(bs.get_total_qty(100), None);
        }

        // Assert: nothing accumulated across the cycles
        assert!(bs.prices.is_empty());
        assert!(bs.price_levels.is_empty());
        assert!(bs.map.is_empty());
    }

    #[test]
    fn best_price_and_total_qty_agree_after_partial_drain_of_a_level() {
        // Setup: two orders at the best ask, one at the level behind it
        let side = Side::Ask;
        let mut bs = BookSide::new(side);
        for (id, price) in [(1, 100), (2, 100), (3, 101)] {
            bs.insert(&Order {
                price,
                qty: 5,
                side,
                id,
            });
        }

        // Act: empty the best level one order at a time
        bs.remove(1);

        // Assert: still occupied, so it stays the best ask
        assert_eq!(bs.get_best_price(), Some(&100));
        assert_eq!(bs.get_total_qty(100), Some(5));

        bs.remove(2);

        // Assert: emptied, so it disappears from both queries at once
        assert_eq!(bs.get_best_price(), Some(&101));
        assert_eq!(bs.get_total_qty(100), None);
        assert_eq!(bs.prices.len(), 1);
        assert_eq!(bs.price_levels.len(), 1);
    }

    #[test]
    fn reinsert_after_level_drained_by_qty() {
        // Setup
        let side = Side::Ask;
        let mut bs = BookSide::new(side);
        bs.insert(&Order {
            price: 100,
            qty: 5,
            side,
            id: 1,
        });

        // Act: drain the level through the matching path rather than `remove`
        let res = bs.get_orders_till_qty(100, 5);
        assert!(res.is_some());

        // Assert: a fully matched level is not left behind as a phantom top of book
        assert_eq!(bs.get_best_price(), None);
        assert_eq!(bs.get_total_qty(100), None);
        assert!(bs.price_levels.is_empty());

        // Act: refill the same price
        bs.insert(&Order {
            price: 100,
            qty: 2,
            side,
            id: 2,
        });

        // Assert
        assert_eq!(bs.get_best_price(), Some(&100));
        assert_eq!(bs.get_total_qty(100), Some(2));
    }

    #[test]
    fn partial_drain_keeps_level() {
        // Setup
        let side = Side::Ask;
        let mut bs = BookSide::new(side);
        bs.insert(&Order {
            price: 100,
            qty: 5,
            side,
            id: 1,
        });

        // Act: take less than the level holds
        let res = bs.get_orders_till_qty(100, 2);
        assert!(res.is_some());

        // Assert: residual quantity keeps the level visible
        assert_eq!(bs.get_best_price(), Some(&100));
        assert_eq!(bs.get_total_qty(100), Some(3));
    }

    #[test]
    fn get_best_price_empty() {
        // Setup
        let asks = BookSide::new(Side::Ask);
        let bids = BookSide::new(Side::Bid);

        // Assert
        assert_eq!(asks.get_best_price(), None);
        assert_eq!(bids.get_best_price(), None);
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
