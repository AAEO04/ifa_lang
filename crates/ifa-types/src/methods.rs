pub use crate::odu_metadata::{resolve_method_id, method_name_from_id};

/// Stable method IDs for all built-in Odù domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OduMethodId(pub u16);

impl OduMethodId {
    pub fn new(domain_id: u8, method_idx: u8) -> Self {
        OduMethodId(((domain_id as u16) << 8) | (method_idx as u16))
    }
}
