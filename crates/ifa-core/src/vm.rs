//! # Ifá-Lang Virtual Machine
//!
//! Stack-based bytecode interpreter for Ifá-Lang.

use crate::bytecode::{Bytecode, OpCode};
use crate::error::{IfaError, IfaResult};
use crate::native::OduRegistry;
use crate::opon::Opon;
use ifa_types::value_union::IfaValue;

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

/// Recovery frame for exception handling (The Shield of Ọ̀kànràn)
#[derive(Debug, Clone, Copy)]
pub struct RecoveryFrame {
    /// Stack depth to restore to
    pub stack_depth: usize,
    /// Call frame depth to restore to
    pub call_depth: usize,
    /// Instruction pointer to jump to (Catch Handler)
    pub catch_ip: usize,
}

use crate::vm_ikin::Ikin;
use crate::vm_iroke;

/// The Ifá Virtual Machine
pub struct IfaVM {
    /// Value stack
    stack: Vec<IfaValue>,
    /// Call stack
    frames: Vec<CallFrame>,
    /// Instruction pointer
    pub ip: usize,

    /// Global variables
    globals: std::collections::HashMap<String, IfaValue>,
    /// Memory (Opon)
    pub opon: Opon,
    /// Function Registry (Standard Library)
    pub registry: Option<Box<dyn OduRegistry>>,
    /// Halt flag
    halted: bool,
    /// Execution ticks (for GC/Interrupts)
    pub ticks: usize,
    
    /// Recovery stack (for Try/Catch)
    recovery_stack: Vec<RecoveryFrame>,

    /// The Sacred Nuts - Runtime Constant Pool
    pub ikin: Ikin,
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
            ticks: 0,
            recovery_stack: Vec::with_capacity(32),
            ikin: Ikin::new(),
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
            ticks: 0,
            recovery_stack: Vec::with_capacity(32),
            ikin: Ikin::new(),
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

    /// Pop an integer from the stack


    // =========================================================================
    // BYTECODE EXECUTION
    // =========================================================================

    /// Execute bytecode
    pub fn execute(&mut self, bytecode: &Bytecode) -> IfaResult<IfaValue> {
        self.ip = 0;
        self.halted = false;
        
        // Phase 1: Consult the Nuts (Load Constants)
        self.ikin.load_from_bytecode(bytecode);

        while !self.halted && self.ip < bytecode.code.len() {
            if let Err(e) = self.step(bytecode) {
                // The Shield of Ọ̀kànràn: Attempt recovery before crashing
                // Pass reference to avoid cloning unless we actually recover
                if self.attempt_recovery(&e)? {
                     continue;
                }
                return Err(e);
            }
        }

        // Return top of stack or Null
        Ok(self.stack.pop().unwrap_or(IfaValue::null()))
    }

