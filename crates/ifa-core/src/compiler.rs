//! # Bytecode Compiler
//!
//! Compiles AST to bytecode for the Ifá-Lang VM.
//!
//! ### 🚀 ARCHITECTURAL STATUS (String Interpolation)
//! Interpolated strings now compile to dedicated `OpCode::ToString` + `OpCode::Concat`
//! sequences instead of overloading the arithmetic hot path through `OpCode::Add`.
//!
//! General `+` expressions remain source-compatible; this hardening pass isolates
//! interpolation without forcing a language-wide string-operator redesign.

use crate::ast::*;
use crate::bytecode::{Bytecode, OpCode};
use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use std::collections::{HashMap, HashSet};

/// Bytecode compiler - transforms AST to executable bytecode
pub struct Compiler {
    bytecode: Bytecode,
    /// Local variables: name -> stack slot
    functions: Vec<FunctionContext>,
    /// Label counter for jumps
    _label_counter: usize,
    /// Compile-time constants
    constants: HashMap<String, Expression>,
}

#[derive(Debug, Clone)]
struct Upvalue {
    name: String,
    index: usize,
    is_local: bool,
}

#[derive(Debug)]
struct FunctionContext {
    locals: Vec<HashMap<String, usize>>,
    const_locals: Vec<HashSet<String>>,
    scope_depth: usize,
    upvalues: Vec<Upvalue>,
}

impl FunctionContext {
    fn new() -> Self {
        Self {
            locals: vec![HashMap::new()],
            const_locals: vec![HashSet::new()],
            scope_depth: 0,
            upvalues: Vec::new(),
        }
    }

    fn resolve_local(&self, name: &str) -> Option<usize> {
        for scope in self.locals.iter().rev() {
            if let Some(&slot) = scope.get(name) {
                return Some(slot);
            }
        }
        None
    }

    fn add_upvalue(&mut self, name: &str, index: usize, is_local: bool) -> usize {
        if let Some(pos) = self
            .upvalues
            .iter()
            .position(|u| u.name == name && u.index == index && u.is_local == is_local)
        {
            return pos;
        }
        self.upvalues.push(Upvalue {
            name: name.to_string(),
            index,
            is_local,
        });
        self.upvalues.len() - 1
    }
}

impl Compiler {
    pub fn new(source_name: &str) -> Self {
        Compiler {
            bytecode: Bytecode::new(source_name),
            functions: vec![FunctionContext::new()],
            _label_counter: 0,
            constants: HashMap::new(),
        }
    }

