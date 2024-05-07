#![allow(unused, clippy::unused_self)]
use crate::error::OrderBookError;
use crate::{Order, OrderId, Qty, Side};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

type TimeStamp = u128; // SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();

#[derive(Debug, Eq, PartialEq)]
struct TimeStampId {
    id: OrderId,
    ts: TimeStamp,
}

impl TimeStampId {
    fn new(id: OrderId) -> Self {
        Self {
            id,
            ts: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        }
    }
}

impl PartialOrd for TimeStampId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TimeStampId {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.ts.cmp(&other.ts) {
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
            Ordering::Equal => Ordering::Equal,
        }
    }
}

#[derive(Debug)]
pub(super) struct PriceLevel {
    queue: BinaryHeap<TimeStampId>,
    total_qty: Qty,
    map: HashMap<OrderId, Order>,
}

impl PriceLevel {
    /// Constructor function
    pub(super) fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
            total_qty: 0,
            map: HashMap::new(),
        }
    }

    /// Function inserts new `Order` into `PriceLevel`
    pub(super) fn insert(&mut self, order: &Order) {
        let id = order.id;
        self.map.insert(id, *order);
        self.queue.push(TimeStampId::new(id));
        self.total_qty += order.qty;
    }

    /// Function removes `Order` from `PriceLevel`
    pub(super) fn remove(&mut self, id: OrderId) {
        if let Some(order) = self.map.remove(&id) {
            self.total_qty -= order.qty;
        }
    }

    pub(super) fn get_total_qty(&self) -> Qty {
        self.total_qty
    }

    /// Function drains map on the given `Side` up to the given `Qty`
    ///
    /// Returns [`Some`] with map and total collected `Qty`
    /// Returns [`None`] if there are no map on the given `Side` and `Price` combination
    pub(super) fn get_orders_till_qty(&mut self, total_qty: Qty) -> (Vec<Order>, Qty) {
        let mut collected_qty = 0;
        let mut orders = vec![];

        // Peek order
        while let Some(item) = self.queue.pop() {
            if let Some(order) = self.map.get_mut(&item.id) {
                match (collected_qty + order.qty).cmp(&total_qty) {
                    Ordering::Less => {
                        collected_qty += order.qty;
                        self.total_qty -= order.qty;
                        orders.push(*order);
                        self.map.remove(&item.id);
                    }
                    Ordering::Greater => {
                        let order_diff = (collected_qty + order.qty) - total_qty;
                        let total_diff = order.qty - order_diff;
                        orders.push(Order {
                            id: order.id,
                            price: order.price,
                            qty: (order.qty - order_diff),
                            side: Side::Ask,
                        });
                        collected_qty = total_qty;
                        order.qty = order_diff;
                        self.total_qty -= total_diff;
                        self.queue.push(item);
                        break;
                    }
                    Ordering::Equal => {
                        collected_qty += order.qty;
                        self.total_qty -= order.qty;
                        orders.push(*order);
                        self.map.remove(&item.id);
                        break;
                    }
                }
            }
        }
        (orders, collected_qty)
    }
}

