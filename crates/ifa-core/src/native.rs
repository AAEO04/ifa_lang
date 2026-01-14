use crate::error::IfaResult;
use crate::value::IfaValue;

/// Registry for finding and executing native Odù functions
pub trait OduRegistry {
    /// Execute a function from a specific Odù domain
    /// 
    /// # Arguments
    /// * `domain_id` - ID of the Odù domain (e.g. 1 for Ogbè, 2 for Oyẹ̀kú...)
    /// * `method_name` - Name of the method
    /// * `args` - Arguments for the function
    fn call(&self, domain_id: u8, method_name: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue>;

    /// Execute a method on an object instance (Optional)
    fn call_method(&self, _object: &IfaValue, _method_idx: u16, _args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        // Default implementation returns NotSupported or similar
        // For now, let's return Null or error
        Err(crate::error::IfaError::Custom("Method calls not implemented in registry".to_string()))
    }

    /// Import a module by path
    fn import(&self, _path: &str) -> IfaResult<IfaValue> {
        Err(crate::error::IfaError::Custom("Imports not implemented in registry".to_string()))
    }
}
