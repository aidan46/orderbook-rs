use std::time::{Duration, Instant};

use orderbook::{Order, OrderBook, Side};

const LIMIT: u64 = 100_000;

fn average(items: &[Duration]) -> Duration {
    total(items) / items.len() as u32
}

fn total(items: &[Duration]) -> Duration {
    items.iter().sum()
}

fn main() {
    let mut ob = OrderBook::default();
    let price = 69;
    let qty = 420;
    let side = Side::Ask;
    let mut results = vec![];

    for id in 0..LIMIT {
        ob.insert(Order::new(price, qty, side, id)).unwrap();
    }
    for id in 0..LIMIT {
        let now = Instant::now();
        ob.remove(id).unwrap();
        let elapsed = now.elapsed();
        results.push(elapsed);
    }
    let total = total(&results);
    let average = average(&results);
    println!("\n|================ Remove orders ================|");
    println!("|\t\tRemoved {LIMIT} orders\t\t|");
    println!("|\tTotal duration:\t\t{total:?}\t|");
    println!("|\tAverage time per order:\t{average:?}\t\t|");
    println!("|===============================================|\n");
}
