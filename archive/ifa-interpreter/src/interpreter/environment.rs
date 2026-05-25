//! # GPC - Grandparent-Parent-Child Scope Chain
//!
//! Lexical scoping implementation for Ifá-Lang.
//! This module implements the scope chain pattern where variables are resolved
//! by walking up from child → parent → grandparent scopes.

use crate::value::IfaValue;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Shared reference to an environment node.
pub type EnvRef = Rc<RefCell<Environment>>;

/// Runtime environment (scope) - implements GPC pattern
///
/// Variables are resolved by looking in the current scope first,
/// then walking up the parent chain until found or reaching the root.
#[derive(Debug, Clone)]
pub struct Environment {
    /// Variables defined in this scope
    pub values: HashMap<String, IfaValue>,
    /// Constant bindings (ayanfe) defined in this scope.
    pub consts: HashSet<String>,
    /// Parent scope (if any) - walking up the chain
    pub parent: Option<EnvRef>,
}

impl Environment {
    /// Create a new root environment (no parent).
    pub fn new() -> EnvRef {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            consts: HashSet::new(),
            parent: None,
        }))
    }

    /// Create a child environment with the given parent.
    /// Used for function calls, blocks, and closures.
    pub fn with_parent(parent: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            consts: HashSet::new(),
            parent: Some(parent),
        }))
    }

    /// Define a variable in the current scope
    pub fn define(env: &EnvRef, name: &str, value: IfaValue) {
        env.borrow_mut().values.insert(name.to_string(), value);
    }

    /// Define a constant binding in the current scope.
    pub fn define_const(env: &EnvRef, name: &str, value: IfaValue) {
        let mut env = env.borrow_mut();
        env.values.insert(name.to_string(), value);
        env.consts.insert(name.to_string());
    }

    /// Returns true if `name` resolves to a constant binding in any active scope.
    pub fn is_const(env: &EnvRef, name: &str) -> bool {
        let env_ref = env.borrow();
        if env_ref.consts.contains(name) {
            true
        } else if let Some(ref parent) = env_ref.parent {
            Environment::is_const(parent, name)
        } else {
            false
        }
    }

    /// Get a variable by walking up the scope chain (GPC resolution)
    ///
    /// Resolution order: Child → Parent → Grandparent → ... → Root
    pub fn get(env: &EnvRef, name: &str) -> Option<IfaValue> {
        let env_ref = env.borrow();
        if let Some(value) = env_ref.values.get(name) {
            Some(value.clone())
        } else if let Some(ref parent) = env_ref.parent {
            Environment::get(parent, name) // Recursive walk up the chain
        } else {
            None
        }
    }

    /// Set a variable in the scope where it's defined
    /// Returns true if found and updated, false if not found
    pub fn set(env: &EnvRef, name: &str, value: IfaValue) -> bool {
        let mut env_ref = env.borrow_mut();
        if env_ref.values.contains_key(name) {
            env_ref.values.insert(name.to_string(), value);
            true
        } else if let Some(parent) = &mut env_ref.parent {
            Environment::set(parent, name, value) // Recursive walk up
        } else {
            false
        }
    }

    /// Get all variable names in the current scope (not parents)
    pub fn local_names(env: &EnvRef) -> Vec<String> {
        env.borrow().values.keys().cloned().collect()
    }

    /// Check if a variable exists in any scope
    pub fn contains(env: &EnvRef, name: &str) -> bool {
        Environment::get(env, name).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpc_resolution() {
        // Grandparent scope
        let grandparent = Environment::new();
        Environment::define(&grandparent, "x", IfaValue::Int(1));

        // Parent scope
        let parent = Environment::with_parent(grandparent);
        Environment::define(&parent, "y", IfaValue::Int(2));

        // Child scope
        let child = Environment::with_parent(parent);
        Environment::define(&child, "z", IfaValue::Int(3));

        // Child can access all: x, y, z
        assert_eq!(Environment::get(&child, "x"), Some(IfaValue::Int(1)));
        assert_eq!(Environment::get(&child, "y"), Some(IfaValue::Int(2)));
        assert_eq!(Environment::get(&child, "z"), Some(IfaValue::Int(3)));
    }

    #[test]
    fn test_shadowing() {
        let parent = Environment::new();
        Environment::define(&parent, "x", IfaValue::Int(1));

        let child = Environment::with_parent(parent);
        Environment::define(&child, "x", IfaValue::Int(2)); // Shadow parent's x

        assert_eq!(Environment::get(&child, "x"), Some(IfaValue::Int(2)));
    }
}
