//! # Bytecode Compiler
//!
//! Compiles AST to bytecode for the Ifá-Lang VM.

use crate::ast::*;
use crate::bytecode::{Bytecode, OpCode};
use crate::error::IfaResult;
use crate::lexer::OduDomain;
use std::collections::HashMap;

/// Bytecode compiler - transforms AST to executable bytecode
pub struct Compiler {
    bytecode: Bytecode,
    /// Local variables: name -> stack slot
    locals: Vec<HashMap<String, usize>>,
    /// Current scope depth
    scope_depth: usize,
    /// Label counter for jumps
    _label_counter: usize,
}

impl Compiler {
    pub fn new(source_name: &str) -> Self {
        Compiler {
            bytecode: Bytecode::new(source_name),
            locals: vec![HashMap::new()],
            scope_depth: 0,
            _label_counter: 0,
        }
    }
    
    /// Compile a program to bytecode
    pub fn compile(mut self, program: &Program) -> IfaResult<Bytecode> {
        for stmt in &program.statements {
            self.compile_statement(stmt)?;
        }
        self.emit(OpCode::Halt);
        Ok(self.bytecode)
    }
    
    fn emit(&mut self, op: OpCode) {
        self.bytecode.code.push(op as u8);
    }
    
    fn emit_byte(&mut self, byte: u8) {
        self.bytecode.code.push(byte);
    }
    
    fn emit_i64(&mut self, value: i64) {
        self.bytecode.code.extend_from_slice(&value.to_le_bytes());
    }
    
    fn emit_f64(&mut self, value: f64) {
        self.bytecode.code.extend_from_slice(&value.to_le_bytes());
    }
    
