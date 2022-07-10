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
