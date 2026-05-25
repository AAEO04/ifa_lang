use dashmap::DashMap;
use crate::token::ResourceToken;
use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "std")]
thread_local! {
    static CURRENT_ACTOR_ID: std::cell::Cell<Option<u64>> = std::cell::Cell::new(None);
}

#[cfg(feature = "std")]
pub fn set_current_actor_id(id: Option<u64>) {
    CURRENT_ACTOR_ID.with(|cell| cell.set(id));
}

#[cfg(feature = "std")]
pub fn get_current_actor_id() -> Option<u64> {
    CURRENT_ACTOR_ID.with(|cell| cell.get())
}

#[cfg(not(feature = "std"))]
pub fn set_current_actor_id(_id: Option<u64>) {}

#[cfg(not(feature = "std"))]
pub fn get_current_actor_id() -> Option<u64> {
    None
}

/// The Global Resource Registry (Olubode - The Gatekeeper)
///
/// Stores system resources (Files, Sockets, DB Connections) safely across threads.
/// Users hold `ResourceToken` (IDs) while the actual resource lives here.
///
/// "The key to the door is not the house itself."
pub struct ResourceRegistry {
    // We store Arc<dyn Any + Send + Sync> to allow downcasting to concrete types
    resources: DashMap<u64, Arc<dyn Any + Send + Sync>>,
    // Map resource ID -> Owner Actor ID (0 = global/unowned)
    owners: DashMap<u64, u64>,
    counter: AtomicU64,
}

impl std::fmt::Debug for ResourceRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceRegistry")
            .field("resources_count", &self.resources.len())
            .field("owners", &self.owners)
            .field("counter", &self.counter)
            .finish()
    }
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            resources: DashMap::new(),
            owners: DashMap::new(),
            counter: AtomicU64::new(1), // Start IDs at 1, 0 is reserved/null
        }
    }
}

impl Default for ResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceRegistry {
    /// Register a new resource and get a token
    pub fn register<T: Any + Send + Sync>(&self, resource: T) -> ResourceToken {
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        self.resources.insert(id, Arc::new(resource));
        
        let owner = get_current_actor_id().unwrap_or(0);
        self.owners.insert(id, owner);
        
        ResourceToken(id)
    }

    /// Set the owner of a resource token
    pub fn set_owner(&self, token: ResourceToken, owner_id: u64) {
        self.owners.insert(token.0, owner_id);
    }

    /// Check if a resource token is owned by the given actor_id (or is global)
    pub fn check_owner(&self, token: ResourceToken, actor_id: u64) -> bool {
        if let Some(r) = self.owners.get(&token.0) {
            let owner = *r.value();
            owner == 0 || owner == actor_id
        } else {
            false
        }
    }

    /// Get a strong reference to a resource by token
    /// Returns None if token is invalid, type mismatch, or ownership check fails
    pub fn get<T: Any + Send + Sync>(&self, token: ResourceToken) -> Option<Arc<T>> {
        // Enforce ownership check if running inside an actor context
        if let Some(actor_id) = get_current_actor_id() {
            if !self.check_owner(token, actor_id) {
                return None;
            }
        }

        // Retrieve generic Arc from map
        if let Some(resource_ref) = self.resources.get(&token.0) {
            // Clone the Arc (cheap increment)
            let arc_clone = resource_ref.value().clone();
            // Attempt downcast
            arc_clone.downcast::<T>().ok()
        } else {
            None // Not found
        }
    }

    /// Remove/Close a resource (Sacrifice)
    pub fn close(&self, token: ResourceToken) -> bool {
        self.owners.remove(&token.0);
        self.resources.remove(&token.0).is_some()
    }

    /// Take ownership of a resource, removing it from this registry
    pub fn take(&self, token: ResourceToken) -> Option<Arc<dyn Any + Send + Sync>> {
        self.owners.remove(&token.0);
        self.resources.remove(&token.0).map(|(_, v)| v)
    }

    /// Insert a raw resource into this registry (used for actor transfers)
    pub fn insert_raw(&self, token: ResourceToken, resource: Arc<dyn Any + Send + Sync>, owner_id: u64) {
        self.resources.insert(token.0, resource);
        self.owners.insert(token.0, owner_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockFile {
        path: String,
    }

    #[test]
    fn test_registry_flow() {
        let registry = ResourceRegistry::new();
        let file = MockFile {
            path: "test.txt".to_string(),
        };
        let token = registry.register(file);

        // Access valid
        let retrieved = registry.get::<MockFile>(token);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().path, "test.txt");

        // Type mismatch
        let wrong_type = registry.get::<String>(token);
        assert!(wrong_type.is_none());

        // Close
        assert!(registry.close(token));

        // Access after close
        assert!(registry.get::<MockFile>(token).is_none());
    }
}
