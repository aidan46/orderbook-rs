use orderbook::{Order, OrderBook, Side};

fn main() {
    let mut ob = OrderBook::default();
    let id = 1;
    let side = Side::Ask;
    let price = 69;
    let qty = 420;
    let order = Order {
        price,
        qty,
        side,
        id,
    };

    // Insert order
    match ob.insert(order) {
        Ok(()) => println!("Added order!"),
        Err(e) => eprintln!("{e:?}"),
    }

    // Utility functions
    let best_price = ob.get_best_price(side);
    assert!(best_price.is_some());
    let best_price = best_price.unwrap();
    println!("Best price = {best_price:?} on side {side:?}");
    let total_qty = ob.get_total_qty(*best_price, side);
    assert!(total_qty.is_some());
    let total_qty = total_qty.unwrap();
    println!("Total quantity = {total_qty} at price: {best_price} on side {side:?}");
    match ob.get_orders_till_qty(price, side, total_qty) {
        Some((orders, drained_qty)) => {
            println!(
                "Got {} order(s) with total quantity of {drained_qty}",
                orders.len()
            );
            println!("Orders drained: {orders:#?}");
        }
        None => (),
    }

    // Remove order
    match ob.remove(id) {
        Ok(()) => println!("Removed order!"),
        Err(e) => eprintln!("{e:?}"),
    }
}
