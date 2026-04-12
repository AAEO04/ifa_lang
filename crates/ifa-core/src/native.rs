use crate::bytecode::Bytecode;
use crate::error::IfaResult;
use crate::value::IfaValue;
use crate::vm::IfaVM;
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
        self.vm.await_future(cell, self.bytecode)
    }
}

/// Registry for finding and executing native Odù functions
pub trait OduRegistry {
    /// Execute a function from a specific Odù domain
    ///
    /// # Arguments
    /// * `domain_id` - ID of the Odù domain (e.g. 1 for Ogbè, 2 for Oyẹ̀kú...)
    /// * `method_name` - Name of the method
    /// * `args` - Arguments for the function
    fn call(
        &self,
        domain_id: u8,
        method_name: &str,
        args: Vec<IfaValue>,
        ctx: &mut VmContext,
    ) -> IfaResult<IfaValue>;

    /// Execute a method on an object instance (Optional)
    fn call_method(
        &self,
        _object: &IfaValue,
        _method_idx: u16,
        _args: Vec<IfaValue>,
    ) -> IfaResult<IfaValue> {
        // Default implementation returns NotSupported or similar
        // For now, let's return Null or error
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
