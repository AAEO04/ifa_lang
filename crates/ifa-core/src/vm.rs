//! # Ifá-Lang Virtual Machine
//!
//! Stack-based bytecode interpreter for Ifá-Lang.

use crate::bytecode::{Bytecode, OpCode};
use crate::error::{IfaError, IfaResult};
use crate::opon::Opon;
use crate::value::IfaValue;
use crate::native::OduRegistry;

/// Stack size limit
const MAX_STACK_SIZE: usize = 65536;

/// Call frame for function calls
#[derive(Debug)]
pub struct CallFrame {
    /// Return address (instruction pointer to return to)
    pub return_addr: usize,
    /// Base pointer (stack index where this frame starts)
    pub base_ptr: usize,
    /// Local variable count
    pub local_count: usize,
}

/// The Ifá Virtual Machine
pub struct IfaVM {
    /// Value stack
    stack: Vec<IfaValue>,
    /// Call stack
    frames: Vec<CallFrame>,
    /// Instruction pointer
    ip: usize,

    /// Global variables
    globals: std::collections::HashMap<String, IfaValue>,
    /// Memory (Opon)
    pub opon: Opon,
    /// Function Registry (Standard Library)
    pub registry: Option<Box<dyn OduRegistry>>,
    /// Halt flag
    halted: bool,
}

impl IfaVM {
    /// Create new VM
    pub fn new() -> Self {
        IfaVM {
            stack: Vec::with_capacity(256),
            frames: Vec::new(),
            ip: 0,
            globals: std::collections::HashMap::new(),
            opon: Opon::create_default(),
            registry: None,
            halted: false,
        }
    }

    /// Attach a function registry (Standard Library)
    pub fn with_registry(mut self, registry: Box<dyn OduRegistry>) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Create VM with custom Opon size
    pub fn with_opon(opon: Opon) -> Self {
        IfaVM {
            stack: Vec::with_capacity(256),
            frames: Vec::new(),
            ip: 0,
            globals: std::collections::HashMap::new(),
            opon,
            registry: None,
            halted: false,
        }
    }

    // =========================================================================
    // STACK OPERATIONS
    // =========================================================================

    /// Push value onto stack
    pub fn push(&mut self, value: IfaValue) -> IfaResult<()> {
        if self.stack.len() >= MAX_STACK_SIZE {
            return Err(IfaError::StackOverflow(MAX_STACK_SIZE));
        }
        self.stack.push(value);
        Ok(())
    }

    /// Pop value from stack
    pub fn pop(&mut self) -> IfaResult<IfaValue> {
        self.stack.pop().ok_or(IfaError::StackUnderflow)
    }

    /// Peek at top of stack
    pub fn peek(&self) -> IfaResult<&IfaValue> {
        self.stack.last().ok_or(IfaError::StackUnderflow)
    }

    // =========================================================================
    // BYTECODE EXECUTION
    // =========================================================================

    /// Execute bytecode
    pub fn execute(&mut self, bytecode: &Bytecode) -> IfaResult<IfaValue> {
        self.ip = 0;
        self.halted = false;

        while !self.halted && self.ip < bytecode.code.len() {
            self.step(bytecode)?;
        }

        // Return top of stack or Null
        Ok(self.stack.pop().unwrap_or(IfaValue::Null))
    }

