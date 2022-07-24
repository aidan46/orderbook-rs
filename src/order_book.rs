use crate::{error::OrderBookError, BookSide, OrderId, Price, Qty, Side};
use std::collections::{hash_map::Entry, HashMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Order {
    pub price: Price,
    pub qty: Qty,
    pub side: Side,
    pub id: OrderId,
}

pub struct OrderBook {
    asks: BookSide,
    bids: BookSide,
    orders: HashMap<OrderId, Order>,
}

impl OrderBook {
    /// Constructor function
    #[must_use]
    pub fn new() -> Self {
        Self {
            asks: BookSide::new(Side::Ask),
            bids: BookSide::new(Side::Bid),
            orders: HashMap::new(),
        }
    }

    /// Function insert a new [`Order`] into the [`OrderBook`]
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the given `id` is already in the orderbook
    /// Example:
    /// ```
    /// use orderbook::{OrderBook, Order, Side};
    /// let mut ob = OrderBook::new();
    /// let price = 69;
    /// let qty = 420;
    /// let side = Side::Ask;
    /// let id = 1;
    /// let order = Order {
    ///     price,
    ///     qty,
    ///     side,
    ///     id
    /// };
    ///
    /// ob.insert(order);
    ///
    /// ```
    pub fn insert(&mut self, order: Order) -> Result<(), OrderBookError> {
        let id = order.id;
        match self.orders.entry(id) {
            Entry::Vacant(entry) => {
                match order.side {
                    Side::Ask => self.asks.insert(&order),
                    Side::Bid => self.bids.insert(&order),
                };
                entry.insert(order);
                Ok(())
            }
            Entry::Occupied(..) => Err(OrderBookError::DuplicateOrderId(id)),
        }
    }

    /// Function removes an [`Order`] according to an `OrderId`
    ///
    /// # Errors
    /// Returns [`Err`] if the order with the given `OrderId` is not present
    ///
    /// Example:
    /// ```
    /// use orderbook::{OrderBook};
    ///
    /// let mut ob = OrderBook::new();
    /// let id = 69;
    ///
    /// match ob.remove(id) {
    ///     Ok(()) => (), // Order Removed!
    ///     Err(e) => (), // OrderId not found!
    /// }
    /// ```
    pub fn remove(&mut self, id: OrderId) -> Result<(), OrderBookError> {
        match self.orders.remove(&id) {
            Some(order) => {
                match order.side {
                    Side::Ask => self.asks.remove(id),
                    Side::Bid => self.bids.remove(id),
                };
                Ok(())
            }
            None => Err(OrderBookError::UnknownId(id)),
        }
    }

    /// Function gets the best price for the given `Side`
    ///
    /// Returns [`Some`] `Price` on success
    ///
    /// Returns [`None`] if there are no orders on given side
    pub fn get_best_price(&self, side: Side) -> Option<&Price> {
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
    use crate::{Order, OrderBook, OrderId, Side};

    #[test]
    fn insert() {
        // Setup
        let mut ob = OrderBook::default();
        let price = 69;
        let qty = 420;
        let side = Side::Ask;
        let id = 1;
        let order = Order {
            price,
            qty,
            side,
            id,
        };

        // Act
        let res = ob.insert(order);

        // Assert
        assert!(res.is_ok());
        assert!(ob.orders.contains_key(&id));
    }

    #[test]
    fn remove() {
        // Setup
        let mut ob = OrderBook::default();
        let price = 69;
        let qty = 420;
        let side = Side::Ask;
        let id = 1;
        let order = Order {
            price,
            qty,
            side,
            id,
        };

        let res = ob.insert(order);
        assert!(res.is_ok());

        // Act
        let res = ob.remove(id);

        // Assert
        assert!(res.is_ok());
        assert!(!ob.orders.contains_key(&id));
    }

    #[test]
    fn remove_unknown_id() {
        // Setup
        let mut ob = OrderBook::default();
        let id: OrderId = 1;

        // Act
        let res = ob.remove(id);

        // Assert
        assert!(res.is_err());
    }

    #[test]
    fn get_best_price_ask() {
        // Setup
        let side = Side::Ask;
        let mut ob = OrderBook::new();
        // First order
        let price = 69;
        let qty = 420;
        let id = 1;
        let o1 = Order {
            price,
            qty,
            side,
            id,
        };
        let res = ob.insert(o1);
        assert!(res.is_ok());

        // Second order
        let price = 70;
        let o2 = Order {
            price,
            qty,
            side,
            id: id + 1,
        };
        let res = ob.insert(o2);
        assert!(res.is_ok());

        // Act
        let best_price = ob.get_best_price(side);

        // Assert
        assert_eq!(best_price, Some(&o2.price));
    }

    #[test]
    fn get_best_price_bid() {
        // Setup
        let side = Side::Bid;
        let mut ob = OrderBook::new();
        // First order
        let price = 69;
        let qty = 420;
        let id = 1;
        let o1 = Order {
            price,
            qty,
            side,
            id,
        };
        let res = ob.insert(o1);
        assert!(res.is_ok());

        // Second order
        let price = 70;
        let o2 = Order {
            price,
            qty,
            side,
            id: id + 1,
        };
        let res = ob.insert(o2);
        assert!(res.is_ok());

        // Act
        let best_price = ob.get_best_price(side);

        // Assert
        assert_eq!(best_price, Some(&o1.price));
    }

    #[test]
    fn get_orders_till_qty() {
        // Setup
        let side = Side::Bid;
        let mut ob = OrderBook::new();
        // First order
        let price = 69;
        let qty = 420;
        let id = 1;
        let o1 = Order {
            price,
            qty,
            side,
            id,
        };
        let res = ob.insert(o1);
        assert!(res.is_ok());

        // Second order
        let price = 69;
        let id_2 = id + 1;
        let o2 = Order {
            price,
            qty,
            side,
            id: id_2,
        };
        let res = ob.insert(o2);
        assert!(res.is_ok());

        // Act
        let res = ob.get_orders_till_qty(price, side, qty * 2);
        assert!(res.is_some());
        let (orders, total_qty) = res.unwrap();
        // Assert
        assert_eq!(orders.len(), 2);
        assert_eq!(total_qty, qty * 2);

        // First item
        let item = orders.get(0).unwrap();
        assert_eq!(item.id, id);
        assert_eq!(item.price, price);
        assert_eq!(item.qty, qty);
        assert_eq!(item.side, side);

        // Second item
        let item = orders.get(1).unwrap();
        assert_eq!(item.id, id_2);
        assert_eq!(item.price, price);
        assert_eq!(item.qty, qty);
        assert_eq!(item.side, side);
    }
}
