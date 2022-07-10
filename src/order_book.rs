use crate::{
    models::{InstrumentId, Order, OrderId, Price, Side},
    sequencer::Sequencer,
};
use anyhow::{bail, Result};
use std::{
    collections::{hash_map::Entry, HashMap},
    fs,
};

pub type PriceLevel = HashMap<Price, Vec<Order>>;
pub type InstrumentSide = (InstrumentId, Side);
pub type InstrumentBook = HashMap<InstrumentSide, PriceLevel>;
pub type OrderDetails = HashMap<OrderId, Order>;
pub type InstrumentMap = HashMap<InstrumentId, String>;

pub struct OrderBook {
    pub instrument_book: InstrumentBook,
    pub order_details: OrderDetails,
    pub instrument_map: InstrumentMap,
    sequencer: Sequencer,
}

impl OrderBook {
    /// Constructor function
    /// `instrument_path`: The path to csv file with `instrument_id` + `instrument_name`
    /// # Panics
    /// Function panics if the `instrument_path` is invalid
    #[must_use]
    pub fn new(instrument_path: &str) -> OrderBook {
        let (instrument_book, instrument_map) =
            OrderBook::read_instruments(instrument_path).unwrap();
        OrderBook {
            instrument_book,
            order_details: HashMap::new(),
            instrument_map,
            sequencer: Sequencer::new(),
        }
    }

    fn read_instruments(instrument_path: &str) -> Result<(InstrumentBook, InstrumentMap)> {
        let data = match fs::read_to_string(instrument_path) {
            Ok(data) => data,
            Err(e) => bail!(e),
        };
        let mut instrument_book = InstrumentBook::new();
        let mut instrument_map = InstrumentMap::new();
        let mut rdr = csv::ReaderBuilder::new().from_reader(data.as_bytes());
        for record in rdr.records() {
            let record = record?;
            let instrument_id = record.get(0).unwrap().parse()?;
            let instrument_name = record.get(1).unwrap();
            let ask_side = (instrument_id, Side::Ask);
            let bid_side = (instrument_id, Side::Bid);
            instrument_map.insert(instrument_id, instrument_name.to_owned());
            instrument_book.insert(ask_side, PriceLevel::new());
            instrument_book.insert(bid_side, PriceLevel::new());
        }
        Ok((instrument_book, instrument_map))
    }

    fn assign_order_id(order: &Order, order_id: OrderId) -> Order {
        let mut order = *order;
        order.order_id = Some(order_id);

        order
    }

    /// Add an order to the [`OrderBook`]
    ///
    /// # Errors
    ///
    /// Returns an [`Err`] if the instrument does not exist
    pub fn add_order(&mut self, order: &Order) -> Result<OrderId> {
        let instrument_id = order.instrument_id;
        let side = order.side;
        let price = order.price;
        let order_id = self.sequencer.get_next_id();
        let order = OrderBook::assign_order_id(order, order_id);
        self.order_details.insert(order_id, order);
        match self.instrument_book.entry((instrument_id, side)) {
            Entry::Vacant(..) => {
                bail!("InstrumentId {instrument_id} does not exist!");
            }
            Entry::Occupied(mut entry) => {
                let price_level = entry.get_mut();
                match price_level.entry(price) {
                    Entry::Vacant(new_entry) => {
                        // Create new Vec and insert
                        new_entry.insert(vec![order]);
                    }
                    Entry::Occupied(mut entry) => {
                        // Append order to Vec
                        entry.get_mut().push(order);
                    }
                }
            }
        }
        Ok(order_id)
    }

    /// Remove an order from the [`OrderBook`]
    ///
    /// # Errors
    ///
    /// Function returns [`Err`] if the given `order_id` does not exist
    pub fn remove_order(&mut self, order_id: OrderId) -> Result<()> {
        // Remove from `OrderDetails`
        let order = match self.order_details.remove(&order_id) {
            None => bail!("Order with OrderId {order_id} does not exist"),
            Some(order) => order,
        };

        // Remove from instrument_book
        let instrument_side = (order.instrument_id, order.side);
        let price = order.price;
        match self.instrument_book.get_mut(&instrument_side) {
            None => (),
            Some(price_level) => match price_level.get_mut(&price) {
                Some(price) => price.retain(|o| o.order_id != Some(order_id)),
                None => bail!("Order with OrderId {order_id} not found in `InstrumentBook`"),
            },
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        models::{Order, Side},
        order_book::OrderBook,
    };

    fn get_order_book() -> OrderBook {
        OrderBook::new("data/instruments.csv")
    }

    #[test]
    fn add_order() {
        // Setup
        let mut ob = get_order_book();
        let price = 69;
        let qty = 420;
        let side = Side::Ask;
        let instrument_id = 1;
        let order = Order::new(price, qty, side, instrument_id);

        // Act
        let res = ob.add_order(&order);

        // Assert
        assert!(res.is_ok());
        let order_id = res.unwrap();
        let instrument_side = (instrument_id, side);
        assert_eq!(order_id, 1);
        assert!(ob.order_details.get(&order_id).is_some());
        let instrument = ob.instrument_book.get(&instrument_side);
        assert!(instrument.is_some());
        let price_level = instrument.unwrap().get(&price);
        assert!(price_level.is_some());
        let price_level = price_level.unwrap();
        assert_eq!(price_level.len(), 1);
        let inserted = price_level.iter().find(|&&o| {
            o.price == price && o.qty == qty && o.side == side && o.instrument_id == instrument_id
        });
        assert!(inserted.is_some());
    }

    #[test]
    fn remove_order() {
        // Setup
        let mut ob = get_order_book();
        let price = 69;
        let qty = 420;
        let side = Side::Ask;
        let instrument_id = 1;
        let order = Order::new(price, qty, side, instrument_id);

        // Add order so we can remove it
        let order_id = ob.add_order(&order).unwrap();

        // Act
        let res = ob.remove_order(order_id);

        // Assert
        assert!(res.is_ok());
        assert!(ob.order_details.get(&order_id).is_none());
    }
}
