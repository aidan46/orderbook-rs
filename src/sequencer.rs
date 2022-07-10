use crate::models::OrderId;

pub struct Sequencer {
    next_id: OrderId,
}

impl Sequencer {
    /// Constructor function
    #[must_use]
    pub fn new() -> Sequencer {
        Sequencer { next_id: 1 }
    }

    pub fn get_next_id(&mut self) -> OrderId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

impl Default for Sequencer {
    fn default() -> Self {
        Self::new()
    }
}
