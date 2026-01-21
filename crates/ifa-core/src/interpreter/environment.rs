//! # GPC - Grandparent-Parent-Child Scope Chain
//!
//! Lexical scoping implementation for Ifá-Lang.
//! This module implements the scope chain pattern where variables are resolved
//! by walking up from child → parent → grandparent scopes.

use crate::value::IfaValue;
use std::collections::HashMap;

/// Runtime environment (scope) - implements GPC pattern
///
/// Variables are resolved by looking in the current scope first,
/// then walking up the parent chain until found or reaching the root.
#[derive(Debug, Clone)]
pub struct Environment {
    /// Variables defined in this scope
    pub values: HashMap<String, IfaValue>,
    /// Parent scope (if any) - walking up the chain
    pub parent: Option<Box<Environment>>,
}

impl Environment {
    /// Create a new root environment (no parent)
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
            parent: None,
        }
    }

    /// Create a child environment with the given parent
    /// Used for function calls, blocks, and closures
    pub fn with_parent(parent: Environment) -> Self {
        Environment {
            values: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    /// Define a variable in the current scope
    pub fn define(&mut self, name: &str, value: IfaValue) {
        self.values.insert(name.to_string(), value);
    }

    /// Get a variable by walking up the scope chain (GPC resolution)
    ///
    /// Resolution order: Child → Parent → Grandparent → ... → Root
    pub fn get(&self, name: &str) -> Option<IfaValue> {
        if let Some(value) = self.values.get(name) {
            Some(value.clone())
        } else if let Some(ref parent) = self.parent {
            parent.get(name) // Recursive walk up the chain
        } else {
            None
        }
    }

    /// Set a variable in the scope where it's defined
    /// Returns true if found and updated, false if not found
    pub fn set(&mut self, name: &str, value: IfaValue) -> bool {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            true
        } else if let Some(parent) = &mut self.parent {
            parent.set(name, value) // Recursive walk up
        } else {
            false
        }
    }

    /// Get all variable names in the current scope (not parents)
    pub fn local_names(&self) -> Vec<&String> {
        self.values.keys().collect()
    }

    /// Check if a variable exists in any scope
    pub fn contains(&self, name: &str) -> bool {
        self.get(name).is_some()
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpc_resolution() {
        // Grandparent scope
        let mut grandparent = Environment::new();
        grandparent.define("x", IfaValue::Int(1));

        // Parent scope
        let mut parent = Environment::with_parent(grandparent);
        parent.define("y", IfaValue::Int(2));

        // Child scope
        let mut child = Environment::with_parent(parent);
        child.define("z", IfaValue::Int(3));

        // Child can access all: x, y, z
        assert_eq!(child.get("x"), Some(IfaValue::Int(1)));
        assert_eq!(child.get("y"), Some(IfaValue::Int(2)));
        assert_eq!(child.get("z"), Some(IfaValue::Int(3)));
    }

    #[test]
    fn test_shadowing() {
        let mut parent = Environment::new();
        parent.define("x", IfaValue::Int(1));

        let mut child = Environment::with_parent(parent);
        child.define("x", IfaValue::Int(2)); // Shadow parent's x

        assert_eq!(child.get("x"), Some(IfaValue::Int(2)));
    }
}
