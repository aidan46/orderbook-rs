# Orderbook-rs
A blazingly fast orderbook library written in Rust.

# Features
- Add limit orders
- Remove limit orders
- Orders are sorted in Price-Time priority

# Usage
The order book can be constructed with `OrderBook::new(instrument_data)`, where `instrument_data` is a csv file with the instrument id and instrument name.

# Example
```rust
use orderbook_rs::{
    models::{Order, Side},
    order_book::OrderBook,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let instrument_data = "data/instruments.csv";
    let mut ob = OrderBook::new(instrument_data);
    let price = 69;
    let qty = 420;
    let side = Side::Ask;
    let instrument_id = 1;
    let order = Order::new(price, qty, side, instrument_id);

    match ob.add_order(&order) {
        Ok(order_id) => println!("Order with OrderId {order_id} inserted"),
        Err(e) => eprintln!("ERROR: {e}"),
    }
    Ok(())
}
```
