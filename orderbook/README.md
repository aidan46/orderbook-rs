# OrderBook-rs
> A blazingly fast limit order book library.

## Key features
- Insert orders into order book
- Remove orders from the order book
- Orders sorted FIFO (First In, First Out)
- Helper functions for matching

## Getting started
Add the orderbook-rs to your `Cargo.toml` file.
```toml
..
[dependencies]
orderbook = { git = "https://github.com/aidan46/orderbook-rs/" }
..
```

### Example Usage
Constructing an order book and inserting an order, then deleting it.
```rust
use orderbook::{error::OrderBookError, Order, OrderBook, Side};

let mut ob = OrderBook::default();
let order = Order {
	price: 69,
	qty: 420,
	side: Side::Ask,
};

// Insert
let order_id: u64 = ob.insert(&order);

// Remove
if let Err(OrderBookError::UnknownId(id)) = ob.remove(order_id) {
	eprintln!("Order with OrderId {order_id} not found");
}
```

## Order book implementation details

### PriceLevel
The most granular data structure in the order book is the `PriceLevel` which represents a single `Price` on a `Side`.
A `PriceLevel` holds a queue that is sorted on price time and the total quantity at this `Price`

### BookSide
A `BookSide` represents one side (`Bid` or `Ask`) of an order book, holding multiple `PriceLevel`'s.
Other that `PriceLevel`'s, it also holds the a sorted list of the best price for quick acccess.

### OrderBook
A full `OrderBook`, this is the public facing data structure that end-users of the library will interact with. An `OrderBook` holds 2 `BookSide`'s and a `Sequencer` to create `OrderId`'s.
