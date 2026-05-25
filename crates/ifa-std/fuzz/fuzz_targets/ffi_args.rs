#![no_main]

use ifa_std::ffi::{BoundFunction, FfiSignature, FfiValue, IfaFfi, IfaType};
use libfuzzer_sys::fuzz_target;
use std::ptr::NonNull;

fn decode_type(byte: u8) -> IfaType {
    match byte % 6 {
        0 => IfaType::I32,
        1 => IfaType::I64,
        2 => IfaType::F64,
        3 => IfaType::Str,
        4 => IfaType::OwnedStr,
        _ => IfaType::Void,
    }
}

fn decode_value(byte: u8, ty: IfaType) -> FfiValue {
    match ty {
        IfaType::I32 => FfiValue::I32(byte as i32),
        IfaType::I64 => FfiValue::I64(byte as i64),
        IfaType::F64 => FfiValue::F64(byte as f64),
        IfaType::Str | IfaType::OwnedStr => {
            if byte & 1 == 0 {
                FfiValue::Str(format!("s{}", byte))
            } else {
                FfiValue::Str(String::from_utf8_lossy(&[byte]).into_owned())
            }
        }
        IfaType::Ptr => FfiValue::Ptr(byte as usize),
        IfaType::U8 => FfiValue::U8(byte),
        IfaType::Void => FfiValue::Null,
    }
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let ffi = IfaFfi::new();
    let arg_count = (data[0] as usize % 6).min(data.len().saturating_sub(1));
    let mut arg_types = Vec::with_capacity(arg_count);
    let mut args = Vec::with_capacity(arg_count);

    for i in 0..arg_count {
        let type_byte = data.get(i + 1).copied().unwrap_or(0);
        let value_byte = data.get(i + 1 + arg_count).copied().unwrap_or(type_byte);
        let ty = decode_type(type_byte);
        if matches!(ty, IfaType::Void) {
            continue;
        }
        arg_types.push(ty);
        args.push(decode_value(value_byte, ty));
    }

    let bound = BoundFunction {
        name: "ffi_args".to_string(),
        ptr: NonNull::dangling(),
        sig: FfiSignature {
            arg_types,
            ret_type: IfaType::Void,
        },
    };

    let _ = ffi.validate_ffi_args(&bound, &args);
});