    /// Attempt to recover from a runtime error using the Shield of Ọ̀kànràn
    fn attempt_recovery(&mut self, error: &IfaError) -> IfaResult<bool> {
        if let Some(frame) = self.recovery_stack.pop() {
            // 1. Restore stacks
            if self.stack.len() > frame.stack_depth {
                self.stack.truncate(frame.stack_depth); // Drop triggers Ebo cleanup
            }
            if self.frames.len() > frame.call_depth {
                self.frames.truncate(frame.call_depth);
            }

            // 2. Convert Error to Value (Result::Err)
            // For now, convert error string to IfaValue::Str
            let err_val = IfaValue::str(error.to_string());
            let result = IfaValue::Result(false, Box::new(err_val));

            // 3. Push Result and Jump
            self.push(result)?;
            self.ip = frame.catch_ip;
            
            Ok(true) // Recovered
        } else {
            Ok(false) // No shield found, crash
        }
    }
    /// Execute single instruction (The Step of Iroke)
    fn step(&mut self, bytecode: &Bytecode) -> IfaResult<()> {
        let opcode = vm_iroke::tap(self, bytecode)?;

        match opcode {
            // Stack operations
            OpCode::PushNull => self.push(IfaValue::null())?,
            OpCode::PushTrue => self.push(IfaValue::bool(true))?,
            OpCode::PushFalse => self.push(IfaValue::bool(false))?,
            OpCode::PushList => {
                self.push(IfaValue::list(Vec::new()))? 
            },
            OpCode::PushMap => {
                self.push(IfaValue::map(std::collections::HashMap::new()))?
            },

            OpCode::PushFn => {
                // AST-mode function values are not supported in the bytecode VM.
                // Functions are compiled to bytecode chunks and called via OpCode::Call.
                // Pushing a raw function literal as a value is not yet implemented.
                return Err(IfaError::Runtime(
                    "OpCode::PushFn: function-as-value is not yet supported in the bytecode VM".into()
                ));
            }

            OpCode::PushInt => {
                let value = self.read_i64(bytecode)?;
                self.push(IfaValue::int(value))?;
            }

            OpCode::PushFloat => {
                let value = self.read_f64(bytecode)?;
                self.push(IfaValue::float(value))?;
            }

            OpCode::PushStr => {
                let idx = self.read_u16(bytecode)? as usize;
                // Use Ikin for O(1) Arc access
                let arc = self.ikin.consult_string(idx)
                    .ok_or_else(|| IfaError::Custom("Invalid string constant index in Ikin".into()))?;
                self.push(IfaValue::Str(arc.clone()))?;
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
                match (a, b) {
                    (IfaValue::Int(ia), IfaValue::Int(ib)) => self.push(IfaValue::int(ia + ib))?,
                    (IfaValue::Float(fa), IfaValue::Float(fb)) => self.push(IfaValue::float(fa + fb))?,
                    (IfaValue::Str(lhs), IfaValue::Str(rhs)) => {
                         // Concat
                         let mut s = String::with_capacity(lhs.len() + rhs.len());
                         s.push_str(&lhs);
                         s.push_str(&rhs);
                         self.push(IfaValue::str(s))?;
                    }
                    _ => return Err(IfaError::TypeError { expected: "Int/Float/Str".into(), got: "Mismatch".into() })
                }
            }

            OpCode::Sub => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                     (IfaValue::Int(ia), IfaValue::Int(ib)) => self.push(IfaValue::int(ia - ib))?,
                     (IfaValue::Float(fa), IfaValue::Float(fb)) => self.push(IfaValue::float(fa - fb))?,
                     _ => return Err(IfaError::TypeError { expected: "Int/Float".into(), got: "Mismatch".into() })
                }
            }

