pub use crate::odu_metadata::{method_name_from_id, resolve_method_id};

/// Stable method IDs for all built-in Odù domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OduMethodId(pub u16);

impl OduMethodId {
    pub fn new(domain_id: u8, method_idx: u8) -> Self {
        OduMethodId(((domain_id as u16) << 8) | (method_idx as u16))
    }
}
