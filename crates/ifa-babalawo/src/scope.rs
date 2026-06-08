use ifa_types::ast::{Span, TypeHint, Visibility};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub type_hint: Option<TypeHint>,
    pub visibility: Visibility,
    pub domain: Option<String>,
    pub span: Span,
    pub is_const: bool,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub values: HashMap<String, VarInfo>,
    pub parent: Option<Box<Scope>>,
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            values: HashMap::new(),
            parent: None,
        }
    }

    pub fn child(parent: Scope) -> Self {
        Scope {
            values: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn define(&mut self, name: &str, info: VarInfo) {
        self.values.insert(name.to_string(), info);
    }

    pub fn resolve(&self, name: &str) -> Option<&VarInfo> {
        self.values
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.resolve(name)))
    }

    pub fn resolve_mut(&mut self, name: &str) -> Option<&mut VarInfo> {
        if self.values.contains_key(name) {
            self.values.get_mut(name)
        } else if let Some(ref mut parent) = self.parent {
            parent.resolve_mut(name)
        } else {
            None
        }
    }

    pub fn set(&mut self, name: &str, info: VarInfo) -> bool {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), info);
            true
        } else if let Some(ref mut parent) = self.parent {
            parent.set(name, info)
        } else {
            false
        }
    }

    pub fn is_const(&self, name: &str) -> bool {
        self.resolve(name)
            .map(|info| info.is_const)
            .unwrap_or(false)
    }

    pub fn local_names(&self) -> Vec<String> {
        self.values.keys().cloned().collect()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.resolve(name).is_some()
    }
}

#[derive(Debug, Clone)]
pub struct ScopeChain {
    current: Scope,
}

impl ScopeChain {
    pub fn new() -> Self {
        ScopeChain {
            current: Scope::new(),
        }
    }

    pub fn enter_scope(&mut self) {
        let new_current = Scope::child(std::mem::take(&mut self.current));
        self.current = new_current;
    }

    pub fn exit_scope(&mut self) {
        if let Some(parent) = self.current.parent.take() {
            self.current = *parent;
        }
    }

    pub fn define(&mut self, name: &str, info: VarInfo) {
        self.current.define(name, info);
    }

    pub fn resolve(&self, name: &str) -> Option<&VarInfo> {
        self.current.resolve(name)
    }

    pub fn set(&mut self, name: &str, info: VarInfo) -> bool {
        self.current.set(name, info)
    }

    pub fn is_const(&self, name: &str) -> bool {
        self.current.is_const(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.current.contains(name)
    }
}

impl Default for ScopeChain {
    fn default() -> Self {
        Self::new()
    }
}
