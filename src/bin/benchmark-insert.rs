use std::time::{Duration, Instant};

use orderbook::{Order, OrderBook, Side};

const LIMIT: u64 = 100_000;

fn average(items: &[Duration]) -> Duration {
    total(items) / items.len() as u32
}

fn total(items: &[Duration]) -> Duration {
    items.iter().sum()
}

fn single_side_single_price() {
    let mut ob = OrderBook::new();
    let price = 69;
    let qty = 420;
    let side = Side::Ask;
    let mut results = vec![];

    for id in 0..LIMIT {
        let now = Instant::now();
        ob.insert(Order::new(price, qty, side, id)).unwrap();
        let elapsed = now.elapsed();
        results.push(elapsed);
    }
    let total = total(&results);
    let average = average(&results);
    println!("\n|============= single side + price =============|");
    println!("|\t\tInserted {LIMIT} orders\t\t|");
    println!("|\tTotal duration orders:\t{total:?}\t|");
    println!("|\tAverage time per order:\t{average:?}\t\t|");
    println!("|===============================================|\n");
}

fn single_side_changing_price() {
    let mut ob = OrderBook::new();
    let mut price = 69;
    let qty = 420;
    let side = Side::Ask;
    let mut results = vec![];

    for id in 0..LIMIT {
        if id % (LIMIT / 1000) == 0 {
            price += 1;
        }
        let now = Instant::now();
        ob.insert(Order::new(price, qty, side, id)).unwrap();
        let elapsed = now.elapsed();
        results.push(elapsed);
    }
    let total = total(&results);
    let average = average(&results);
    println!("\n|=============== changing price ================|");
    println!("|\t\tInserted {LIMIT} orders\t\t|");
    println!("|\tTotal duration orders:\t{total:?}\t|");
    println!("|\tAverage time per order:\t{average:?}\t\t|");
    println!("|===============================================|\n");
}

fn changing_side_and_price() {
    let mut ob = OrderBook::new();
    let mut price = 69;
    let qty = 420;
    let mut side = Side::Ask;
    let mut results = vec![];

    for id in 0..LIMIT {
        if id % (LIMIT / 1000) == 0 {
            price += 1;
            side = match side {
                Side::Ask => Side::Bid,
                Side::Bid => Side::Ask,
            };
        }
        let now = Instant::now();
        ob.insert(Order::new(price, qty, side, id)).unwrap();
        let elapsed = now.elapsed();
        results.push(elapsed);
    }
    let total = total(&results);
    let average = average(&results);
    println!("\n|============ changing price + side ============|");
    println!("|\t\tInserted {LIMIT} orders\t\t|");
    println!("|\tTotal duration orders:\t{total:?}\t|");
    println!("|\tAverage time per order:\t{average:?}\t\t|");
    println!("|===============================================|\n");
}

fn main() {
    println!("Running insert benchmarks");
    single_side_single_price();
    single_side_changing_price();
    changing_side_and_price();
    println!("Finished running insert benchmarks");
}
