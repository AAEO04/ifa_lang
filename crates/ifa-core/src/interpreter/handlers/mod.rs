//! # Ifá-Lang Domain Handlers
//!
//! Trait-based handler architecture for Odù domain operations.
//! Each handler implements the `OduHandler` trait to process domain-specific methods.
//!
//! This modular approach replaces the monolithic `execute_odu_call()` function,
//! improving maintainability and extensibility.

use std::collections::HashMap;

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

// Import Environment
use super::environment::Environment;

// Sub-modules containing domain-specific handlers (16 core Odù)
mod irosu;    // 1100 - Console I/O
mod ogbe;     // 1111 - System/Lifecycle
mod obara;    // 1000 - Math (Add/Mul)
mod oturupon; // 0010 - Math (Sub/Div)
mod ika;      // 0100 - Strings
mod oyeku;    // 0000 - Exit/Sleep
mod owonrin;  // 0011 - Random
mod ogunda;   // 1110 - Arrays/Lists
mod iwori;    // 0110 - Time/DateTime
mod okanran;  // 0001 - Errors/Assertions
mod otura;    // 1011 - Networking
mod odi;      // 1001 - Files/Database
mod osa;      // 0111 - Concurrency
mod ofun;     // 0101 - Permissions/Reflection
mod irete;    // 1101 - Crypto/Security
mod ose;      // 1010 - Graphics/UI

// Infrastructure handlers
mod ohun;     // Audio I/O
mod fidio;    // Video I/O

// Re-export handlers
pub use irosu::IrosuHandler;
pub use ogbe::OgbeHandler;
pub use obara::ObaraHandler;
pub use oturupon::OturuponHandler;
pub use ika::IkaHandler;
pub use oyeku::OyekuHandler;
pub use owonrin::OwonrinHandler;
pub use ogunda::OgundaHandler;
pub use iwori::IworiHandler;
pub use okanran::OkanranHandler;
pub use otura::OturaHandler;
pub use odi::OdiHandler;
pub use osa::OsaHandler;
pub use ofun::OfunHandler;
pub use irete::IreteHandler;
pub use ose::OseHandler;

// Infrastructure handlers
pub use ohun::OhunHandler;
pub use fidio::FidioHandler;

/// Trait for domain-specific operation handlers.
///
/// Each Odù domain implements this trait to handle its methods.
/// The interpreter dispatches to the appropriate handler based on the domain.
pub trait OduHandler: Send + Sync {
    /// Returns the domain this handler is responsible for.
    fn domain(&self) -> OduDomain;
    
    /// Execute a method call on this domain.
    fn call(
        &self, 
        method: &str, 
        args: Vec<IfaValue>, 
        env: &mut Environment
    ) -> IfaResult<IfaValue>;
    
    /// Returns the list of methods this handler supports.
    fn methods(&self) -> &'static [&'static str];
}

/// Registry of domain handlers.
pub struct HandlerRegistry {
    handlers: HashMap<OduDomain, Box<dyn OduHandler>>,
}

impl HandlerRegistry {
    /// Create a new registry with all built-in handlers registered.
    pub fn new() -> Self {
        let mut handlers: HashMap<OduDomain, Box<dyn OduHandler>> = HashMap::new();
        
        // Register all 16 core Odù handlers
        handlers.insert(OduDomain::Irosu, Box::new(IrosuHandler));
        handlers.insert(OduDomain::Ogbe, Box::new(OgbeHandler));
        handlers.insert(OduDomain::Obara, Box::new(ObaraHandler));
        handlers.insert(OduDomain::Oturupon, Box::new(OturuponHandler));
        handlers.insert(OduDomain::Ika, Box::new(IkaHandler));
        handlers.insert(OduDomain::Oyeku, Box::new(OyekuHandler));
        handlers.insert(OduDomain::Owonrin, Box::new(OwonrinHandler));
        handlers.insert(OduDomain::Ogunda, Box::new(OgundaHandler));
        handlers.insert(OduDomain::Iwori, Box::new(IworiHandler));
        handlers.insert(OduDomain::Okanran, Box::new(OkanranHandler));
        handlers.insert(OduDomain::Otura, Box::new(OturaHandler));
        handlers.insert(OduDomain::Odi, Box::new(OdiHandler));
        handlers.insert(OduDomain::Osa, Box::new(OsaHandler));
        handlers.insert(OduDomain::Ofun, Box::new(OfunHandler));
        handlers.insert(OduDomain::Irete, Box::new(IreteHandler));
        handlers.insert(OduDomain::Ose, Box::new(OseHandler));
        
        // Infrastructure handlers
        handlers.insert(OduDomain::Ohun, Box::new(OhunHandler));
        handlers.insert(OduDomain::Fidio, Box::new(FidioHandler));
        
        HandlerRegistry { handlers }
    }
    
    /// Get a handler for the given domain.
    pub fn get(&self, domain: &OduDomain) -> Option<&dyn OduHandler> {
        self.handlers.get(domain).map(|b| b.as_ref())
    }
    
    /// Execute an Odù call using the appropriate handler.
    pub fn dispatch(
        &self,
        domain: OduDomain,
        method: &str,
        args: Vec<IfaValue>,
        env: &mut Environment,
    ) -> IfaResult<IfaValue> {
        match self.handlers.get(&domain) {
            Some(handler) => handler.call(method, args, env),
            None => Err(IfaError::Runtime(format!(
                "No handler registered for domain {:?}",
                domain
            ))),
        }
    }
    
    /// List all registered domains.
    pub fn domains(&self) -> Vec<OduDomain> {
        self.handlers.keys().cloned().collect()
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
