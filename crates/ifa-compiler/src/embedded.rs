//! Embedded Codegen
//!
//! Compiles an AST directly to the embedded instruction set.

use ifa_bytecode::embedded::EmbeddedOpCode;
use ifa_types::ast::{BinaryOperator, Expression, Program, Statement};
use ifa_types::target::Target;

/// Compiles AST to EmbeddedOpCode bytes
pub struct EmbeddedCodegen {
    bytecode: Vec<u8>,
    locals: std::collections::HashMap<String, u8>,
}

// Note: We don't store the `Target` (e.g. EmbeddedTier0) here because
// target-specific constraints and gatekeeping are handled strictly upstream
// in the static analysis phase (EmbeddedTargetChecker). By the time we
// reach codegen, the AST is already validated as safe for the target.
impl EmbeddedCodegen {
    pub fn new(_target: Target) -> Self {
        Self {
            bytecode: Vec::new(),
            locals: std::collections::HashMap::new(),
        }
    }

    pub fn compile(mut self, program: &Program) -> Result<Vec<u8>, String> {
        for stmt in &program.statements {
            self.compile_statement(stmt)?;
        }
        self.emit(EmbeddedOpCode::Halt);
        Ok(self.bytecode)
    }

    fn emit(&mut self, op: EmbeddedOpCode) {
        self.bytecode.push(op as u8);
    }

    fn emit_u8(&mut self, val: u8) {
        self.bytecode.push(val);
    }

    // Note: `emit_u16` was removed because the current `EmbeddedOpCode`
    // instruction set only uses 8-bit registers or 32-bit immediate offsets.

    fn emit_i32(&mut self, val: i32) {
        self.bytecode.extend_from_slice(&val.to_le_bytes());
    }

    fn compile_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::VarDecl { name, value, .. } => {
                self.compile_expression(value)?;
                let next_idx = self.locals.len();
                if next_idx > 255 {
                    return Err("Too many local variables for embedded target".to_string());
                }
                let idx = next_idx as u8;
                self.locals.insert(name.clone(), idx);
                self.emit(EmbeddedOpCode::StoreLocal);
                self.emit_u8(idx);
            }
            Statement::Expr { expr, span: _ } => {
                self.compile_expression(expr)?;
                self.emit(EmbeddedOpCode::Pop);
            }
            Statement::IwaDef(_) => {
                // Erased at runtime
            }
            _ => {
                return Err(format!(
                    "Statement type {:?} not supported on embedded",
                    stmt
                ));
            }
        }
        Ok(())
    }

    fn compile_expression(&mut self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::Int(val) => {
                self.emit(EmbeddedOpCode::PushInt);
                self.emit_i32(*val as i32); // Note: Lossy for now
            }
            Expression::Float(_) => {
                // Not supported in this simplified impl yet
                return Err("Floats not yet supported in codegen".to_string());
            }
            Expression::Bool(true) => {
                self.emit(EmbeddedOpCode::PushTrue);
            }
            Expression::Bool(false) => {
                self.emit(EmbeddedOpCode::PushFalse);
            }
            Expression::Nil => {
                self.emit(EmbeddedOpCode::PushNull);
            }
            Expression::Identifier(name) => {
                if let Some(&idx) = self.locals.get(name) {
                    self.emit(EmbeddedOpCode::LoadLocal);
                    self.emit_u8(idx);
                } else {
                    return Err(format!("Undefined variable '{}'", name));
                }
            }
            Expression::BinaryOp { left, op, right } => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;
                match op {
                    BinaryOperator::Add => self.emit(EmbeddedOpCode::Add),
                    BinaryOperator::Sub => self.emit(EmbeddedOpCode::Sub),
                    BinaryOperator::Mul => self.emit(EmbeddedOpCode::Mul),
                    BinaryOperator::Div => self.emit(EmbeddedOpCode::Div),
                    BinaryOperator::Eq => self.emit(EmbeddedOpCode::Eq),
                    BinaryOperator::Lt => self.emit(EmbeddedOpCode::Lt),
                    BinaryOperator::Gt => self.emit(EmbeddedOpCode::Gt),
                    _ => return Err(format!("Binary op '{:?}' not supported on embedded", op)),
                }
            }
            _ => {
                return Err(format!(
                    "Expression type {:?} not supported on embedded",
                    expr
                ));
            }
        }
        Ok(())
    }
}
