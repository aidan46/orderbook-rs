#![warn(clippy::pedantic)]
mod book_side;
mod error;
mod order_book;
mod price_level;
mod sequencer;

use book_side::BookSide;
pub use book_side::Side;
pub use order_book::{Order, OrderBook};
use price_level::PriceLevel;
use sequencer::Sequencer;

type OrderId = u64;
type Price = u64;
type Qty = u64;