    fn emit_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.emit_byte(bytes.len() as u8);
        self.bytecode.code.extend_from_slice(bytes);
    }
    
    fn current_offset(&self) -> usize {
        self.bytecode.code.len()
    }
    
    fn emit_jump(&mut self, op: OpCode) -> usize {
        self.emit(op);
        let offset = self.current_offset();
        // Placeholder for 16-bit offset
        self.emit_byte(0);
        self.emit_byte(0);
        offset
    }
    
    fn patch_jump(&mut self, offset: usize) {
        let jump = (self.current_offset() - offset - 2) as u16;
        self.bytecode.code[offset] = (jump & 0xff) as u8;
        self.bytecode.code[offset + 1] = ((jump >> 8) & 0xff) as u8;
    }
    
    fn begin_scope(&mut self) {
        self.scope_depth += 1;
        self.locals.push(HashMap::new());
    }
    
    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        self.locals.pop();
    }
    
    fn declare_local(&mut self, name: &str) -> usize {
        let slot = self.locals.iter().map(|m| m.len()).sum();
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name.to_string(), slot);
        }
        slot
    }
    
    fn resolve_local(&self, name: &str) -> Option<usize> {
        for scope in self.locals.iter().rev() {
            if let Some(&slot) = scope.get(name) {
                return Some(slot);
            }
        }
        None
    }
    
    fn compile_statement(&mut self, stmt: &Statement) -> IfaResult<()> {
        match stmt {
            Statement::VarDecl { name, value, .. } => {
                self.compile_expression(value)?;
                let slot = self.declare_local(name);
                self.emit(OpCode::StoreLocal);
                self.emit_byte(slot as u8);
            }
            
            Statement::Assignment { target, value, .. } => {
                self.compile_expression(value)?;
                match target {
                    AssignTarget::Variable(name) => {
                        if let Some(slot) = self.resolve_local(name) {
                            self.emit(OpCode::StoreLocal);
                            self.emit_byte(slot as u8);
                        } else {
                            self.emit(OpCode::StoreGlobal);
                            self.emit_string(name);
                        }
                    }
                    AssignTarget::Index { name, index } => {
                        // Push container, index, value
                        if let Some(slot) = self.resolve_local(name) {
                            self.emit(OpCode::LoadLocal);
                            self.emit_byte(slot as u8);
                        } else {
                            self.emit(OpCode::LoadGlobal);
                            self.emit_string(name);
                        }
                        self.compile_expression(index)?;
                        // Swap so stack is: value, container, index
                        // Then call SetIndex
                        self.emit(OpCode::SetIndex);
                    }
                }
            }
            
            Statement::Instruction { call, .. } => {
                self.compile_odu_call(call)?;
                self.emit(OpCode::Pop); // Discard result
            }
            
            Statement::If { condition, then_body, else_body, .. } => {
                self.compile_expression(condition)?;
                let else_jump = self.emit_jump(OpCode::JumpIfFalse);
                self.emit(OpCode::Pop); // Pop condition
                
                self.begin_scope();
                for s in then_body {
                    self.compile_statement(s)?;
                }
                self.end_scope();
                
                if let Some(else_stmts) = else_body {
                    let end_jump = self.emit_jump(OpCode::Jump);
                    self.patch_jump(else_jump);
                    self.emit(OpCode::Pop); // Pop condition
                    
                    self.begin_scope();
                    for s in else_stmts {
                        self.compile_statement(s)?;
                    }
                    self.end_scope();
                    self.patch_jump(end_jump);
                } else {
                    self.patch_jump(else_jump);
                }
            }
            
            Statement::While { condition, body, .. } => {
                let loop_start = self.current_offset();
                
                self.compile_expression(condition)?;
                let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
                self.emit(OpCode::Pop); // Pop condition
                
                self.begin_scope();
                for s in body {
                    self.compile_statement(s)?;
                }
                self.end_scope();
                
                // Jump back to start
                self.emit(OpCode::Jump);
                let back_offset = (self.current_offset() - loop_start + 2) as i16;
                self.emit_byte((-back_offset as u16 & 0xff) as u8);
                self.emit_byte(((-back_offset as u16 >> 8) & 0xff) as u8);
                
                self.patch_jump(exit_jump);
                self.emit(OpCode::Pop); // Pop condition
            }
            
            Statement::For { var: _var, iterable, body, .. } => {
                // Compile iterable
                self.compile_expression(iterable)?;
                
                // Create iterator - push index 0
                self.emit(OpCode::PushInt);
                self.emit_i64(0);
                
                let loop_start = self.current_offset();
                
                // Check if index < len
                self.emit(OpCode::Dup); // Dup index
                // Get length - this is simplified, real impl needs GetLen opcode
                
                // For now, emit a placeholder
                self.emit(OpCode::Pop); // Simplified
                
                for s in body {
                    self.compile_statement(s)?;
                }
                
                // Increment index and loop
                self.emit(OpCode::PushInt);
                self.emit_i64(1);
                self.emit(OpCode::Add);
                
                // Jump back
                let _ = loop_start; // TODO: proper for loop compilation
            }
            
            Statement::Return { value, .. } => {
                if let Some(expr) = value {
                    self.compile_expression(expr)?;
                } else {
                    self.emit(OpCode::PushNull);
                }
                self.emit(OpCode::Return);
            }
            
            Statement::Ase { .. } => {
                self.emit(OpCode::Halt);
            }
            
            Statement::Expr { expr, .. } => {
                self.compile_expression(expr)?;
                self.emit(OpCode::Pop);
            }
            
            Statement::EseDef { name, params, body, .. } => {
                // Functions are compiled as separate bytecode chunks
                // For now, store as a constant and skip
                let _ = (name, params, body);
                // TODO: Implement function compilation
            }
            
            Statement::OduDef { name, .. } => {
                // Class definitions
                let _ = name;
                // TODO: Implement class compilation
            }
            
            Statement::Import { path, .. } => {
                // Import handling
                let _ = path;
                // TODO: Implement import compilation
            }
            
            Statement::Taboo { source, target, .. } => {
                // Taboo is a compile-time directive, no bytecode emitted
                // Could be stored in metadata for later validation
                let _ = (source, target);
            }
            
            Statement::Ewo { condition, message, .. } => {
                // Compile the condition expression
                self.compile_expression(condition)?;
                // Note: Assertion is verified at bytecode interpretation time
                // For now, we just compile the condition check
                let _ = message;
            }
            
            Statement::Opon { size, .. } => {
                // Opon is a compile-time directive, no bytecode emitted
                // Memory size is configured at VM initialization
                let _ = size;
            }
        }
        Ok(())
    }
    
    fn compile_expression(&mut self, expr: &Expression) -> IfaResult<()> {
        match expr {
            Expression::Int(n) => {
                self.emit(OpCode::PushInt);
                self.emit_i64(*n);
            }
            
            Expression::Float(f) => {
                self.emit(OpCode::PushFloat);
                self.emit_f64(*f);
            }
            
            Expression::String(s) => {
                self.emit(OpCode::PushStr);
                self.emit_string(s);
            }
            
            Expression::Bool(b) => {
                self.emit(if *b { OpCode::PushTrue } else { OpCode::PushFalse });
            }
            
            Expression::Nil => {
                self.emit(OpCode::PushNull);
            }
            
            Expression::Identifier(name) => {
                if let Some(slot) = self.resolve_local(name) {
                    self.emit(OpCode::LoadLocal);
                    self.emit_byte(slot as u8);
                } else {
                    self.emit(OpCode::LoadGlobal);
                    self.emit_string(name);
                }
            }
            
            Expression::BinaryOp { left, op, right } => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;
                
                let opcode = match op {
                    BinaryOperator::Add => OpCode::Add,
                    BinaryOperator::Sub => OpCode::Sub,
                    BinaryOperator::Mul => OpCode::Mul,
                    BinaryOperator::Div => OpCode::Div,
                    BinaryOperator::Mod => OpCode::Mod,
                    BinaryOperator::Eq => OpCode::Eq,
                    BinaryOperator::NotEq => OpCode::Ne,
                    BinaryOperator::Lt => OpCode::Lt,
                    BinaryOperator::LtEq => OpCode::Le,
                    BinaryOperator::Gt => OpCode::Gt,
                    BinaryOperator::GtEq => OpCode::Ge,
                    BinaryOperator::And => OpCode::And,
                    BinaryOperator::Or => OpCode::Or,
                };
                self.emit(opcode);
            }
            
            Expression::UnaryOp { op, expr } => {
                self.compile_expression(expr)?;
                match op {
                    UnaryOperator::Neg => self.emit(OpCode::Neg),
                    UnaryOperator::Not => self.emit(OpCode::Not),
                }
            }
            
            Expression::List(items) => {
                for item in items {
                    self.compile_expression(item)?;
                }
                self.emit(OpCode::BuildList);
                self.emit_byte(items.len() as u8);
            }
            
            Expression::Map(entries) => {
                for (key, value) in entries {
                    self.compile_expression(key)?;
                    self.compile_expression(value)?;
                }
                self.emit(OpCode::BuildMap);
                self.emit_byte(entries.len() as u8);
            }
            
            Expression::Index { object, index } => {
                self.compile_expression(object)?;
                self.compile_expression(index)?;
                self.emit(OpCode::GetIndex);
            }
            
            Expression::OduCall(call) => {
                self.compile_odu_call(call)?;
            }
            
            Expression::Call { name, args } => {
                // Push function
                if let Some(slot) = self.resolve_local(name) {
                    self.emit(OpCode::LoadLocal);
                    self.emit_byte(slot as u8);
                } else {
                    self.emit(OpCode::LoadGlobal);
                    self.emit_string(name);
                }
                
                // Push arguments
                for arg in args {
                    self.compile_expression(arg)?;
                }
                
                self.emit(OpCode::Call);
                self.emit_byte(args.len() as u8);
            }
            
            Expression::MethodCall { object, method, args } => {
                self.compile_expression(object)?;
                for arg in args {
                    self.compile_expression(arg)?;
                }
                self.emit(OpCode::CallMethod);
                self.emit_string(method);
                self.emit_byte(args.len() as u8);
            }
        }
        Ok(())
    }
    
    fn compile_odu_call(&mut self, call: &OduCall) -> IfaResult<()> {
        // Push arguments
        for arg in &call.args {
            self.compile_expression(arg)?;
        }
        
        // Emit OduCall opcode with domain and method
        self.emit(OpCode::CallOdu);
        self.emit_byte(domain_to_byte(&call.domain));
        self.emit_string(&call.method);
        self.emit_byte(call.args.len() as u8);
        
        Ok(())
    }
}

