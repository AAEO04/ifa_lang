use ifa_types::domain::OduDomain;
use ifa_types::odu_metadata::{is_valid_odu_method, odu_methods};

#[derive(Debug, Clone)]
pub struct OduMethodDescriptor {
    pub domain: OduDomain,
    pub method: &'static str,
    pub yoruba_alias: &'static str,
    pub english_alias: &'static str,
    pub description: &'static str,
}

pub fn validate_odu_call(domain: &OduDomain, method: &str) -> Option<&'static str> {
    if is_valid_odu_method(domain, method) {
        None
    } else {
        let methods = odu_methods(domain);
        if methods.is_empty() {
            Some("Unknown domain — no methods registered")
        } else {
            Some("Method not found for this domain")
        }
    }
}

pub fn domain_has_method(domain: &OduDomain, method: &str) -> bool {
    is_valid_odu_method(domain, method)
}

pub fn list_methods_for_domain(domain: &OduDomain) -> &'static [&'static str] {
    odu_methods(domain)
}
