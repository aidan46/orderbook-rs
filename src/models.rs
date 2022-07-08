pub mod order;
pub mod order_book;

pub type OrderId = i32;
pub type Price = i32;
pub type Qty = i32;

#[derive(Debug)]
pub enum Side {
    Ask,
    Bid,
}