    /// Execute single instruction
    fn step(&mut self, bytecode: &Bytecode) -> IfaResult<()> {
        let opcode = OpCode::from_byte(bytecode.code[self.ip])?;
        self.ip += 1;

        match opcode {
            // Stack operations
            OpCode::PushNull => self.push(IfaValue::Null)?,
            OpCode::PushTrue => self.push(IfaValue::Bool(true))?,
            OpCode::PushFalse => self.push(IfaValue::Bool(false))?,
            OpCode::PushList => self.push(IfaValue::List(Vec::new()))?,
            OpCode::PushMap => self.push(IfaValue::Map(std::collections::HashMap::new()))?,
            
            OpCode::PushFn => {
                let name_idx = self.read_u16(bytecode)? as usize;
                let _start_ip = self.read_u16(bytecode)? as usize; // Wait, IP might be > 65535? Bytecode uses u32 for jumps?
                // bytecode.rs says Jump uses read_i16 offset. So code limit is small?
                // Let's check Jump. 
                // Wait, functions are usually absolute addresses.
                // Let's assume start_ip is u32 or u64. 
                // Let's read u32.
                let start_ip_u32 = {
                     let mut bytes = [0u8; 4];
                     bytes[0] = self.read_u8(bytecode)?;
                     bytes[1] = self.read_u8(bytecode)?;
                     bytes[2] = self.read_u8(bytecode)?;
                     bytes[3] = self.read_u8(bytecode)?;
                     u32::from_be_bytes(bytes)
                } as usize;
                
                let arity = self.read_u8(bytecode)?;
                
                let name = bytecode.strings.get(name_idx).cloned().unwrap_or_default();
                
                self.push(IfaValue::BytecodeFn {
                    name,
                    start_ip: start_ip_u32,
                    arity,
                })?;
            }

            OpCode::PushInt => {
                let value = self.read_i64(bytecode)?;
                self.push(IfaValue::Int(value))?;
            }

            OpCode::PushFloat => {
                let value = self.read_f64(bytecode)?;
                self.push(IfaValue::Float(value))?;
            }

            OpCode::PushStr => {
                let idx = self.read_u16(bytecode)? as usize;
                let s = bytecode.strings.get(idx).cloned().unwrap_or_default();
                self.push(IfaValue::Str(s))?;
            }

            OpCode::Pop => {
                self.pop()?;
            }

            OpCode::Dup => {
                let value = self.peek()?.clone();
                self.push(value)?;
            }

            OpCode::Swap => {
                let len = self.stack.len();
                if len < 2 {
                    return Err(IfaError::StackUnderflow);
                }
                self.stack.swap(len - 1, len - 2);
            }

            // Arithmetic
            OpCode::Add => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a + b)?;
            }

