use crate::bytecode::Bytecode;
use crate::error::IfaResult;
use crate::vm::IfaVM;
use ifa_types::IfaValue;
use ifa_types::value_union::FutureCell;

pub struct VmContext<'a> {
    pub vm: &'a mut IfaVM,
    pub bytecode: &'a Bytecode,
}

impl<'a> VmContext<'a> {
    pub fn spawn_task(&mut self, func: IfaValue, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        self.vm.spawn_task(func, args)
    }

    pub fn await_future(&mut self, cell: &FutureCell) -> IfaResult<IfaValue> {
        self.vm.await_future(
            &ifa_types::value_union::IfaValue::Future(cell.clone()),
            self.bytecode,
        )
    }

    pub fn resource_registry(&mut self) -> std::sync::Arc<ifa_types::registry::ResourceRegistry> {
        self.vm.resource_registry.clone()
    }

    /// H2: Spawn a new isolated actor VM on an OS thread.
    /// The actor's bytecode is shared read-only from the parent VM's current bytecode.
    pub fn spawn_actor(&mut self, handler: IfaValue) -> IfaResult<IfaValue> {
        let bytecode = std::sync::Arc::new(self.bytecode.clone());
        let new_registry = self.vm.registry.as_ref().map(|r| r.clone_registry());
        let resource_registry = std::sync::Arc::new(ifa_types::registry::ResourceRegistry::new());
        crate::actor::spawn_actor(
            handler,
            bytecode,
            self.vm.actor_table.clone(),
            new_registry,
            resource_registry,
        )
    }

    /// H2: Send a value to an actor. Non-blocking — returns error on full inbox.
    pub fn actor_send(&self, actor: &IfaValue, value: IfaValue) -> IfaResult<()> {
        crate::actor::actor_send(actor, value, &self.vm.resource_registry)
    }

    /// E1: Execute a closure or function synchronously on the VM
    pub fn call_value(&mut self, func: IfaValue, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        let val = self.vm.spawn_task(func, args)?;
        if let IfaValue::Future(cell) = val {
            self.vm.await_future(
                &ifa_types::value_union::IfaValue::Future(cell.clone()),
                self.bytecode,
            )
        } else {
            Ok(val)
        }
    }
}

/// Registry for finding and executing native Odù functions
pub trait OduRegistry: Send + Sync {
    /// Execute a function from a specific Odù domain
    ///
    /// # Arguments
    /// * `domain_id` - ID of the Odù domain (e.g. 0 for Ogbè, 1 for Oyẹ̀kú...)
    /// * `method_name` - Name of the method (canonical Yoruba or alias)
    /// * `args` - Arguments for the function
    fn call(
        &self,
        domain_id: u8,
        method_name: &str,
        args: Vec<IfaValue>,
        ctx: &mut VmContext,
    ) -> IfaResult<IfaValue>;

    /// Clone the registry for child VMs
    fn clone_registry(&self) -> Box<dyn OduRegistry>;

    /// Check if the sandbox allows side effects for a specific domain.
    /// Used by the VM for direct opcodes (Print, Import, Store) that bypass `call`.
    #[inline]
    fn check_effect(&self, _domain_id: u8) -> IfaResult<()> {
        Ok(())
    }

    /// E6: Statically-dispatched fast path — integer domain + method ID, no string lookup.
    ///
    /// Default implementation decodes the method_id back to its canonical name and
    /// delegates to `call()`. Override this in the concrete registry to get a pure
    /// integer jump table with zero string overhead.
    ///
    /// Encoding mirrors `ifa_types::methods::resolve_method_id`:
    ///   method_id high byte = domain_id, low byte = method index within domain.
    #[inline]
    fn call_fast(
        &self,
        domain_id: u8,
        method_id: u16,
        args: Vec<IfaValue>,
        ctx: &mut VmContext,
    ) -> IfaResult<IfaValue> {
        let method_name = ifa_types::methods::method_name_from_id(domain_id, method_id)
            .unwrap_or("__unknown_method__");
        self.call(domain_id, method_name, args, ctx)
    }

    /// Execute a method on an object instance (Optional)
    fn call_method(
        &self,
        _object: &IfaValue,
        _method_idx: u16,
        _args: Vec<IfaValue>,
    ) -> IfaResult<IfaValue> {
        Err(crate::error::IfaError::Custom(
            "Method calls not implemented in registry".to_string(),
        ))
    }

    /// Import a module by path
    fn import(&self, _path: &str) -> IfaResult<IfaValue> {
        Err(crate::error::IfaError::Custom(
            "Imports not implemented in registry".to_string(),
        ))
    }
}
