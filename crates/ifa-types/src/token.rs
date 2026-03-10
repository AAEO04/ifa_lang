use serde::{Deserialize, Serialize};

/// A unique identifier for a system resource (File, Socket, etc.)
///
/// This token acts as a key to the Global Resource Registry ("The Village Market").
/// It is Copy, Send, and Sync, making it safe to pass between domains.
///
/// "The key to the door is not the house itself."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceToken(pub u64);

impl ResourceToken {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}
