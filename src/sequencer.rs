use crate::OrderId;

pub(super) struct Sequencer {
    next_id: OrderId,
}

impl Sequencer {
    /// Constructor function
    pub fn new() -> Self {
        Self { next_id: 1 }
    }

    /// Get next available `OrderId`
    pub fn get_next_id(&mut self) -> OrderId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}
