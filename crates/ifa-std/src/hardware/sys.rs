//! System Info Domain (Domain 29) wrapper
//! Bridges the VM dispatch to the OS kernel and sysinfo utilities.

use ifa_types::value_union::IfaValue;
use ifa_types::{IfaError, IfaResult};

pub fn dispatch(method: &str, _args: Vec<IfaValue>) -> IfaResult<IfaValue> {
    match method {
        "num_cores" | "cores" => Ok(IfaValue::int(ifa_infra::kernel::num_cores() as i64)),
        #[cfg(feature = "sysinfo")]
        "total_memory" | "mem_total" => Ok(IfaValue::int(ifa_infra::kernel::total_memory() as i64)),
        #[cfg(feature = "sysinfo")]
        "available_memory" | "mem_available" => {
            Ok(IfaValue::int(ifa_infra::kernel::available_memory() as i64))
        }
        #[cfg(feature = "sysinfo")]
        "uptime" => Ok(IfaValue::int(ifa_infra::kernel::uptime() as i64)),
        #[cfg(not(feature = "sysinfo"))]
        "total_memory" | "mem_total" | "available_memory" | "mem_available" | "uptime" => {
            Err(IfaError::Custom("Sys: sysinfo feature not enabled".into()))
        }
        _ => Err(IfaError::Custom(format!(
            "Sys: unknown method '{}'",
            method
        ))),
    }
}