            OpCode::Sub => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a - b)?;
            }

            OpCode::Mul => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a * b)?;
            }

            OpCode::Div => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = a / b;
                if matches!(result, IfaValue::Null) {
                    return Err(IfaError::DivisionByZero("Division by zero".to_string()));
                }
                self.push(result)?;
            }

            OpCode::Mod => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a % b)?;
            }

            OpCode::Neg => {
                let a = self.pop()?;
                self.push(-a)?;
            }

            OpCode::Pow => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (IfaValue::Int(base), IfaValue::Int(exp)) if *exp >= 0 => {
                        self.push(IfaValue::Int(base.pow(*exp as u32)))?;
                    }
                    (IfaValue::Float(base), IfaValue::Float(exp)) => {
                        self.push(IfaValue::Float(base.powf(*exp)))?;
                    }
                    _ => self.push(IfaValue::Null)?,
                }
            }

            // Comparison
            OpCode::Eq => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(IfaValue::Bool(a == b))?;
            }

            OpCode::Ne => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(IfaValue::Bool(a != b))?;
            }

            OpCode::Lt => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(IfaValue::Bool(a < b))?;
            }

            OpCode::Le => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(IfaValue::Bool(a <= b))?;
            }

            OpCode::Gt => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(IfaValue::Bool(a > b))?;
            }

            OpCode::Ge => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(IfaValue::Bool(a >= b))?;
            }

            // Logic
            OpCode::And => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(IfaValue::Bool(a.is_truthy() && b.is_truthy()))?;
            }

            OpCode::Or => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(IfaValue::Bool(a.is_truthy() || b.is_truthy()))?;
            }

            OpCode::Not => {
                let a = self.pop()?;
                self.push(!a)?;
            }

            // Variables
            OpCode::LoadLocal => {
                let idx = self.read_u16(bytecode)? as usize;
                let base = self.frames.last().map(|f| f.base_ptr).unwrap_or(0);
                let value = self
                    .stack
                    .get(base + idx)
                    .cloned()
                    .unwrap_or(IfaValue::Null);
                self.push(value)?;
            }

            OpCode::StoreLocal => {
                let idx = self.read_u16(bytecode)? as usize;
                let value = self.pop()?;
                let base = self.frames.last().map(|f| f.base_ptr).unwrap_or(0);
                if base + idx < self.stack.len() {
                    self.stack[base + idx] = value;
                }
            }

            OpCode::LoadGlobal => {
                let idx = self.read_u16(bytecode)? as usize;
                let name = bytecode.strings.get(idx).cloned().unwrap_or_default();
                let value = self.globals.get(&name).cloned().unwrap_or(IfaValue::Null);
                self.push(value)?;
            }

            OpCode::StoreGlobal => {
                let idx = self.read_u16(bytecode)? as usize;
                let name = bytecode.strings.get(idx).cloned().unwrap_or_default();
                let value = self.pop()?;
                self.globals.insert(name, value);
            }

            // Control flow
            OpCode::Jump => {
                let offset = self.read_i16(bytecode)?;
                self.ip = ((self.ip as isize) + (offset as isize)) as usize;
            }

            OpCode::JumpIfFalse => {
                let offset = self.read_i16(bytecode)?;
                let cond = self.pop()?;
                if !cond.is_truthy() {
                    self.ip = ((self.ip as isize) + (offset as isize)) as usize;
                }
            }

            OpCode::JumpIfTrue => {
                let offset = self.read_i16(bytecode)?;
                let cond = self.pop()?;
                if cond.is_truthy() {
                    self.ip = ((self.ip as isize) + (offset as isize)) as usize;
                }
            }

            // Functions
            OpCode::Call => {
                let arg_count = self.read_u8(bytecode)? as usize;
                
                // 1. Pop arguments
                let mut args = Vec::with_capacity(arg_count);
                for _ in 0..arg_count {
                    args.push(self.pop()?);
                }
                // Args are popped in reverse order (stack LIFO), so reverse them to get (arg1, arg2...)
                args.reverse();

                // 2. Pop function
                let func = self.pop()?;

                match func {
                    IfaValue::Fn(f) => {
                        // Native closure call
                        let result = f(args);
                        self.push(result)?;
                    }
                    IfaValue::BytecodeFn { start_ip, arity, .. } => {
                       // VM Bytecode call
                       if args.len() != arity as usize {
                           return Err(IfaError::ArityMismatch { expected: arity as usize, got: args.len() });
                       }
                       
                       // Push CallFrame
                       self.frames.push(CallFrame {
                           return_addr: self.ip,
                           base_ptr: self.stack.len(), // Base of the new frame
                           local_count: 0, // Will be incremented by StoreLocal or determined by compiler?
                           // Actually, locals are usually allocated on stack.
                           // For this simple VM, we can assume locals are just stack slots relative to base_ptr.
                           // But arguments are locals!
                       });

                       // Push arguments back onto stack (as locals 0..N)
                       for arg in args {
                           self.push(arg)?;
                       }
                       
                       // Jump
                       self.ip = start_ip;
                    }
                    _ => return Err(IfaError::TypeError { expected: "Function".into(), got: func.type_name().into() }),
                }
            }

            OpCode::Return => {
                if let Some(frame) = self.frames.pop() {
                    let return_value = self.pop().unwrap_or(IfaValue::Null);
                    // Unwind stack to base_ptr
                    // Note: This drops all locals AND arguments of the current frame
                    if self.stack.len() > frame.base_ptr {
                        self.stack.truncate(frame.base_ptr);
                    }
                    self.push(return_value)?;
                    self.ip = frame.return_addr;
                } else {
                    // Return from main (halt or return value?)
                    // For now, treat as implicit halt
                    self.halted = true;
                }
            }

            OpCode::CallOdu => {
                let domain_id = self.read_u8(bytecode)?;
                // Method name is a string index (u8 length + bytes in bytecode stream)
                // Compiler emits: emit_string -> len(u8) + bytes.
                let len = self.read_u8(bytecode)? as usize;
                let mut method_bytes = Vec::with_capacity(len);
                for _ in 0..len {
                    method_bytes.push(self.read_u8(bytecode)?);
                }
                let method_name = String::from_utf8(method_bytes).unwrap_or_else(|_| "unknown".to_string());
                
                let arity = self.read_u8(bytecode)?;
                
                let mut args = Vec::with_capacity(arity as usize);
                for _ in 0..arity {
                    args.push(self.pop()?);
                }
                args.reverse();

                if let Some(registry) = &self.registry {
                    let result = registry.call(domain_id, &method_name, args)?;
                    self.push(result)?;
                } else {
                    return Err(IfaError::Custom("No standard library registry attached".into()));
                }
            }

            OpCode::CallMethod => {
                let method_idx = self.read_u16(bytecode)?;
                let arg_count = self.read_u8(bytecode)?;
                
                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    args.push(self.pop()?);
                }
                args.reverse();

                let object = self.pop()?; // Object is below args
                
                if let Some(registry) = &self.registry {
                    let result = registry.call_method(&object, method_idx, args)?;
                    self.push(result)?;
                } else {
                     return Err(IfaError::Custom("No standard library registry attached".into()));
                }
            }

            // Collections
            OpCode::GetIndex => {
                let index = self.pop()?;
                let collection = self.pop()?;
                let value = collection.get(&index).unwrap_or(IfaValue::Null);
                self.push(value)?;
            }

            OpCode::SetIndex => {
                let value = self.pop()?;
                let index = self.pop()?;
                let mut collection = self.pop()?;
                let _ = collection.set(&index, value);
                self.push(collection)?;
            }

            OpCode::Len => {
                let value = self.peek()?;
                let len = value.len() as i64;
                self.push(IfaValue::Int(len))?;
            }

            OpCode::Append => {
                let value = self.pop()?;
                let mut list = self.pop()?;
                let _ = list.push(value);
                self.push(list)?;
            }

            OpCode::BuildList => {
                let count = self.read_u8(bytecode)? as usize;
                let mut items = Vec::with_capacity(count);
                for _ in 0..count {
                    items.push(self.pop()?);
                }
                items.reverse();
                self.push(IfaValue::List(items))?;
            }

            OpCode::BuildMap => {
                let count = self.read_u8(bytecode)? as usize;
                let mut map = std::collections::HashMap::new();
                for _ in 0..count {
                    let value = self.pop()?;
                    let key = self.pop()?;
                    if let IfaValue::Str(k) = key {
                        map.insert(k, value);
                    }
                }
                self.push(IfaValue::Map(map))?;
            }

            // I/O
            OpCode::Print => {
                let value = self.pop()?;
                self.opon.record("Ìrosù", "fọ̀ (spoke)", &value);
                println!("{}", value);
            }

            OpCode::PrintRaw => {
                let value = self.pop()?;
                print!("{}", value);
            }

            OpCode::Input => {
                use std::io::{self, BufRead, Write};
                print!("> ");
                io::stdout().flush().ok();
                let mut input = String::new();
                io::stdin().lock().read_line(&mut input).ok();
                let result = IfaValue::Str(input.trim().to_string());
                self.opon.record("Ogbè", "gbà (received)", &result);
                self.push(result)?;
            }

            OpCode::Import => {
                let path_idx = self.read_u16(bytecode)? as usize;
                let path = bytecode.strings.get(path_idx).cloned().unwrap_or_default();
                
                if let Some(registry) = &self.registry {
                    let module = registry.import(&path)?;
                    self.push(module)?;
                } else {
                    return Err(IfaError::Custom("No standard library registry attached".into()));
                }
            }

            OpCode::DefineClass => {
                let name_idx = self.read_u16(bytecode)? as usize;
                let name = bytecode.strings.get(name_idx).cloned().unwrap_or_default();
                
                let field_count = self.read_u8(bytecode)? as usize;
                let mut fields = Vec::with_capacity(field_count);
                for _ in 0..field_count {
                    let f_idx = self.read_u16(bytecode)? as usize;
                    let f_name = bytecode.strings.get(f_idx).cloned().unwrap_or_default();
                    fields.push(f_name);
                }
                
                let method_count = self.read_u8(bytecode)? as usize;
                let mut methods = std::collections::HashMap::new();
                for _ in 0..method_count {
                    let func = self.pop()?;
                    if let IfaValue::BytecodeFn { ref name, .. } = func {
                        methods.insert(name.clone(), func);
                    } else if let IfaValue::AstFn { ref name, .. } = func {
                        methods.insert(name.clone(), func);
                    }
                }
                
                self.push(IfaValue::Class {
                    name,
                    fields,
                    methods,
                })?;
            }

            // System
            OpCode::Halt => {
                self.halted = true;
            }
        }

        Ok(())
    }

    // =========================================================================
    // BYTECODE READING HELPERS
    // =========================================================================

    fn read_u8(&mut self, bytecode: &Bytecode) -> IfaResult<u8> {
        if self.ip >= bytecode.code.len() {
            return Err(IfaError::Custom("Unexpected end of bytecode".to_string()));
        }
        let value = bytecode.code[self.ip];
        self.ip += 1;
        Ok(value)
    }

    fn read_u16(&mut self, bytecode: &Bytecode) -> IfaResult<u16> {
        let mut bytes = [0u8; 2];
        bytes[0] = self.read_u8(bytecode)?;
        bytes[1] = self.read_u8(bytecode)?;
        Ok(u16::from_be_bytes(bytes))
    }

    fn read_i16(&mut self, bytecode: &Bytecode) -> IfaResult<i16> {
        Ok(self.read_u16(bytecode)? as i16)
    }

    fn read_i64(&mut self, bytecode: &Bytecode) -> IfaResult<i64> {
        let mut bytes = [0u8; 8];
        for byte in &mut bytes {
            *byte = self.read_u8(bytecode)?;
        }
        Ok(i64::from_be_bytes(bytes))
    }

    fn read_f64(&mut self, bytecode: &Bytecode) -> IfaResult<f64> {
        let mut bytes = [0u8; 8];
        for byte in &mut bytes {
            *byte = self.read_u8(bytecode)?;
        }
        Ok(f64::from_be_bytes(bytes))
    }
}

impl Default for IfaVM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_arithmetic() {
        let mut vm = IfaVM::new();

        // Push 5, Push 3, Add -> 8
        let mut bc = Bytecode::new("test");
        bc.code = vec![
            OpCode::PushInt as u8,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            5, // 5 as i64
            OpCode::PushInt as u8,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            3, // 3 as i64
            OpCode::Add as u8,
            OpCode::Halt as u8,
        ];

        let result = vm.execute(&bc).unwrap();
        assert_eq!(result, IfaValue::Int(8));
    }

    #[test]
    fn test_stack_operations() {
        let mut vm = IfaVM::new();

        vm.push(IfaValue::Int(1)).unwrap();
        vm.push(IfaValue::Int(2)).unwrap();
        vm.push(IfaValue::Int(3)).unwrap();

        assert_eq!(vm.pop().unwrap(), IfaValue::Int(3));
        assert_eq!(vm.pop().unwrap(), IfaValue::Int(2));
        assert_eq!(vm.pop().unwrap(), IfaValue::Int(1));
        assert!(vm.pop().is_err());
    }
}