/// Convert OduDomain to byte
fn domain_to_byte(domain: &OduDomain) -> u8 {
    match domain {
        OduDomain::Ogbe => 0,
        OduDomain::Oyeku => 1,
        OduDomain::Iwori => 2,
        OduDomain::Odi => 3,
        OduDomain::Irosu => 4,
        OduDomain::Owonrin => 5,
        OduDomain::Obara => 6,
        OduDomain::Okanran => 7,
        OduDomain::Ogunda => 8,
        OduDomain::Osa => 9,
        OduDomain::Ika => 10,
        OduDomain::Oturupon => 11,
        OduDomain::Otura => 12,
        OduDomain::Irete => 13,
        OduDomain::Ose => 14,
        OduDomain::Ofun => 15,
        OduDomain::Coop => 16,   // Pseudo-domain for FFI
        OduDomain::Opele => 17,  // Pseudo-domain for Divination
    }
}

/// Compile source code to bytecode
pub fn compile(source: &str) -> IfaResult<Bytecode> {
    let program = crate::parser::parse(source)?;
    let compiler = Compiler::new("<main>");
    compiler.compile(&program)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compile_simple() {
        let bytecode = compile("ayanmo x = 42;").unwrap();
        assert!(!bytecode.code.is_empty());
    }
    
    #[test]
    fn test_compile_arithmetic() {
        let bytecode = compile("ayanmo x = 1 + 2 * 3;").unwrap();
        assert!(!bytecode.code.is_empty());
    }
    
    #[test]
    fn test_compile_print() {
        let bytecode = compile(r#"Irosu.fo("Hello");"#).unwrap();
        assert!(!bytecode.code.is_empty());
    }
}