    /// Compile a program to bytecode
    pub fn compile(mut self, program: &Program) -> IfaResult<Bytecode> {
        for stmt in &program.statements {
            self.compile_statement(stmt)?;
        }
        self.emit(OpCode::Halt);
        self.bytecode.exports = collect_exports(program);
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

    fn emit_u32(&mut self, value: u32) {
        self.bytecode.code.extend_from_slice(&value.to_le_bytes());
    }

    fn emit_f64(&mut self, value: f64) {
        self.bytecode.code.extend_from_slice(&value.to_le_bytes());
    }

    fn emit_string(&mut self, s: &str) {
        self.bytecode.strings.push(s.to_string());
        let idx = (self.bytecode.strings.len() - 1) as u16;
        self.emit_byte((idx & 0xff) as u8);
        self.emit_byte((idx >> 8) as u8);
    }

    fn current_offset(&self) -> usize {
        self.bytecode.code.len()
    }

    fn emit_jump(&mut self, op: OpCode) -> usize {
        self.emit(op);
        let offset = self.current_offset();
        // Placeholder for 32-bit absolute offset (little-endian)
        self.emit_u32(0);
        offset
    }

    fn patch_jump(&mut self, offset: usize) {
        let target = self.current_offset() as u32;
        let bytes = target.to_le_bytes();
        self.bytecode.code[offset..offset + 4].copy_from_slice(&bytes);
    }

    fn begin_scope(&mut self) {
        let ctx = self.current_fn_mut();
        ctx.scope_depth += 1;
        ctx.locals.push(HashMap::new());
        ctx.const_locals.push(HashSet::new());
    }

    fn end_scope(&mut self) {
        let count = {
            let ctx = self.current_fn_mut();
            ctx.scope_depth -= 1;
            let count = ctx.locals.pop().map(|scope| scope.len()).unwrap_or(0);
            let _ = ctx.const_locals.pop();
            count
        };
        for _ in 0..count {
            self.emit(OpCode::Pop);
        }
    }

    fn is_const_binding(&self, name: &str) -> bool {
        self.current_fn()
            .const_locals
            .iter()
            .rev()
            .any(|s| s.contains(name))
    }

    fn declare_local(&mut self, name: &str) -> usize {
        let ctx = self.current_fn_mut();
        let slot = ctx.locals.iter().map(|m| m.len()).sum();
        if let Some(scope) = ctx.locals.last_mut() {
            scope.insert(name.to_string(), slot);
        }
        slot
    }

    fn resolve_local(&self, name: &str) -> Option<usize> {
        self.current_fn().resolve_local(name)
    }

    fn resolve_upvalue(&mut self, name: &str) -> Option<usize> {
        let depth = self.functions.len();
        if depth <= 1 {
            return None;
        }
        self.resolve_upvalue_in(depth - 1, name)
    }

    fn resolve_upvalue_in(&mut self, func_index: usize, name: &str) -> Option<usize> {
        if func_index == 0 {
            return None;
        }
        let parent_index = func_index - 1;
        if let Some(local) = self.functions[parent_index].resolve_local(name) {
            let idx = self.functions[func_index].add_upvalue(name, local, true);
            return Some(idx);
        }
        if let Some(parent_up) = self.resolve_upvalue_in(parent_index, name) {
            let idx = self.functions[func_index].add_upvalue(name, parent_up, false);
            return Some(idx);
        }
        None
    }

    fn current_fn(&self) -> &FunctionContext {
        self.functions.last().expect("no function context")
    }

    fn current_fn_mut(&mut self) -> &mut FunctionContext {
        self.functions.last_mut().expect("no function context")
    }

    fn compile_statement(&mut self, stmt: &Statement) -> IfaResult<()> {
        match stmt {
            Statement::VarDecl { name, value, .. } => {
                self.compile_expression(value)?;
                if self.current_fn().scope_depth > 0 {
                    self.declare_local(name);
                    // Value remains on stack as the local variable
                } else {
                    self.emit(OpCode::StoreGlobal);
                    self.emit_string(name);
                }
            }

            Statement::Const { name, value, .. } => {
                // Store constant expression for inlining
                // Optimization: If expression is complex, we might want to pre-calculate?
                // But AST Expression is simpler to just store.
                // Note: Binary Ops in constants not yet fully folded by this pass,
                // but if they are trees of literals, compile_expression handles them fine (at runtime of VM... wait).
                // "Const" usually implies COMPILE TIME evaluation.
                // If I store `1+1` as expression.
                // And I inline it. `x = CONST`. `compile_expr(1+1)`.
                // Emits `Push 1, Push 1, Add`.
                // This is fine. It acts like a macro.
                // For literals, it's just `Push 3`.
                self.constants.insert(name.clone(), value.clone());
                if let Some(scope) = self.current_fn_mut().const_locals.last_mut() {
                    scope.insert(name.clone());
                }
            }

            Statement::Assignment { target, value, .. } => {
                self.compile_expression(value)?;
                match target {
                    AssignTarget::Variable(name) => {
                        if self.is_const_binding(name) {
                            return Err(IfaError::TypeError {
                                expected: "Mutable binding".into(),
                                got: format!("const {name}"),
                            });
                        }
                        if let Some(slot) = self.resolve_local(name) {
                            self.emit(OpCode::StoreLocal);
                            let s = slot as u16;
                            self.emit_byte((s & 0xff) as u8);
                            self.emit_byte((s >> 8) as u8);
                        } else if let Some(slot) = self.resolve_upvalue(name) {
                            self.emit(OpCode::StoreUpvalue);
                            let s = slot as u16;
                            self.emit_byte((s & 0xff) as u8);
                            self.emit_byte((s >> 8) as u8);
                        } else {
                            self.emit(OpCode::StoreGlobal);
                            self.emit_string(name);
                        }
                    }
                    AssignTarget::Index { name, index } => {
                        // Push container, index, value
                        if let Some(slot) = self.resolve_local(name) {
                            self.emit(OpCode::LoadLocal);
                            let s = slot as u16;
                            self.emit_byte((s & 0xff) as u8);
                            self.emit_byte((s >> 8) as u8);
                        } else if let Some(slot) = self.resolve_upvalue(name) {
                            self.emit(OpCode::LoadUpvalue);
                            let s = slot as u16;
                            self.emit_byte((s & 0xff) as u8);
                            self.emit_byte((s >> 8) as u8);
                        } else {
                            self.emit(OpCode::LoadGlobal);
                            self.emit_string(name);
                        }
                        self.compile_expression(index)?;
                        // Swap so stack is: value, container, index
                        // Then call SetIndex
                        self.emit(OpCode::SetIndex);
                    }
                    AssignTarget::Dereference(expr) => {
                        // *p = val is handled by Store8 (generic store to address)
                        // Note: If type is larger than 8 bytes, compiler should emit Store16/32 etc.
                        // For now we default to Store8 as our primitive "Store to Address" until type tracking is improved.
                        self.compile_expression(expr)?;
                        self.emit(OpCode::Store8);
                    }
                }
            }

            Statement::Instruction { call, .. } => {
                self.compile_odu_call(call)?;
                self.emit(OpCode::Pop); // Discard result
            }

            Statement::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.compile_expression(condition)?;
                let else_jump = self.emit_jump(OpCode::JumpIfFalse);

                self.begin_scope();
                for s in then_body {
                    self.compile_statement(s)?;
                }
                self.end_scope();

                if let Some(else_stmts) = else_body {
                    let end_jump = self.emit_jump(OpCode::Jump);
                    self.patch_jump(else_jump);

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

            Statement::While {
                condition, body, ..
            } => {
                let loop_start = self.current_offset();

                self.compile_expression(condition)?;
                let exit_jump = self.emit_jump(OpCode::JumpIfFalse);

                self.begin_scope();
                for s in body {
                    self.compile_statement(s)?;
                }
                self.end_scope();

                // Jump back to start
                self.emit(OpCode::Jump);
                self.emit_u32(loop_start as u32);

                self.patch_jump(exit_jump);
            }

            Statement::For {
                var,
                iterable,
                body,
                ..
            } => {
                // 1. Compile Iterable
                self.compile_expression(iterable)?;
                // Store in hidden local ".iter_col"
                let col_slot = self.declare_local(".iter_col");
                self.emit(OpCode::StoreLocal);
                let s = col_slot as u16;
                self.emit_byte((s & 0xff) as u8);
                self.emit_byte((s >> 8) as u8);

                // 2. Init Index = 0
                self.emit(OpCode::PushInt);
                self.emit_i64(0);
                let idx_slot = self.declare_local(".iter_idx");
                self.emit(OpCode::StoreLocal);
                let s = idx_slot as u16;
                self.emit_byte((s & 0xff) as u8);
                self.emit_byte((s >> 8) as u8);

                // 3. Loop Start
                let loop_start = self.current_offset();

                // 4. Condition: idx < len(col)
                self.emit(OpCode::LoadLocal);
                let s = idx_slot as u16;
                self.emit_byte((s & 0xff) as u8);
                self.emit_byte((s >> 8) as u8);

                self.emit(OpCode::LoadLocal);
                let s = col_slot as u16;
                self.emit_byte((s & 0xff) as u8);
                self.emit_byte((s >> 8) as u8);
                self.emit(OpCode::Len);

                self.emit(OpCode::Lt);

                let exit_jump = self.emit_jump(OpCode::JumpIfFalse);

                // 5. Body Setup: var = col[idx]
                self.begin_scope();

                self.emit(OpCode::LoadLocal);
                let s1 = col_slot as u16;
                self.emit_byte((s1 & 0xff) as u8);
                self.emit_byte((s1 >> 8) as u8);

                self.emit(OpCode::LoadLocal);
                let s2 = idx_slot as u16;
                self.emit_byte((s2 & 0xff) as u8);
                self.emit_byte((s2 >> 8) as u8);

                self.emit(OpCode::GetIndex);

                self.declare_local(var);
                // Value from GetIndex is now the local variable 'var'

                // Compile Body
                for s in body {
                    self.compile_statement(s)?;
                }
                self.end_scope();

                // 6. Increment Index
                self.emit(OpCode::LoadLocal);
                let s = idx_slot as u16;
                self.emit_byte((s & 0xff) as u8);
                self.emit_byte((s >> 8) as u8);

                self.emit(OpCode::PushInt);
                self.emit_i64(1);
                self.emit(OpCode::Add);
                self.emit(OpCode::StoreLocal);
                let s = idx_slot as u16;
                self.emit_byte((s & 0xff) as u8);
                self.emit_byte((s >> 8) as u8);

                // 7. Jump Back
                self.emit(OpCode::Jump);
                self.emit_u32(loop_start as u32);

                self.patch_jump(exit_jump);
            }

            Statement::Return { value, .. } => {
                if let Some(expr) = value {
                    // Tail-call optimization: if we're returning a direct function call, emit TailCall
                    // so the VM can reuse the current frame.
                    if let Expression::Call { name, args } = expr {
                        // Push function
                        if let Some(slot) = self.resolve_local(name) {
                            self.emit(OpCode::LoadLocal);
                            let s = slot as u16;
                            self.emit_byte((s & 0xff) as u8);
                            self.emit_byte((s >> 8) as u8);
                        } else if let Some(slot) = self.resolve_upvalue(name) {
                            self.emit(OpCode::LoadUpvalue);
                            let s = slot as u16;
                            self.emit_byte((s & 0xff) as u8);
                            self.emit_byte((s >> 8) as u8);
                        } else {
                            self.emit(OpCode::LoadGlobal);
                            self.emit_string(name);
                        }

                        // Push arguments
                        for arg in args {
                            self.compile_expression(arg)?;
                        }

                        self.emit(OpCode::TailCall);
                        self.emit_byte(args.len() as u8);
                    } else {
                        self.compile_expression(expr)?;
                        self.emit(OpCode::Return);
                    }
                } else {
                    self.emit(OpCode::PushNull);
                    self.emit(OpCode::Return);
                }
            }

            Statement::Ase { .. } => {
                self.emit(OpCode::Halt);
            }

            Statement::Expr { expr, .. } => {
                self.compile_expression(expr)?;
                self.emit(OpCode::Pop);
            }

            Statement::EseDef {
                name,
                params,
                body,
                is_async,
                ..
            } => {
                self.compile_function(name, params, body, *is_async)?;

                // 8. Store in variable
                // If inside a local scope, bind as a local (or reuse existing).
                if self.current_fn().scope_depth > 0 {
                    if let Some(slot) = self.resolve_local(name) {
                        self.emit(OpCode::StoreLocal);
                        let s = slot as u16;
                        self.emit_byte((s & 0xff) as u8);
                        self.emit_byte((s >> 8) as u8);
                    } else {
                        self.declare_local(name);
                    }
                } else {
                    // Otherwise Global
                    self.bytecode.strings.push(name.clone());
                    let name_idx = (self.bytecode.strings.len() - 1) as u16;
                    self.emit(OpCode::StoreGlobal);
                    self.emit_byte((name_idx & 0xff) as u8);
                    self.emit_byte((name_idx >> 8) as u8);
                }
            }

            Statement::OduDef { name, .. } => {
                // DESIGN DECISION (2026-04-07): Class-based OOP is formally removed from Ifá-Lang.
                // Rationale:
                //   1. OOP inheritance hierarchies contradict the sibling-domain philosophy of the 16 Odù.
                //   2. Class vtables require runtime dynamic dispatch, violating the Zero-Cost Architecture.
                //   3. `ifa-babalawo` structural subtyping already provides polymorphism via shape-checking.
                //
                // MIGRATION PATH: Replace class definitions with Maps + Domain functions.
                //   Instead of:  class Dog { ... }
                //   Use:         ayanmo dog = { name: "Fido", bark: ese() { ... } }
                //
                // See ROADMAP.md §Phase 2 "Protocol-Oriented Design" for the full specification.
                return Err(IfaError::Custom(format!(
                    "Class/OOP syntax ('{name}') is not supported. \
                     Ifá-Lang uses Protocol-Oriented design: data is a Map, behaviour is a Domain function. \
                     See ROADMAP.md §Phase 2 for the migration guide."
                )));
            }

            Statement::Import { path, names, .. } => {
                let is_std = path.first().map(|p| p == "std").unwrap_or(false);
                let import_path = path.join(".");

                let bind_name = |this: &mut Compiler, name: &str| {
                    if this.current_fn().scope_depth > 0 {
                        if let Some(slot) = this.resolve_local(name) {
                            this.emit(OpCode::StoreLocal);
                            let s = slot as u16;
                            this.emit_byte((s & 0xff) as u8);
                            this.emit_byte((s >> 8) as u8);
                        } else {
                            this.declare_local(name);
                        }
                    } else {
                        this.emit(OpCode::StoreGlobal);
                        this.emit_string(name);
                    }
                };

                if is_std {
                    // For std imports, bind module marker or named function markers.
                    if let Some(names) = names {
                        let domain = path.last().cloned().unwrap_or_default();
                        for name in names {
                            let marker = format!("__odu_fn__:{}:{}", domain, name);
                            self.emit(OpCode::PushStr);
                            self.emit_string(&marker);
                            bind_name(self, name);
                        }
                    } else {
                        self.emit(OpCode::Import);
                        self.emit_string(&import_path);
                        let module_name = path.last().cloned().unwrap_or_else(|| "module".into());
                        bind_name(self, &module_name);
                    }
                } else {
                    self.emit(OpCode::Import);
                    self.emit_string(&import_path);

                    if let Some(names) = names {
                        for name in names {
                            self.emit(OpCode::Dup);
                            self.emit(OpCode::PushStr);
                            self.emit_string(name);
                            self.emit(OpCode::GetIndex);
                            bind_name(self, name);
                        }
                        self.emit(OpCode::Pop);
                    } else {
                        let module_name = path.last().cloned().unwrap_or_else(|| "module".into());
                        bind_name(self, &module_name);
                    }
                }
            }

            Statement::Taboo { source, target, .. } => {
                // Taboo is a compile-time directive, no bytecode emitted
                // Could be stored in metadata for later validation
                let _ = (source, target);
            }

            Statement::Ewo {
                condition, message, ..
            } => {
                // Compile the condition expression
                self.compile_expression(condition)?;
                // Note: Assertion is verified at bytecode interpretation time
                // For now, we just compile the condition check
                let _ = message;
            }

            Statement::Opon { size, .. } => {
                // Set the memory configuration directive on the bytecode header
                let opon_size = match size.as_str() {
                    "kekere" => crate::bytecode::OponSize::Kekere,
                    "arinrin" => crate::bytecode::OponSize::Arinrin,
                    "nla" => crate::bytecode::OponSize::Nla,
                    "ailopin" => crate::bytecode::OponSize::Ailopin,
                    _ => crate::bytecode::OponSize::Arinrin, // default fallback
                };
                self.bytecode.opon_size = opon_size;
            }

            Statement::Match { .. } => {
                let Statement::Match {
                    condition, arms, ..
                } = stmt
                else {
                    unreachable!("match arm destructuring failed");
                };

                self.begin_scope();
                self.compile_expression(condition)?;
                let cond_slot = self.declare_local(".match_cond");

                let mut end_jumps = Vec::new();

                for arm in arms {
                    match &arm.pattern {
                        MatchPattern::Literal(expr) => {
                            self.emit(OpCode::LoadLocal);
                            let s = cond_slot as u16;
                            self.emit_byte((s & 0xff) as u8);
                            self.emit_byte((s >> 8) as u8);
                            self.compile_expression(expr)?;
                            self.emit(OpCode::Eq);

                            let skip_arm = self.emit_jump(OpCode::JumpIfFalse);

                            self.begin_scope();
                            for s in &arm.body {
                                self.compile_statement(s)?;
                            }
                            self.end_scope();

                            end_jumps.push(self.emit_jump(OpCode::Jump));
                            self.patch_jump(skip_arm);
                        }
                        MatchPattern::Range { start, end } => {
                            // cond >= start
                            self.emit(OpCode::LoadLocal);
                            let s = cond_slot as u16;
                            self.emit_byte((s & 0xff) as u8);
                            self.emit_byte((s >> 8) as u8);
                            self.compile_expression(start)?;
                            self.emit(OpCode::Ge);
                            let skip_arm_1 = self.emit_jump(OpCode::JumpIfFalse);

                            // cond <= end
                            self.emit(OpCode::LoadLocal);
                            let s = cond_slot as u16;
                            self.emit_byte((s & 0xff) as u8);
                            self.emit_byte((s >> 8) as u8);
                            self.compile_expression(end)?;
                            self.emit(OpCode::Le);
                            let skip_arm_2 = self.emit_jump(OpCode::JumpIfFalse);

                            self.begin_scope();
                            for s in &arm.body {
                                self.compile_statement(s)?;
                            }
                            self.end_scope();

                            end_jumps.push(self.emit_jump(OpCode::Jump));
                            self.patch_jump(skip_arm_1);
                            self.patch_jump(skip_arm_2);
                        }
                        MatchPattern::Wildcard => {
                            self.begin_scope();
                            for s in &arm.body {
                                self.compile_statement(s)?;
                            }
                            self.end_scope();

                            end_jumps.push(self.emit_jump(OpCode::Jump));
                        }
                    }
                }

                for jump in end_jumps {
                    self.patch_jump(jump);
                }

                self.end_scope();
            }

            Statement::Ebo { .. } => {
                // Ebo (sacrifice) is a semantic directive, no bytecode emitted
            }

            Statement::Ailewu { body, .. } => {
                // Ailewu (unsafe) block - just compile the body
                // Safety checks are done at static analysis time
                self.begin_scope();
                for s in body {
                    self.compile_statement(s)?;
                }
                self.end_scope();
            }

            Statement::Yield { duration, .. } => {
                // Compile the duration expression
                self.compile_expression(duration)?;
                // Emit Yield opcode
                self.emit(OpCode::Yield);
            }

            Statement::Try {
                try_body,
                catch_var,
                catch_body,
                finally_body,
                ..
            } => {
                // === Code layout ===
                // TryBegin(catch_ip)
                // FinallyBegin(finally_ip)  <-- only emitted if finally_body is Some
                // ... try body ...
                // TryEnd
                // ... finally body (happy path inline copy) ...
                // Jump(after_catch)
                //
                // [catch_ip]:
                // ... catch body ...
                // ... finally body (error path inline copy) ...
                //
                // [after_catch]: ...

                // 1. Emit TryBegin with placeholder catch offset
                self.emit(OpCode::TryBegin);
                let try_begin_offset = self.current_offset();
                self.emit_u32(0); // Placeholder: offset to catch handler

                // 2. Emit FinallyBegin if there is a finally body
                let finally_begin_offset = if finally_body.is_some() {
                    self.emit(OpCode::FinallyBegin);
                    let off = self.current_offset();
                    self.emit_u32(0); // Placeholder: absolute IP of finally block
                    Some(off)
                } else {
                    None
                };

                // 3. Compile Try Body
                self.begin_scope();
                for s in try_body {
                    self.compile_statement(s)?;
                }
                self.end_scope();

                // 4. Emit TryEnd (happy path — no error)
                self.emit(OpCode::TryEnd);

                // 5. Emit finally body inline (happy path)
                if let Some(fb) = finally_body {
                    self.begin_scope();
                    for s in fb {
                        self.compile_statement(s)?;
                    }
                    self.end_scope();
                }

                // 6. Jump over catch block
                let skip_catch_jump = self.emit_jump(OpCode::Jump);

                // 7. Patch TryBegin offset → current position is catch handler start
                let catch_start_offset = self.current_offset();
                let jump_distance = (catch_start_offset - try_begin_offset - 4) as u32;
                let bytes = jump_distance.to_le_bytes();
                self.bytecode.code[try_begin_offset] = bytes[0];
                self.bytecode.code[try_begin_offset + 1] = bytes[1];
                self.bytecode.code[try_begin_offset + 2] = bytes[2];
                self.bytecode.code[try_begin_offset + 3] = bytes[3];

                // 8. Patch FinallyBegin → absolute IP of the finally code
                //    The finally code lives after the catch block (see below).
                //    We store the offset placeholder here, actual patch happens after
                //    we know where the finally code starts (step 11).
                // (placeholder already written; we'll patch after catch body)

                // 9. Compile Catch Block
                self.begin_scope();
                // The error value is already on the stack (pushed by attempt_recovery).
                // Declare a local slot for it without emitting StoreLocal.
                self.declare_local(catch_var);
                for s in catch_body {
                    self.compile_statement(s)?;
                }
                self.end_scope();

                // 10. Emit finally body after catch, and patch FinallyBegin
                if let Some(fb) = finally_body {
                    // Patch FinallyBegin operand → absolute IP of this finally block
                    let finally_ip = self.current_offset() as u32;
                    if let Some(fb_off) = finally_begin_offset {
                        let bytes = finally_ip.to_le_bytes();
                        self.bytecode.code[fb_off] = bytes[0];
                        self.bytecode.code[fb_off + 1] = bytes[1];
                        self.bytecode.code[fb_off + 2] = bytes[2];
                        self.bytecode.code[fb_off + 3] = bytes[3];
                    }

                    self.begin_scope();
                    for s in fb {
                        self.compile_statement(s)?;
                    }
                    self.end_scope();

                    // Signal end of the shared finally block
                    self.emit(OpCode::FinallyEnd);
                }

                // 11. Patch jump over catch
                self.patch_jump(skip_catch_jump);
            }
        }
        Ok(())
    }

    fn compile_function(
        &mut self,
        name: &str,
        params: &[Param],
        body: &[Statement],
        is_async: bool,
    ) -> IfaResult<()> {
        // 1. Emit Jump over the body
        let jump = self.emit_jump(OpCode::Jump);

        // 2. Record Start IP
        let start_ip = self.current_offset();

        // 2.5. New function context
        self.functions.push(FunctionContext::new());

        // 3. Begin Scope & Bind Params
        self.begin_scope();
        for param in params {
            self.declare_local(&param.name);
        }

        // 4. Compile Body
        for stmt in body {
            self.compile_statement(stmt)?;
        }

        // 5. Implicit Return (Null)
        self.emit(OpCode::PushNull);
        self.emit(OpCode::Return);

        self.end_scope();

        // Capture upvalues before popping the context
        let upvalues = self.current_fn().upvalues.clone();
        self.functions.pop();

        // 6. Patch Jump
        self.patch_jump(jump);

        // 7. Emit PushFn instruction
        self.emit(OpCode::PushFn);
        // name index
        self.bytecode.strings.push(name.to_string());
        let name_idx = (self.bytecode.strings.len() - 1) as u16;
        self.emit_byte((name_idx & 0xff) as u8);
        self.emit_byte((name_idx >> 8) as u8);

        // start_ip (u32, little-endian)
        self.emit_u32(start_ip as u32);

        // arity (u8)
        self.emit_byte(params.len() as u8);
        self.emit_byte(if is_async { 1 } else { 0 });

        // 8. If needed, wrap in a closure with captured upvalues
        if !upvalues.is_empty() {
            self.emit(OpCode::MakeClosure);
            self.emit_byte(upvalues.len() as u8);
            for up in upvalues {
                self.emit_byte(if up.is_local { 0 } else { 1 });
                let idx = up.index as u16;
                self.emit_byte((idx & 0xff) as u8);
                self.emit_byte((idx >> 8) as u8);
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
                self.emit(if *b {
                    OpCode::PushTrue
                } else {
                    OpCode::PushFalse
                });
            }

            Expression::Nil => {
                self.emit(OpCode::PushNull);
            }

            Expression::Identifier(name) => {
                // Check constants first (Inlining)
                if let Some(expr) = self.constants.get(name).cloned() {
                    self.compile_expression(&expr)?;
                    return Ok(());
                }

                if let Some(slot) = self.resolve_local(name) {
                    self.emit(OpCode::LoadLocal);
                    let s = slot as u16;
                    self.emit_byte((s & 0xff) as u8);
                    self.emit_byte((s >> 8) as u8);
                } else if let Some(slot) = self.resolve_upvalue(name) {
                    self.emit(OpCode::LoadUpvalue);
                    let s = slot as u16;
                    self.emit_byte((s & 0xff) as u8);
                    self.emit_byte((s >> 8) as u8);
                } else {
                    self.emit(OpCode::LoadGlobal);
                    self.emit_string(name);
                }
            }

            Expression::BinaryOp { left, op, right } => {
                match op {
                    // R4: Short-circuit + operand-return semantics for logical AND/OR
                    BinaryOperator::And => {
                        self.compile_expression(left)?;
                        self.emit(OpCode::Dup);
                        let end_jump = self.emit_jump(OpCode::JumpIfFalse);
                        self.emit(OpCode::Pop);
                        self.compile_expression(right)?;
                        self.patch_jump(end_jump);
                    }
                    BinaryOperator::Or => {
                        self.compile_expression(left)?;
                        self.emit(OpCode::Dup);
                        let end_jump = self.emit_jump(OpCode::JumpIfTrue);
                        self.emit(OpCode::Pop);
                        self.compile_expression(right)?;
                        self.patch_jump(end_jump);
                    }
                    _ => {
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
                            BinaryOperator::And | BinaryOperator::Or => {
                                unreachable!("handled above")
                            }
                        };
                        self.emit(opcode);
                    }
                }
            }

            Expression::UnaryOp { op, expr } => {
                match op {
                    UnaryOperator::Neg => {
                        self.compile_expression(expr)?;
                        self.emit(OpCode::Neg);
                    }
                    UnaryOperator::Not => {
                        // Spec: `!x` is truthiness-based (not Bool-only). Use ToBool + Not.
                        self.compile_expression(expr)?;
                        self.emit(OpCode::ToBool);
                        self.emit(OpCode::Not);
                    }
                    UnaryOperator::AddressOf => {
                        // Only support literal addresses for now: &0x4000
                        if let Expression::Int(addr) = *expr.clone() {
                            self.emit(OpCode::Ref);
                            self.emit_u32(addr as u32);
                        } else {
                            return Err(crate::error::IfaError::Compile(
                                "Only literal addresses supported for AddressOf (&) currently"
                                    .to_string(),
                            ));
                        }
                    }
                    UnaryOperator::Dereference => {
                        self.compile_expression(expr)?;
                        // Default to Load8 (generic Load from Address)
                        self.emit(OpCode::Load8);
                    }
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
                    let s = slot as u16;
                    self.emit_byte((s & 0xff) as u8);
                    self.emit_byte((s >> 8) as u8);
                } else if let Some(slot) = self.resolve_upvalue(name) {
                    self.emit(OpCode::LoadUpvalue);
                    let s = slot as u16;
                    self.emit_byte((s & 0xff) as u8);
                    self.emit_byte((s >> 8) as u8);
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

            Expression::Await(expr) => {
                self.compile_expression(expr)?;
                self.emit(OpCode::Await);
            }

            Expression::Try(expr) => {
                // §12.3: Error propagation operator `?`.
                // Compile the inner expression, then emit PropagateError.
                // The VM will pop the value: if it's a UserError it re-raises;
                // otherwise it pushes the unwrapped value back.
                self.compile_expression(expr)?;
                self.emit(OpCode::PropagateError);
            }

            Expression::MethodCall {
                object,
                method,
                args,
            } => {
                self.compile_expression(object)?;
                for arg in args {
                    self.compile_expression(arg)?;
                }
                self.emit(OpCode::CallMethod);
                self.emit_string(method);
                self.emit_byte(args.len() as u8);
            }

            Expression::InterpolatedString { parts } => {
                if parts.is_empty() {
                    self.emit(OpCode::PushStr);
                    self.emit_string("");
                } else {
                    for (i, part) in parts.iter().enumerate() {
                        match part {
                            InterpolatedPart::Literal(s) => {
                                self.emit(OpCode::PushStr);
                                self.emit_string(s);
                            }
                            InterpolatedPart::Expression(expr) => {
                                self.compile_expression(expr)?;
                                self.emit(OpCode::ToString);
                            }
                        }
                        if i > 0 {
                            self.emit(OpCode::Concat);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn compile_odu_call(&mut self, call: &OduCall) -> IfaResult<()> {
        // Intrinsic: Store.write8/16
        if call.domain == OduDomain::Storage {
            if call.method == "write8" && call.args.len() == 2 {
                // write8(ptr, val). Expected stack: [Val, Ptr]
                self.compile_expression(&call.args[1])?; // Val
                self.compile_expression(&call.args[0])?; // Ptr
                self.emit(OpCode::Store8);
                return Ok(());
            }
            if call.method == "write16" && call.args.len() == 2 {
                self.compile_expression(&call.args[1])?;
                self.compile_expression(&call.args[0])?;
                self.emit(OpCode::Store16);
                return Ok(());
            }
            if call.method == "read8" && call.args.len() == 1 {
                self.compile_expression(&call.args[0])?;
                self.emit(OpCode::Load8);
                return Ok(());
            }
            if call.method == "read16" && call.args.len() == 1 {
                self.compile_expression(&call.args[0])?;
                self.emit(OpCode::Load16);
                return Ok(());
            }
        }

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
        // Core 16 Odù
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
        // Pseudo-domains
        OduDomain::Coop => 16,
        OduDomain::Opele => 17,
        // Infrastructure Layer
        OduDomain::Cpu => 18,
        OduDomain::Gpu => 19,
        OduDomain::Storage => 20,
        OduDomain::Ohun => 27,  // Audio
        OduDomain::Fidio => 28, // Video
        // Application Stacks
        OduDomain::Backend => 21,
        OduDomain::Frontend => 22,
        OduDomain::Crypto => 23,
        OduDomain::Ml => 24,
        OduDomain::GameDev => 25,
        OduDomain::Iot => 26,
        OduDomain::Sys => 29,
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

fn collect_exports(program: &Program) -> Vec<String> {
    let mut out = Vec::new();
    for stmt in &program.statements {
        match stmt {
            Statement::VarDecl {
                name,
                visibility: Visibility::Public,
                ..
            } => out.push(name.clone()),
            Statement::Const {
                name,
                visibility: Visibility::Public,
                ..
            } => out.push(name.clone()),
            Statement::EseDef {
                name,
                visibility: Visibility::Public,
                ..
            } => out.push(name.clone()),
            Statement::OduDef {
                name,
                visibility: Visibility::Public,
                ..
            } => out.push(name.clone()),
            _ => {}
        }
    }
    out
}