            OpCode::Mul => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                     (IfaValue::Int(ia), IfaValue::Int(ib)) => self.push(IfaValue::int(ia * ib))?,
                     (IfaValue::Float(fa), IfaValue::Float(fb)) => self.push(IfaValue::float(fa * fb))?,
                     _ => return Err(IfaError::TypeError { expected: "Int/Float".into(), got: "Mismatch".into() })
                }
            }

            OpCode::Div => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                     (IfaValue::Int(ia), IfaValue::Int(ib)) => {
                         if ib == 0 { return Err(IfaError::Runtime("Division by zero".into())); }
                         self.push(IfaValue::int(ia / ib))?
                     },
                     (IfaValue::Float(fa), IfaValue::Float(fb)) => {
                        if fb == 0.0 { return Err(IfaError::Runtime("Division by zero".into())); }
                        self.push(IfaValue::float(fa / fb))?
                     },
                     _ => return Err(IfaError::TypeError { expected: "Int/Float".into(), got: "Mismatch".into() })
                }
            }





            // Variables
            OpCode::LoadLocal => {
                let idx = self.read_u16(bytecode)? as usize;
                let base = self.frames.last().map(|f| f.base_ptr).unwrap_or(0);
                
                // Use strict checking for local access
                let value = self.stack
                    .get(base + idx)
                    .cloned()
                    .ok_or(IfaError::Runtime("Local variable access check failed (stack underflow/index error)".into()));

                match value {
                    Ok(v) => self.push(v)?,
                    Err(_) => self.push(IfaValue::null())? // Graceful failure for robustness (or could propagate error)
                }
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
                let name = bytecode.strings.get(idx).cloned().ok_or(IfaError::Custom("Invalid global name index".into()))?;
                let value = self.globals.get(&name).cloned().unwrap_or(IfaValue::null());
                self.push(value)?;
            }

            OpCode::StoreGlobal => {
                let idx = self.read_u16(bytecode)? as usize;
                let name = bytecode.strings.get(idx).cloned().ok_or(IfaError::Custom("Invalid global name index".into()))?;
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
                    // Native closure call
                    // Note: IfaValue::Fn carries the function data directly
                    
                    IfaValue::Fn(data) => {
                        // VM Bytecode call
                        if args.len() != data.arity as usize {
                            return Err(IfaError::ArityMismatch {
                                expected: data.arity as usize,
                                got: args.len(),
                            });
                        }

                        // Push CallFrame
                        self.frames.push(CallFrame {
                            return_addr: self.ip,
                            base_ptr: self.stack.len(), // Base of the new frame
                            local_count: 0, 
                        });

                        // Push arguments back onto stack (as locals 0..N)
                        for arg in args {
                            self.push(arg)?;
                        }

                        // Jump
                        self.ip = data.start_ip;
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Function".into(),
                            got: func.type_name().into(),
                        });
                    }
                }
            }

            OpCode::Return => {
                if let Some(frame) = self.frames.pop() {
                    let return_value = self.pop().unwrap_or(IfaValue::null());
                    // Unwind stack to base_ptr
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
                let len = self.read_u8(bytecode)? as usize;
                let mut method_bytes = Vec::with_capacity(len);
                for _ in 0..len {
                    method_bytes.push(self.read_u8(bytecode)?);
                }
                let method_name =
                    String::from_utf8(method_bytes).map_err(|_| IfaError::Custom("Invalid UTF-8 in CallOdu method name".into()))?;

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
                    return Err(IfaError::Custom(
                        "No standard library registry attached".into(),
                    ));
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
                    return Err(IfaError::Custom(
                        "No standard library registry attached".into(),
                    ));
                }
            }

            // Collections
            OpCode::GetIndex => {


                // Indexing
                let index = self.pop()?;
                let collection = self.pop()?;
                
                match collection {
                    IfaValue::Map(m) => {
                         let key = match index {
                             IfaValue::Str(s) => s,
                             _ => return Err(IfaError::TypeError { expected: "Str".into(), got: index.type_name().into() })
                         };
                         match m.get(&key) {
                             Some(v) => self.push(v.clone())?,
                             None => self.push(IfaValue::null())?
                         }
                    }
                    IfaValue::List(l) => {
                         let idx = match index {
                             IfaValue::Int(i) => i as usize,
                             _ => return Err(IfaError::TypeError { expected: "Int".into(), got: index.type_name().into() })
                         };
                         if idx >= l.len() {
                             return Err(IfaError::Runtime("Index out of bounds".into()));
                         }
                         self.push(l[idx].clone())?
                    }
                    IfaValue::Str(s) => {
                         let idx = match index {
                             IfaValue::Int(i) => i as usize,
                             _ => return Err(IfaError::TypeError { expected: "Int".into(), got: index.type_name().into() })
                         };
                         // Very inefficient char access, but functional
                        if let Some(c) = s.chars().nth(idx) {
                            self.push(IfaValue::str(c.to_string()))?;
                        } else {
                             return Err(IfaError::Runtime("Index out of bounds".into()));
                        }
                    }
                    _ => return Err(IfaError::TypeError { expected: "Collection".into(), got: collection.type_name().into() })
                }
            }

            OpCode::SetIndex => {
                let val = self.pop()?;
                let index = self.pop()?;
                let mut collection = self.pop()?;
                
                match collection {
                    IfaValue::List(ref mut vec_arc) => {
                        let i = match index {
                            IfaValue::Int(n) => n as usize,
                            _ => return Err(IfaError::TypeError { expected: "Int".into(), got: index.type_name().into() })
                        };
                         // HIGH PERFORMANCE: CoW using make_mut
                         let vec = std::sync::Arc::make_mut(vec_arc);
                         if i >= vec.len() { return Err(IfaError::Runtime("Index out of bounds".into())); }
                         vec[i] = val;
                    }
                    IfaValue::Map(ref mut map_arc) => {
                        let k = match index {
                            IfaValue::Str(s) => s.clone(),
                            _ => return Err(IfaError::TypeError { expected: "Str".into(), got: index.type_name().into() })
                        };
                        let map = std::sync::Arc::make_mut(map_arc);
                        map.insert(k, val);
                    }
                    _ => return Err(IfaError::TypeError { expected: "List/Map".into(), got: collection.type_name().into() })
                }
            }



            OpCode::Append => {
                let val = self.pop()?;
                let mut list = self.pop()?;
                if let IfaValue::List(ref mut vec_arc) = list {
                     let vec = std::sync::Arc::make_mut(vec_arc);
                     vec.push(val);
                } else {
                     return Err(IfaError::TypeError { expected: "List".into(), got: list.type_name().into() });
                }
                self.push(list)?;
            }

            OpCode::BuildList => {
                let count = self.read_u8(bytecode)? as usize;
                let mut items = Vec::with_capacity(count);
                for _ in 0..count {
                    items.push(self.pop()?);
                }
                items.reverse();
                self.push(IfaValue::list(items))?;
            }

            OpCode::BuildMap => {
                let count = self.read_u8(bytecode)? as usize;
                // IfaValue::map expects HashMap<String, IfaValue> for input constructor convenience
                let mut map = std::collections::HashMap::with_capacity(count);
                for _ in 0..count {
                    let value = self.pop()?;
                    let key = self.pop()?;
                    if let IfaValue::Str(k) = key {
                        map.insert(k.to_string(), value);
                    }
                }
                self.push(IfaValue::map(map))?;
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
                let result = IfaValue::str(input.trim());
                self.opon.record("Ogbè", "gbà (received)", &result);
                self.push(result)?;
            }

            OpCode::Import => {
                let path_idx = self.read_u16(bytecode)? as usize;
                let path = bytecode.strings.get(path_idx).cloned().ok_or(IfaError::Custom("Invalid import path index".into()))?;

                if let Some(registry) = &self.registry {
                    let module = registry.import(&path)?;
                    self.push(module)?;
                } else {
                    return Err(IfaError::Custom(
                        "No standard library registry attached".into(),
                    ));
                }
            }

            OpCode::DefineClass => {
                let name_idx = self.read_u16(bytecode)? as usize;
                let name = bytecode.strings.get(name_idx).cloned().ok_or(IfaError::Custom("Invalid class name index".into()))?;

                let field_count = self.read_u8(bytecode)? as usize;
                let mut fields = Vec::with_capacity(field_count);
                for _ in 0..field_count {
                    let f_idx = self.read_u16(bytecode)? as usize;
                    let f_name = bytecode.strings.get(f_idx).cloned().ok_or(IfaError::Custom("Invalid field name index".into()))?;
                    fields.push(f_name);
                }

                let method_count = self.read_u8(bytecode)? as usize;
                let mut methods = std::collections::HashMap::new();

                for _ in 0..method_count {
                    let func = self.pop()?;
                    // Borrow data to avoid partial move of func which is used later
                    let method_name = match func {
                        IfaValue::Fn(ref data) => Some(data.name.clone()),

                        IfaValue::Class(ref name) => Some(name.to_string()), 
                        _ => None,
                    };

                    if let Some(n) = method_name {
                        methods.insert(n, func);
                    }
                }

                self.push(IfaValue::Class(std::sync::Arc::new(name)))?;
            }
            
            // Exception Handling
            OpCode::TryBegin => {
                let offset = self.read_u32(bytecode)? as usize;
                let catch_ip = self.ip + offset;
                
                self.recovery_stack.push(RecoveryFrame {
                    stack_depth: self.stack.len(),
                    call_depth: self.frames.len(),
                    catch_ip,
                });
            }
            
            OpCode::TryEnd => {
                // Happy path: pop the unused recovery frame
                self.recovery_stack.pop();
            }
            
            OpCode::Throw => {
                let err_val = self.pop()?;
                // We return a UserError, which the main loop will catch and pass to attempt_recovery
                return Err(IfaError::UserError(err_val.to_string()));
            }

            // System
            OpCode::Halt => {
                self.halted = true;
            }

            OpCode::Ref => {
                let addr = self.read_u32(bytecode)? as usize;
                self.push(IfaValue::Int(addr as i64))?; 
            }

            OpCode::Load8 => {
                let ptr = self.pop()?;
                match ptr {
                    IfaValue::Int(addr) => {
                        let addr = addr as usize;
                        // Mock MMIO behavior validation for simulation or normal Opon read
                        let val = if addr >= 0x4000_0000 {
                            // Simulation: Reads from "hardware" return 0
                            IfaValue::int(0)
                        } else {
                            // Regular Opon access
                            self.opon.get(addr).cloned().unwrap_or(IfaValue::null())
                        };
                        self.push(val)?;
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Pointer".into(),
                            got: ptr.type_name().into(),
                        });
                    }
                }
            }

            // === Pointers & Memory ===
            OpCode::Store8 => {
                let ptr = self.pop()?;
                let val = self.pop()?;

                match ptr {
                    IfaValue::Int(addr_i) => {
                        let addr = addr_i as usize;
                        if addr >= 0x4000_0000 {
                            // Simulation: Log the "hardware" write
                            self.opon.record("MMIO", "write", &val);
                            println!("(SIMULATION) MMIO Write8 -> *0x{:X} = {}", addr, val);
                        } else {
                            let _ = self
                                .opon
                                .try_set(addr, val)
                                .map_err(|e| IfaError::Runtime(e.to_string()))?;
                        }
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Pointer (Int)".into(),
                            got: ptr.type_name().into(),
                        });
                    }
                }
            }

            OpCode::Store16 => {
                let ptr = self.pop()?;
                let val = self.pop()?;
                match ptr {
                    IfaValue::Int(addr_i) => {
                        let addr = addr_i as usize;
                        if addr >= 0x4000_0000 {
                            println!("(SIMULATION) MMIO Write16 -> *0x{:X} = {}", addr, val);
                        } else {
                             let _ = self
                                .opon
                                .try_set(addr, val)
                                .map_err(|e| IfaError::Runtime(e.to_string()))?;
                        }
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Pointer (Int)".into(),
                            got: ptr.type_name().into(),
                        });
                    }
                }
            }
            OpCode::Load16 => {
                let ptr = self.pop()?;
                match ptr {
                    IfaValue::Int(addr_i) => {
                         let addr = addr_i as usize;
                        let val = if addr >= 0x4000_0000 {
                            IfaValue::int(0)
                        } else {
                            self.opon.get(addr).cloned().unwrap_or(IfaValue::null())
                        };
                        self.push(val)?;
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Pointer (Int)".into(),
                            got: ptr.type_name().into(),
                        });
                    }
                }
            }

            OpCode::Load32 => {
                let ptr = self.pop()?;
                match ptr {
                    IfaValue::Int(addr_i) => {
                         let addr = addr_i as usize;
                        let val = if addr >= 0x4000_0000 {
                            IfaValue::int(0)
                        } else {
                            self.opon.get(addr).cloned().unwrap_or(IfaValue::null())
                        };
                        self.push(val)?;
                    }
                    _ => return Err(IfaError::TypeError { expected: "Ptr (Int)".into(), got: ptr.type_name().into() })
                }
            }
            OpCode::Load64 => {
                let ptr = self.pop()?;
                match ptr {
                    IfaValue::Int(addr_i) => {
                         let addr = addr_i as usize;
                        let val = if addr >= 0x4000_0000 {
                            IfaValue::int(0)
                        } else {
                            self.opon.get(addr).cloned().unwrap_or(IfaValue::null())
                        };
                        self.push(val)?;
                    }
                    _ => return Err(IfaError::TypeError { expected: "Ptr (Int)".into(), got: ptr.type_name().into() })
                }
            }

            OpCode::Store32 => {
                let ptr = self.pop()?;
                let val = self.pop()?;
                match ptr {
                    IfaValue::Int(addr_i) => {
                         let addr = addr_i as usize;
                        if addr >= 0x4000_0000 {
                            println!("(SIMULATION) MMIO Write32 -> *0x{:X} = {}", addr, val);
                        } else {
                            let _ = self.opon.try_set(addr, val).map_err(|e| IfaError::Runtime(e.to_string()))?;
                        }
                    }
                    _ => return Err(IfaError::TypeError { expected: "Ptr (Int)".into(), got: ptr.type_name().into() })
                }
            }
            OpCode::Store64 => {
                let ptr = self.pop()?;
                let val = self.pop()?;
                match ptr {
                    IfaValue::Int(addr_i) => {
                        let addr = addr_i as usize;
                        if addr >= 0x4000_0000 {
                            println!("(SIMULATION) MMIO Write64 -> *0x{:X} = {}", addr, val);
                        } else {
                            let _ = self
                                .opon
                                .try_set(addr, val)
                                .map_err(|e| IfaError::Runtime(e.to_string()))?;
                        }
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Pointer".into(),
                            got: ptr.type_name().into(),
                        });
                    }
                }
            }
            // Bitwise operations
            OpCode::And => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (IfaValue::Int(i1), IfaValue::Int(i2)) => self.push(IfaValue::int(i1 & i2))?,
                    (IfaValue::Bool(b1), IfaValue::Bool(b2)) => self.push(IfaValue::bool(b1 && b2))?,
                    (a, _) => return Err(IfaError::TypeError { expected: "Int or Bool".into(), got: a.type_name().into() })
                }
            }

            OpCode::Or => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (IfaValue::Int(i1), IfaValue::Int(i2)) => self.push(IfaValue::int(i1 | i2))?,
                    (IfaValue::Bool(b1), IfaValue::Bool(b2)) => self.push(IfaValue::bool(b1 || b2))?,
                    (a, _) => return Err(IfaError::TypeError { expected: "Int or Bool".into(), got: a.type_name().into() })
                }
            }
            
            OpCode::Xor => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (IfaValue::Int(i1), IfaValue::Int(i2)) => self.push(IfaValue::int(i1 ^ i2))?,
                    (IfaValue::Bool(b1), IfaValue::Bool(b2)) => self.push(IfaValue::bool(b1 ^ b2))?,
                    (a, _) => return Err(IfaError::TypeError { expected: "Int/Bool".into(), got: a.type_name().into() })
                }
            }

            OpCode::Len => {
                let val = self.pop()?;
                match val {
                    IfaValue::Str(s) => self.push(IfaValue::int(s.len() as i64))?,
                    IfaValue::List(l) => self.push(IfaValue::int(l.len() as i64))?,
                    IfaValue::Map(m) => self.push(IfaValue::int(m.len() as i64))?,
                    _ => return Err(IfaError::TypeError { expected: "Collection".into(), got: val.type_name().into() })
                }
            }

            OpCode::Not => {
                let a = self.pop()?;
                 match a {
                    IfaValue::Int(i) => self.push(IfaValue::int(!i))?,
                    IfaValue::Bool(b) => self.push(IfaValue::bool(!b))?,
                    _ => return Err(IfaError::TypeError { expected: "Int/Bool".into(), got: a.type_name().into() })
                }
            }

            OpCode::Shl => {
                let b = self.pop()?;
                let a = self.pop()?;
                 match (a, b) {
                    (IfaValue::Int(val), IfaValue::Int(shift)) => self.push(IfaValue::int(val << shift))?,
                    (a, _) => return Err(IfaError::TypeError { expected: "Int".into(), got: a.type_name().into() })
                }
            }

            OpCode::Shr => {
                let b = self.pop()?;
                let a = self.pop()?;
                 match (a, b) {
                    (IfaValue::Int(val), IfaValue::Int(shift)) => self.push(IfaValue::int(val >> shift))?,
                    (a, _) => return Err(IfaError::TypeError { expected: "Int".into(), got: a.type_name().into() })
                }
            }
            
            // Type Casting
            OpCode::ToInt => {
                let val = self.pop()?;
                match val {
                    IfaValue::Int(i) => self.push(IfaValue::int(i))?,
                    IfaValue::Float(f) => self.push(IfaValue::int(f as i64))?,
                    IfaValue::Bool(b) => self.push(IfaValue::int(if b { 1 } else { 0 }))?,
                    // Parse string as integer; treat malformed input as 0 (same as JS parseInt).
                    IfaValue::Str(s) => {
                        let n = s.trim().parse::<i64>().unwrap_or(0);
                        self.push(IfaValue::int(n))?
                    }
                    _ => self.push(IfaValue::int(0))?,
                }
            }

            OpCode::ToFloat => {
                let val = self.pop()?;
                match val {
                    IfaValue::Int(i) => self.push(IfaValue::float(i as f64))?,
                    IfaValue::Float(f) => self.push(IfaValue::float(f))?,
                    _ => self.push(IfaValue::float(0.0))?,
                }
            }

            OpCode::ToString => {
                let val = self.pop()?;
                self.push(IfaValue::str(val.to_string()))?;
            }

            OpCode::ToBool => {
                let val = self.pop()?;
                self.push(IfaValue::bool(val.is_truthy()))?;
            }

            OpCode::Neg => {
                let val = self.pop()?;
                match val {
                    IfaValue::Int(n) => self.push(IfaValue::int(-n))?,
                    IfaValue::Float(f) => self.push(IfaValue::float(-f))?,
                    _ => return Err(IfaError::Runtime("Invalid type for negation".into())),
                }
            }
            OpCode::Pow => {
                let exp = self.pop()?;
                let base = self.pop()?;
                match (base, exp) {
                    (IfaValue::Int(b), IfaValue::Int(e)) => self.push(IfaValue::int(b.pow(e as u32)))?,
                    (IfaValue::Float(b), IfaValue::Float(e)) => self.push(IfaValue::float(b.powf(e)))?,
                    _ => return Err(IfaError::Runtime("Invalid types for power".into())),
                }
            }
            OpCode::Mod => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (IfaValue::Int(a), IfaValue::Int(b)) => self.push(IfaValue::int(a % b))?,
                    _ => return Err(IfaError::Runtime("Modulus requires integers".into()))
                }
            }
             
             OpCode::Eq => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(IfaValue::bool(a == b))?;
             }
             OpCode::Ne => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(IfaValue::bool(a != b))?;
             }

            // Implementation placeholders
            OpCode::Push => return Err(IfaError::Custom("OpCode::Push not implemented".into())),
            _ => return Err(IfaError::Custom(format!("Unimplemented opcode: {:?}", opcode).into())),
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
        Ok(u16::from_le_bytes(bytes))
    }

    fn read_u32(&mut self, bytecode: &Bytecode) -> IfaResult<u32> {
        let mut bytes = [0u8; 4];
        bytes[0] = self.read_u8(bytecode)?;
        bytes[1] = self.read_u8(bytecode)?;
        bytes[2] = self.read_u8(bytecode)?;
        bytes[3] = self.read_u8(bytecode)?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_i16(&mut self, bytecode: &Bytecode) -> IfaResult<i16> {
        Ok(self.read_u16(bytecode)? as i16)
    }

    fn read_i64(&mut self, bytecode: &Bytecode) -> IfaResult<i64> {
        let mut bytes = [0u8; 8];
        for byte in &mut bytes {
            *byte = self.read_u8(bytecode)?;
        }
        Ok(i64::from_le_bytes(bytes))
    }

    fn read_f64(&mut self, bytecode: &Bytecode) -> IfaResult<f64> {
        let mut bytes = [0u8; 8];
        for byte in &mut bytes {
            *byte = self.read_u8(bytecode)?;
        }
        Ok(f64::from_le_bytes(bytes))
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
            5,
            0,
            0,
            0,
            0,
            0,
            0,
            0, // 5 as i64 LE
            OpCode::PushInt as u8,
            3,
            0,
            0,
            0,
            0,
            0,
            0,
            0, // 3 as i64 LE
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
