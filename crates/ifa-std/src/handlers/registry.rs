use dashmap::DashMap;
use ifa_types::ResourceToken;
use once_cell::sync::Lazy;
use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// The Global Resource Registry (Olubode - The Gatekeeper)
///
/// Stores system resources (Files, Sockets, DB Connections) safely across threads.
/// Users hold `ResourceToken` (IDs) while the actual resource lives here.
///
/// "The key to the door is not the house itself."
pub struct ResourceRegistry {
    // We store Arc<dyn Any + Send + Sync> to allow downcasting to concrete types
    resources: DashMap<u64, Arc<dyn Any + Send + Sync>>,
    counter: AtomicU64,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            resources: DashMap::new(),
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
        ResourceToken(id)
    }

    /// Get a strong reference to a resource by token
    /// Returns None if token is invalid or type mismatch
    pub fn get<T: Any + Send + Sync>(&self, token: ResourceToken) -> Option<Arc<T>> {
        // Retrieve generic Arc from map
        if let Some(resource_ref) = self.resources.get(&token.0) {
            // Clone the Arc (cheap increment)
            let arc_clone = resource_ref.value().clone();
            // Attempt downcast
            // Attempt downcast
            arc_clone.downcast::<T>().ok()
        } else {
            None // Not found
        }
    }

    /// Remove/Close a resource (Sacrifice)
    pub fn close(&self, token: ResourceToken) -> bool {
        self.resources.remove(&token.0).is_some()
    }
}

/// Global Singleton Instance
pub static REGISTRY: Lazy<ResourceRegistry> = Lazy::new(ResourceRegistry::new);

#[cfg(test)]
mod tests {
    use super::*;

    struct MockFile {
        path: String,
    }

    #[test]
    fn test_registry_flow() {
        let file = MockFile {
            path: "test.txt".to_string(),
        };
        let token = REGISTRY.register(file);

        // Access valid
        let retrieved = REGISTRY.get::<MockFile>(token);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().path, "test.txt");

        // Type mismatch
        let wrong_type = REGISTRY.get::<String>(token);
        assert!(wrong_type.is_none());

        // Close
        assert!(REGISTRY.close(token));

        // Access after close
        assert!(REGISTRY.get::<MockFile>(token).is_none());
    }
}
