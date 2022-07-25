//! orderbook
//!
//! A limit order book that sorts order First In First Out (FIFO)
//!
//! Example:
//! ```
//! use orderbook::{OrderBook, Order, Side};
//!
//! let mut ob = OrderBook::new();
//! let order = Order {
//!     price: 69,
//!     qty: 420,
//!     side: Side::Ask,
//!     id: 1
//! };
//!
//! match ob.insert(order) {
//!     Ok(()) => (),
//!     Err(e) => (),
//! }
//!
//! ```
mod book_side;
mod error;
mod order_book;
mod price_level;

use book_side::BookSide;
pub use book_side::Side;
pub use error::OrderBookError;
pub use order_book::{Order, OrderBook};
use price_level::PriceLevel;

type OrderId = u64;
type Price = u64;
type Qty = u64;
