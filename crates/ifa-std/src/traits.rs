//! # Odù Domain Trait
//! 
//! Base trait that all 16 Odù domains implement.

/// Base trait for all Odù domains
pub trait OduDomain: Send + Sync {
    /// Get the Yoruba name of this domain
    fn name(&self) -> &'static str;
    
    /// Get the 4-bit binary code (e.g., "1100" for Ìrosù)
    fn binary(&self) -> &'static str;
    
    /// Get the English description
    fn description(&self) -> &'static str;
    
    /// Get help text with all available methods
    fn help(&self) -> String;
}

/// Macro to implement OduDomain boilerplate
#[macro_export]
macro_rules! impl_odu_domain {
    ($struct:ident, $name:expr, $binary:expr, $desc:expr) => {
        impl $crate::traits::OduDomain for $struct {
            fn name(&self) -> &'static str { $name }
            fn binary(&self) -> &'static str { $binary }
            fn description(&self) -> &'static str { $desc }
            fn help(&self) -> String {
                format!(
                    "=== {} ({}) - {} ===",
                    self.name(), self.binary(), self.description()
                )
            }
        }
    };
}