impl Default for PriceLevel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::PriceLevel;
    use crate::{Order, OrderId, Side};

    #[test]
    fn get_single_till_qty_remaining() {
        // Setup
        let mut pl = PriceLevel::default();
        let price = 69;
        let qty = 420;
        let side = Side::Ask;
        let id: OrderId = 1;
        let order = Order {
            price,
            qty,
            side,
            id,
        };

        pl.insert(&order);
        // Act
        let (items, total_qty) = pl.get_orders_till_qty(qty - 2);

        // Assert
        assert_eq!(pl.total_qty, 2);
        assert_eq!(items.len(), 1);
        let item = items.first().unwrap();
        assert_eq!(item.qty, qty - 2);
        assert_eq!(total_qty, qty - 2);
    }

    #[test]
    fn get_single_till_qty_drain_exact() {
        // Setup
        let mut pl = PriceLevel::default();
        let price = 69;
        let qty = 420;
        let side = Side::Ask;
        let id: OrderId = 1;
        let order = Order {
            price,
            qty,
            side,
            id,
        };

        pl.insert(&order);
        // Act
        let (items, total_qty) = pl.get_orders_till_qty(qty);

        // Assert
        assert_eq!(pl.total_qty, 0);
        assert_eq!(items.len(), 1);
        let item = items.first().unwrap();
        assert_eq!(item.qty, qty);
        assert_eq!(total_qty, qty);
    }

    #[test]
    fn get_multi_till_qty_remaining() {
        // Setup
        let mut pl = PriceLevel::default();
        let price = 69;
        let qty = 420;
        let side = Side::Ask;
        let id: OrderId = 1;
        let order = Order {
            price,
            qty,
            side,
            id,
        };

        pl.insert(&order);
        let id_2: OrderId = 2;
        let order = Order {
            price,
            qty,
            side,
            id: id_2,
        };

        pl.insert(&order);
        // Act
        let (items, total_qty) = pl.get_orders_till_qty(qty + 3);

        // Assert
        assert_eq!(total_qty, qty + 3);
        assert_eq!(items.len(), 2);
        assert_eq!(pl.total_qty, qty - 3);
        assert_eq!(pl.map.len(), 1);
        assert_eq!(pl.queue.len(), 1);

        // First item
        let item = items.first().unwrap();
        assert_eq!(item.qty, qty);
        assert!(!pl.map.contains_key(&id));

        // Second item
        let item = items.get(1).unwrap();
        assert_eq!(item.qty, 3);
        assert!(pl.map.contains_key(&id_2));
    }

    #[test]
    // given quantity larger than total_qty
    fn get_multi_till_qty_overflow() {
        // Setup
        let mut pl = PriceLevel::default();
        let price = 69;
        let qty = 420;
        let side = Side::Ask;
        let id: OrderId = 1;
        let order = Order {
            price,
            qty,
            side,
            id,
        };

        pl.insert(&order);
        let id_2: OrderId = 2;
        let order = Order {
            price,
            qty,
            side,
            id: id_2,
        };

        pl.insert(&order);
        let test_qty = (qty * 2) + 2;
        // Act
        let (items, total_qty) = pl.get_orders_till_qty(test_qty);

        // Assert
        assert_ne!(total_qty, test_qty);
        assert_eq!(total_qty, qty * 2);
        assert_eq!(items.len(), 2);
        assert_eq!(pl.total_qty, 0);
        assert!(pl.map.is_empty());
        assert!(pl.queue.is_empty());

        // First item
        let item = items.first().unwrap();
        assert_eq!(item.qty, qty);
        assert!(!pl.map.contains_key(&id));

        // Second item
        let item = items.get(1).unwrap();
        assert_eq!(item.qty, qty);
        assert!(!pl.map.contains_key(&id_2));
    }

    #[test]
    fn get_multi_till_qty_exact() {
        // Setup
        let mut pl = PriceLevel::default();
        let price = 69;
        let qty = 420;
        let side = Side::Ask;
        let id: OrderId = 1;
        let order = Order {
            price,
            qty,
            side,
            id,
        };

        pl.insert(&order);
        let id_2: OrderId = 2;
        let order = Order {
            price,
            qty,
            side,
            id: id_2,
        };

        pl.insert(&order);
        // Act
        let (items, total_qty) = pl.get_orders_till_qty(qty * 2);

        // Assert
        assert_eq!(total_qty, qty * 2);
        assert_eq!(items.len(), 2);
        assert_eq!(pl.total_qty, 0);
        assert!(pl.map.is_empty());
        assert!(pl.queue.is_empty());

        // First item
        let item = items.first().unwrap();
        assert_eq!(item.qty, qty);
        assert!(!pl.map.contains_key(&id));

        // Second item
        let item = items.get(1).unwrap();
        assert_eq!(item.qty, qty);
        assert!(!pl.map.contains_key(&id_2));
    }

    #[test]
    fn insert() {
        // Setup
        let mut pl = PriceLevel::default();
        let price = 69;
        let qty = 420;
        let side = Side::Ask;
        let id: OrderId = 1;
        let order = Order {
            price,
            qty,
            side,
            id,
        };

        // Act
        pl.insert(&order);

        // Assert
        assert_eq!(pl.total_qty, qty);
        assert_eq!(pl.queue.len(), 1);
        assert!(pl.map.contains_key(&id));
    }

    #[test]
    fn remove() {
        // Setup
        let mut pl = PriceLevel::default();
        let price = 69;
        let qty = 420;
        let side = Side::Ask;
        let id: OrderId = 1;
        let order = Order {
            price,
            qty,
            side,
            id,
        };

        pl.insert(&order);
        // Act
        pl.remove(id);

        // Assert
        assert!(!pl.map.contains_key(&id));
        assert_eq!(pl.total_qty, 0);
    }
}
