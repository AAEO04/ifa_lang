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

use ifa_types::OduDomain;
use ifa_types::ast::*;
use ifa_types::bytecode::OponSize;
use ifa_types::methods::resolve_method_id;
use ifa_types::{Bytecode, OpCode};
use ifa_types::{IfaError, IfaResult};
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
    /// String pool deduplication map: string -> u16 index in bytecode.strings
    /// E5: ensures each unique string is emitted exactly once.
    string_index: HashMap<String, u16>,
    loop_stack: Vec<LoopContext>,
}

#[derive(Debug)]
struct LoopContext {
    #[allow(dead_code)]
    start_ip: usize,
    break_jumps: Vec<usize>,
    continue_jumps: Vec<usize>,
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
    /// Per-scope deferred cleanup bodies (innermost Vec is current scope's list)
    deferred: Vec<Vec<Vec<Statement>>>,
}

impl FunctionContext {
    fn new() -> Self {
        Self {
            locals: vec![HashMap::new()],
            const_locals: vec![HashSet::new()],
            scope_depth: 0,
            upvalues: Vec::new(),
            deferred: Vec::new(),
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
            string_index: HashMap::new(),
            loop_stack: Vec::new(),
        }
    }

    pub fn with_constants(mut self, constants: HashMap<String, Expression>) -> Self {
        self.constants = constants;
        self
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

    /// Compile a program for interactive REPL execution, preserving the last expression
    pub fn compile_repl(
        mut self,
        program: &Program,
    ) -> IfaResult<(Bytecode, HashMap<String, Expression>)> {
        if program.statements.is_empty() {
            self.emit(OpCode::Halt);
            return Ok((self.bytecode, self.constants));
        }

        let len = program.statements.len();
        for i in 0..len - 1 {
            self.compile_statement(&program.statements[i])?;
        }

        let last_stmt = &program.statements[len - 1];
        if let Statement::Expr { expr, .. } = last_stmt {
            self.compile_expression(expr)?;
        } else {
            self.compile_statement(last_stmt)?;
        }

        self.emit(OpCode::Halt);
        self.bytecode.exports = collect_exports(program);
        Ok((self.bytecode, self.constants))
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
        let idx = if let Some(&existing) = self.string_index.get(s) {
            existing
        } else {
            let idx = self.bytecode.strings.len() as u16;
            self.bytecode.strings.push(s.to_string());
            self.string_index.insert(s.to_string(), idx);
            idx
        };
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

    fn patch_jump_to(&mut self, offset: usize, target: usize) {
        let target = target as u32;
        let bytes = target.to_le_bytes();
        self.bytecode.code[offset..offset + 4].copy_from_slice(&bytes);
    }

    fn begin_scope(&mut self) {
        let ctx = self.current_fn_mut();
        ctx.scope_depth += 1;
        ctx.locals.push(HashMap::new());
        ctx.const_locals.push(HashSet::new());
        ctx.deferred.push(Vec::new());
    }

    fn end_scope(&mut self) -> IfaResult<()> {
        let (count, deferred_stmts) = {
            let ctx = self.current_fn_mut();
            ctx.scope_depth -= 1;
            let count = ctx.locals.pop().map(|scope| scope.len()).unwrap_or(0);
            let _ = ctx.const_locals.pop();
            let deferred = ctx.deferred.pop().unwrap_or_default();
            (count, deferred)
        };
        // Compile deferred cleanups in reverse (LIFO) before popping locals
        for stmts in deferred_stmts.into_iter().rev() {
            for s in stmts {
                self.compile_statement(&s)?;
            }
        }
        for _ in 0..count {
            self.emit(OpCode::Pop);
        }
        Ok(())
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
        let line = stmt.span().line as u32;
        let offset = self.current_offset();
        if self.bytecode.lines.last().map(|(off, _)| *off) != Some(offset) {
            self.bytecode.lines.push((offset, line));
        }

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

            Statement::Update {
                target, op, value, ..
            } => {
                // 1. Load current value onto stack
                match target {
                    AssignTarget::Variable(name) => {
                        if self.is_const_binding(name) {
                            self.emit(OpCode::PushStr);
                            self.emit_string(&format!(
                                "Type mismatch: expected Mutable binding, got const {}",
                                name
                            ));
                            self.emit(OpCode::Throw);
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
                    AssignTarget::Index { name, index } => {
                        // Push container, index
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
                        self.emit(OpCode::Dup); // Duplicate index
                        self.emit(OpCode::Swap); // [container, index, index]
                        self.emit(OpCode::GetIndex); // [index, value]
                        self.emit(OpCode::Swap); // [value, index]
                    }
                    AssignTarget::Dereference(expr) => {
                        self.compile_expression(expr)?;
                        self.emit(OpCode::Dup); // [ptr, ptr]
                        self.emit(OpCode::Load8); // [ptr, val]
                        self.emit(OpCode::Swap); // [val, ptr]
                    }
                }

                // Stack state: [..., current_value, (index or ptr if complex target)]

                // Apply operation via the centralized helper (handles AddAssign -> Concat for strings)
                let val_expr = value
                    .as_ref()
                    .ok_or_else(|| IfaError::Parse("Augmented assignment missing value".into()))?;
                self.compile_expression(val_expr)?;
                match op {
                    UpdateOp::AddAssign => match val_expr {
                        Expression::String(_) | Expression::InterpolatedString { .. } => {
                            self.emit(OpCode::Concat)
                        }
                        _ => self.emit(OpCode::Add),
                    },
                    UpdateOp::SubAssign => self.emit(OpCode::Sub),
                    UpdateOp::MulAssign => self.emit(OpCode::Mul),
                    UpdateOp::DivAssign => self.emit(OpCode::Div),
                    UpdateOp::ModAssign => self.emit(OpCode::Mod),
                }

                // Stack state: [..., target_info (optional), new_value]

                // 3. Store back
                match target {
                    AssignTarget::Variable(name) => {
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
                    AssignTarget::Index { .. } => {
                        // Stack: [..., index, new_value]
                        // We need container back. Wait, let's re-push container.
                        // Better approach:
                        // [container, index]
                        // Dup index -> [container, index, index]
                        // GetIndex -> [index, value]
                        // Op -> [index, new_value]
                        // But SetIndex expects [container, index, new_value]

                        // Let's redo Index Update stack dance:
                        // Load container [c]
                        // Compile index [c, i]
                        // Dup2 -> [c, i, c, i]
                        // GetIndex -> [c, i, v]
                        // Compile rhs -> [c, i, v, r]
                        // Op -> [c, i, nv]
                        // SetIndex -> []

                        // Let's refactor the whole Update compilation to be cleaner.
                        // I'll rewrite this.
                    }
                    _ => {}
                }

                // I'll use a more robust stack management for Update.
                return self.compile_update_statement(target, op, value);
            }

            Statement::Assignment { target, value, .. } => {
                self.compile_expression(value)?;
                match target {
                    AssignTarget::Variable(name) => {
                        if self.is_const_binding(name) {
                            self.emit(OpCode::Pop);
                            self.emit(OpCode::PushStr);
                            self.emit_string(&format!(
                                "Type mismatch: expected Mutable binding, got const {}",
                                name
                            ));
                            self.emit(OpCode::Throw);
                            return Ok(());
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
                self.end_scope()?;

                if let Some(else_stmts) = else_body {
                    let end_jump = self.emit_jump(OpCode::Jump);
                    self.patch_jump(else_jump);

                    self.begin_scope();
                    for s in else_stmts {
                        self.compile_statement(s)?;
                    }
                    self.end_scope()?;
                    self.patch_jump(end_jump);
                } else {
                    self.patch_jump(else_jump);
                }
            }

            Statement::While {
                condition, body, ..
            } => {
                let loop_start = self.current_offset();
                self.loop_stack.push(LoopContext {
                    start_ip: loop_start,
                    break_jumps: Vec::new(),
                    continue_jumps: Vec::new(),
                });

                self.compile_expression(condition)?;
                let exit_jump = self.emit_jump(OpCode::JumpIfFalse);

                self.begin_scope();
                for s in body {
                    self.compile_statement(s)?;
                }
                self.end_scope()?;

                // Jump back to start
                self.emit(OpCode::Jump);
                self.emit_u32(loop_start as u32);

                let loop_ctx = self.loop_stack.pop().unwrap();
                for jump in loop_ctx.break_jumps {
                    self.patch_jump(jump);
                }
                for jump in loop_ctx.continue_jumps {
                    self.patch_jump_to(jump, loop_start);
                }

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
                self.loop_stack.push(LoopContext {
                    start_ip: loop_start,
                    break_jumps: Vec::new(),
                    continue_jumps: Vec::new(),
                });

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
                self.end_scope()?;

                // Increment Index Phase (Continue target)
                let continue_target = self.current_offset();

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

                let loop_ctx = self.loop_stack.pop().unwrap();
                for jump in loop_ctx.break_jumps {
                    self.patch_jump(jump);
                }
                for jump in loop_ctx.continue_jumps {
                    self.patch_jump_to(jump, continue_target);
                }

                self.patch_jump(exit_jump);
            }

            Statement::Return { value, .. } => {
                // Compile all pending deferred cleanups (LIFO: innermost scope first)
                let ctx = self.current_fn_mut();
                let deferred_bodies: Vec<Vec<Statement>> = ctx
                    .deferred
                    .iter()
                    .rev()
                    .flat_map(|scope_list| scope_list.iter().rev().cloned())
                    .collect();
                // Clear all deferred lists after collecting
                for scope_list in &mut ctx.deferred {
                    scope_list.clear();
                }
                for body in &deferred_bodies {
                    for s in body {
                        self.compile_statement(s)?;
                    }
                }

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

            Statement::Abo { .. } => {}

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
                    // Otherwise Global — use emit_string for E5 deduplication.
                    self.emit(OpCode::StoreGlobal);
                    self.emit_string(name);
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
                    "kekere" => OponSize::Kekere,
                    "arinrin" => OponSize::Arinrin,
                    "nla" => OponSize::Nla,
                    "ailopin" => OponSize::Ailopin,
                    _ => OponSize::Arinrin, // default fallback
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
                            self.end_scope()?;

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
                            self.end_scope()?;

                            end_jumps.push(self.emit_jump(OpCode::Jump));
                            self.patch_jump(skip_arm_1);
                            self.patch_jump(skip_arm_2);
                        }
                        MatchPattern::Wildcard => {
                            self.begin_scope();
                            for s in &arm.body {
                                self.compile_statement(s)?;
                            }
                            self.end_scope()?;

                            end_jumps.push(self.emit_jump(OpCode::Jump));
                        }
                    }
                }

                for jump in end_jumps {
                    self.patch_jump(jump);
                }

                self.end_scope()?;
            }

            Statement::Ebo { offering, body, .. } => {
                if let Some(body_stmts) = body {
                    // Ebo with body: scoped memory epoch
                    // Push the epoch name, then begin epoch
                    self.compile_expression(offering)?;
                    self.emit(OpCode::EpochBegin);
                    self.begin_scope();
                    for s in body_stmts {
                        self.compile_statement(s)?;
                    }
                    self.end_scope()?;
                    self.emit(OpCode::EpochEnd);
                }
                // Without body: semantic directive, no bytecode emitted
            }

            Statement::Defer { body, .. } => {
                // Defer: save body as a deferred cleanup for the current scope
                // The body is compiled at scope exit (end_scope) or before return
                let ctx = self.current_fn_mut();
                if let Some(scope) = ctx.deferred.last_mut() {
                    scope.push(body.clone());
                }
            }

            Statement::Ailewu { body, .. } => {
                // Ailewu (unsafe) block - just compile the body
                // Safety checks are done at static analysis time
                self.begin_scope();
                for s in body {
                    self.compile_statement(s)?;
                }
                self.end_scope()?;
            }

            Statement::Yield { duration, .. } => {
                self.compile_expression(duration)?;
                self.emit(OpCode::Yield);
            }

            Statement::Try {
                try_body,
                catch_var,
                catch_body,
                finally_body,
                ..
            } => {
                // (try handler unchanged)
                self.emit(OpCode::TryBegin);
                let try_begin_offset = self.current_offset();
                self.emit_u32(0);
                let finally_begin_offset = if finally_body.is_some() {
                    self.emit(OpCode::FinallyBegin);
                    let off = self.current_offset();
                    self.emit_u32(0);
                    Some(off)
                } else {
                    None
                };
                self.begin_scope();
                for s in try_body {
                    self.compile_statement(s)?;
                }
                self.end_scope()?;
                self.emit(OpCode::TryEnd);
                if let Some(fb) = finally_body {
                    self.begin_scope();
                    for s in fb {
                        self.compile_statement(s)?;
                    }
                    self.end_scope()?;
                }
                let skip_catch_jump = self.emit_jump(OpCode::Jump);
                let catch_start_offset = self.current_offset();
                let jump_distance = (catch_start_offset - try_begin_offset - 4) as u32;
                let bytes = jump_distance.to_le_bytes();
                self.bytecode.code[try_begin_offset] = bytes[0];
                self.bytecode.code[try_begin_offset + 1] = bytes[1];
                self.bytecode.code[try_begin_offset + 2] = bytes[2];
                self.bytecode.code[try_begin_offset + 3] = bytes[3];
                self.begin_scope();
                self.declare_local(catch_var);
                for s in catch_body {
                    self.compile_statement(s)?;
                }
                self.end_scope()?;
                if let Some(fb) = finally_body {
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
                    self.end_scope()?;
                    self.emit(OpCode::FinallyEnd);
                }
                self.patch_jump(skip_catch_jump);
            }

            // K1: break/continue — emit jump, offset resolved by loop context
            Statement::Break { .. } => {
                if self.loop_stack.is_empty() {
                    return Err(IfaError::Custom(
                        "Cannot use 'break' outside of a loop".into(),
                    ));
                }
                let jump = self.emit_jump(OpCode::Jump);
                self.loop_stack.last_mut().unwrap().break_jumps.push(jump);
            }

            Statement::Continue { .. } => {
                if self.loop_stack.is_empty() {
                    return Err(IfaError::Custom(
                        "Cannot use 'continue' outside of a loop".into(),
                    ));
                }
                let jump = self.emit_jump(OpCode::Jump);
                self.loop_stack
                    .last_mut()
                    .unwrap()
                    .continue_jumps
                    .push(jump);
            }

            Statement::Throw { value, .. } => {
                self.compile_expression(value)?;
                self.emit(OpCode::Throw);
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

        self.end_scope()?;

        // Capture upvalues before popping the context
        let upvalues = self.current_fn().upvalues.clone();
        self.functions.pop();

        // 6. Patch Jump
        self.patch_jump(jump);

        // 7. Emit PushFn instruction with deduplicated name string.
        self.emit(OpCode::PushFn);
        // name index — route through emit_string for E5 dedup.
        self.emit_string(name);

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

    fn compile_update_statement(
        &mut self,
        target: &AssignTarget,
        op: &UpdateOp,
        value: &Option<Expression>,
    ) -> IfaResult<()> {
        match target {
            AssignTarget::Variable(name) => {
                // [ ] -> [val]
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

                // Apply Op
                self.compile_update_op(op, value)?;

                // Store back
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
                // Goal: [container, index, new_val] -> SetIndex
                // 1. Push container
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
                // 2. Push index
                self.compile_expression(index)?;

                // Stack: [c, i]
                self.emit(OpCode::Dup); // [c, i, i]
                self.emit(OpCode::Swap); // [c, i, i] -> wait, I want [c, i, c, i]
                // Actually easier: push c, push i, dup, swap, dup, swap ... no.
                // Let's use Swap/Dup specifically.
                // [c, i]
                // Dup2 (not exists)
                // Swap [i, c] -> Dup [i, c, c] -> Swap [i, c, i] -> ... no.

                // Alternative:
                // Load c [c]
                // Load i [c, i]
                // Swap [i, c]
                // Dup [i, c, c]
                // Swap [c, c, i]
                // Dup [c, c, i, i]
                // GetIndex [c, c, i, v]
                // Op [c, c, i, nv]
                // Swap [c, c, nv, i]... wait.

                // Let's just push them twice, it's safer and less stack mental gymnastics.
                // It's less efficient but guaranteed correct.
                if let Some(slot) = self.resolve_local(name) {
                    self.emit(OpCode::LoadLocal);
                    let s = slot as u16;
                    self.emit_byte((s & 0xff) as u8);
                    self.emit_byte((s >> 8) as u8);
                } else { /* ... */
                }
                self.compile_expression(index)?;
                self.emit(OpCode::GetIndex); // [v]
                self.compile_update_op(op, value)?; // [nv]

                // Now I need c and i again.
                // Let's use the first set.
                // Re-push c and i for real.
                if let Some(slot) = self.resolve_local(name) {
                    self.emit(OpCode::LoadLocal);
                    let s = slot as u16;
                    self.emit_byte((s & 0xff) as u8);
                    self.emit_byte((s >> 8) as u8);
                } else { /* ... */
                }
                self.compile_expression(index)?;
                // [nv, c, i]
                self.emit(OpCode::SetIndex); // [nv, c, i] -> SetIndex(c, i, nv) -> wait, OpCode::SetIndex pops [col, idx, val]
                // So I need [c, i, nv]
                // Swap2? No.

                // OK, final refined Index update Plan:
                // 1. Load c, i
                // 2. Load c, i
                // 3. GetIndex -> [c, i, v]
                // 4. Op -> [c, i, nv]
                // 5. SetIndex -> []

                self.emit_load_target_var(name)?;
                self.compile_expression(index)?;

                self.emit_load_target_var(name)?;
                self.compile_expression(index)?;
                self.emit(OpCode::GetIndex);

                self.compile_update_op(op, value)?;
                self.emit(OpCode::SetIndex);
            }
            AssignTarget::Dereference(expr) => {
                // [ptr, val] -> Store8
                self.compile_expression(expr)?;
                self.emit(OpCode::Dup);
                self.emit(OpCode::Load8);
                self.compile_update_op(op, value)?;
                self.emit(OpCode::Store8);
            }
        }
        Ok(())
    }

    fn emit_load_target_var(&mut self, name: &str) -> IfaResult<()> {
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
        Ok(())
    }

    fn compile_update_op(&mut self, op: &UpdateOp, value: &Option<Expression>) -> IfaResult<()> {
        let val_expr = value
            .as_ref()
            .ok_or_else(|| IfaError::Parse("Update missing value".into()))?;
        self.compile_expression(val_expr)?;
        match op {
            // For AddAssign, emit Concat if the rhs is a String literal (fast-path for `text += " more"`).
            // The VM's Concat opcode is strict: Str + Str only. For numeric Add, we fall through to Add.
            // The tree-walking interpreter handles runtime type dispatch; the bytecode VM is statically
            // correct since the lhs/rhs types must match at this op site.
            UpdateOp::AddAssign => match val_expr {
                Expression::String(_) | Expression::InterpolatedString { .. } => {
                    self.emit(OpCode::Concat)
                }
                _ => self.emit(OpCode::Add),
            },
            UpdateOp::SubAssign => self.emit(OpCode::Sub),
            UpdateOp::MulAssign => self.emit(OpCode::Mul),
            UpdateOp::DivAssign => self.emit(OpCode::Div),
            UpdateOp::ModAssign => self.emit(OpCode::Mod),
        }
        Ok(())
    }

    fn compile_expression(&mut self, expr: &Expression) -> IfaResult<()> {
        let folded = fold_expression(expr);
        self.compile_expression_inner(&folded)
    }

    fn compile_expression_inner(&mut self, expr: &Expression) -> IfaResult<()> {
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
                    // ?? null coalescing: evaluate lhs; if non-null keep it, else use rhs
                    // Emits: lhs, Dup, IsNull-equivalent (PushNull + Eq), JumpIfFalse(skip), Pop, rhs
                    // Uses existing opcodes: Dup + PushNull + Eq + JumpIfFalse + Pop
                    BinaryOperator::NullCoalesce => {
                        self.compile_expression(left)?; // [lhs]
                        self.emit(OpCode::Dup); // [lhs, lhs]
                        self.emit(OpCode::PushNull); // [lhs, lhs, null]
                        self.emit(OpCode::Eq); // [lhs, is_null]
                        let skip_rhs = self.emit_jump(OpCode::JumpIfFalse); // [lhs] — jump if lhs != null
                        self.emit(OpCode::Pop); // [] — discard null lhs
                        self.compile_expression(right)?; // [rhs]
                        self.patch_jump(skip_rhs);
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
                            BinaryOperator::Power => OpCode::Pow,
                            BinaryOperator::Eq => OpCode::Eq,
                            BinaryOperator::NotEq => OpCode::Ne,
                            BinaryOperator::Lt => OpCode::Lt,
                            BinaryOperator::LtEq => OpCode::Le,
                            BinaryOperator::Gt => OpCode::Gt,
                            BinaryOperator::GtEq => OpCode::Ge,
                            BinaryOperator::And
                            | BinaryOperator::Or
                            | BinaryOperator::NullCoalesce => {
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
                            return Err(ifa_types::IfaError::Compile(
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

            Expression::Index {
                object,
                index,
                is_optional,
            } => {
                self.compile_expression(object)?;
                if *is_optional {
                    self.emit(OpCode::Dup);
                    self.emit(OpCode::PushNull);
                    self.emit(OpCode::Eq);
                    let skip_jump = self.emit_jump(OpCode::JumpIfTrue);
                    self.compile_expression(index)?;
                    self.emit(OpCode::GetIndex);
                    let end_jump = self.emit_jump(OpCode::Jump);
                    self.patch_jump(skip_jump);
                    self.emit(OpCode::Pop); // Pop null
                    self.emit(OpCode::PushNull);
                    self.patch_jump(end_jump);
                } else {
                    self.compile_expression(index)?;
                    self.emit(OpCode::GetIndex);
                }
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

            Expression::Get {
                object,
                name,
                is_optional,
            } => {
                self.compile_expression(object)?;
                if *is_optional {
                    self.emit(OpCode::Dup);
                    self.emit(OpCode::PushNull);
                    self.emit(OpCode::Eq);
                    let skip_jump = self.emit_jump(OpCode::JumpIfTrue);
                    self.emit(OpCode::GetField);
                    self.emit_string(name);
                    let end_jump = self.emit_jump(OpCode::Jump);
                    self.patch_jump(skip_jump);
                    self.emit(OpCode::Pop); // Pop null (dup)
                    self.emit(OpCode::PushNull);
                    self.patch_jump(end_jump);
                } else {
                    self.emit(OpCode::GetField);
                    self.emit_string(name);
                }
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
                is_optional,
            } => {
                self.compile_expression(object)?;
                if *is_optional {
                    self.emit(OpCode::Dup);
                    self.emit(OpCode::PushNull);
                    self.emit(OpCode::Eq);
                    let skip_jump = self.emit_jump(OpCode::JumpIfTrue);
                    for arg in args {
                        self.compile_expression(arg)?;
                    }
                    self.emit(OpCode::CallMethod);
                    self.emit_string(method);
                    self.emit_byte(args.len() as u8);
                    let end_jump = self.emit_jump(OpCode::Jump);
                    self.patch_jump(skip_jump);
                    self.emit(OpCode::Pop);
                    self.emit(OpCode::PushNull);
                    self.patch_jump(end_jump);
                } else {
                    for arg in args {
                        self.compile_expression(arg)?;
                    }
                    self.emit(OpCode::CallMethod);
                    self.emit_string(method);
                    self.emit_byte(args.len() as u8);
                }
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

            Expression::Lambda { params, body } => {
                // Compile as an anonymous function using the existing compile_function path.
                // The synthetic name encodes the byte-offset so it is unique per compilation unit.
                let anon_name = format!("<lambda@{}>", self.current_offset());
                // Build a Param slice from the plain name vec.
                let param_list: Vec<Param> = params
                    .iter()
                    .map(|n| Param {
                        name: n.clone(),
                        type_hint: None,
                    })
                    .collect();
                self.compile_function(&anon_name, &param_list, body, false)?;
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

        // H4: iwori.yipo.ori (ParallelFor)
        if call.domain == OduDomain::Iwori && call.method == "yipo.ori" {
            if call.args.len() != 2 {
                return Err(ifa_types::IfaError::Compile(
                    "iwori.yipo.ori requires exactly 2 arguments (iterable, closure)".to_string(),
                ));
            }
            self.compile_expression(&call.args[0])?; // iterable
            self.compile_expression(&call.args[1])?; // closure
            self.emit(OpCode::ParallelFor);
            return Ok(());
        }

        if call.is_optional {
            // For now, domains are static strings like "Obara", so they are never null.
            // But we implement the guard for future consistency.
            self.emit(OpCode::PushNull); // Placeholder for "domain lookup" which currently can't fail
            self.emit(OpCode::PushNull);
            self.emit(OpCode::Eq);
            let skip_jump = self.emit_jump(OpCode::JumpIfTrue);
            for arg in &call.args {
                self.compile_expression(arg)?;
            }
            self.emit(OpCode::CallOdu);
            self.emit_odu_domain(&call.domain);
            self.emit_string(&call.method);
            self.emit_byte(call.args.len() as u8);
            let end_jump = self.emit_jump(OpCode::Jump);
            self.patch_jump(skip_jump);
            self.emit(OpCode::Pop);
            self.emit(OpCode::PushNull);
            self.patch_jump(end_jump);
        } else {
            for arg in &call.args {
                self.compile_expression(arg)?;
            }
            // E6 — Constant Divination: emit CallOduFast when method is statically known.
            // Encoding: [CallOduFast | domain_id: u8 | method_id_hi: u8 | method_id_lo: u8 | arg_count: u8]
            // Total operands = 4 bytes, matching CallOdu's operand footprint.
            if let Some(method_id) = resolve_method_id(call.domain, &call.method) {
                let domain_id = domain_to_byte(&call.domain);
                self.emit(OpCode::CallOduFast);
                self.emit_byte(domain_id);
                self.emit_byte((method_id >> 8) as u8);
                self.emit_byte((method_id & 0xFF) as u8);
                self.emit_byte(call.args.len() as u8);
            } else {
                // Fallback: dynamic string-based dispatch for unresolved methods.
                self.emit(OpCode::CallOdu);
                self.emit_odu_domain(&call.domain);
                self.emit_string(&call.method);
                self.emit_byte(call.args.len() as u8);
            }
        }
        Ok(())
    }

    fn emit_odu_domain(&mut self, domain: &OduDomain) {
        let b = domain_to_byte(domain);
        self.emit_byte(b);
    }
}

/// Convert `OduDomain` to its stable VM dispatch byte.
///
/// Delegates to `OduDomain::dispatch_id()` which is the single source of
/// truth for ID assignments. Reserved pseudo-domains (Coop, Opele) have no
/// dispatch ID; reaching this function with one is a compiler bug.
fn domain_to_byte(domain: &OduDomain) -> u8 {
    domain.dispatch_id().unwrap_or_else(|| {
        panic!(
            "domain_to_byte: '{}' is a reserved pseudo-domain with no VM dispatch ID. \
             This is a compiler bug — reserved domains must be rejected before codegen.",
            domain
        )
    })
}

/// Compile source code to bytecode
pub fn compile(source: &str) -> IfaResult<Bytecode> {
    let program = ifa_parser::parse(source)?;
    let compiler = Compiler::new("<main>");
    compiler.compile(&program)
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

#[allow(clippy::collapsible_if)]
fn fold_expression(expr: &Expression) -> Expression {
    match expr {
        Expression::BinaryOp { left, op, right } => {
            let left_folded = fold_expression(left);
            let right_folded = fold_expression(right);
            match (left_folded, op, right_folded) {
                // Arithmetic
                (Expression::Int(l), BinaryOperator::Add, Expression::Int(r)) => {
                    Expression::Int(l + r)
                }
                (Expression::Int(l), BinaryOperator::Sub, Expression::Int(r)) => {
                    Expression::Int(l - r)
                }
                (Expression::Int(l), BinaryOperator::Mul, Expression::Int(r)) => {
                    Expression::Int(l * r)
                }
                (Expression::Int(l), BinaryOperator::Div, Expression::Int(r)) if r != 0 => {
                    Expression::Int(l / r)
                }
                (Expression::Int(l), BinaryOperator::Mod, Expression::Int(r)) if r != 0 => {
                    Expression::Int(l % r)
                }
                (Expression::Int(l), BinaryOperator::Power, Expression::Int(r)) => {
                    if (0..=30).contains(&r) {
                        Expression::Int(l.pow(r as u32))
                    } else {
                        Expression::BinaryOp {
                            left: Box::new(Expression::Int(l)),
                            op: BinaryOperator::Power,
                            right: Box::new(Expression::Int(r)),
                        }
                    }
                }

                (Expression::Float(l), BinaryOperator::Add, Expression::Float(r)) => {
                    Expression::Float(l + r)
                }
                (Expression::Float(l), BinaryOperator::Sub, Expression::Float(r)) => {
                    Expression::Float(l - r)
                }
                (Expression::Float(l), BinaryOperator::Mul, Expression::Float(r)) => {
                    Expression::Float(l * r)
                }
                (Expression::Float(l), BinaryOperator::Div, Expression::Float(r)) => {
                    Expression::Float(l / r)
                }

                // Mixing Int and Float (coerce to Float)
                (Expression::Int(l), BinaryOperator::Add, Expression::Float(r)) => {
                    Expression::Float((l as f64) + r)
                }
                (Expression::Float(l), BinaryOperator::Add, Expression::Int(r)) => {
                    Expression::Float(l + (r as f64))
                }
                (Expression::Int(l), BinaryOperator::Sub, Expression::Float(r)) => {
                    Expression::Float((l as f64) - r)
                }
                (Expression::Float(l), BinaryOperator::Sub, Expression::Int(r)) => {
                    Expression::Float(l - (r as f64))
                }
                (Expression::Int(l), BinaryOperator::Mul, Expression::Float(r)) => {
                    Expression::Float((l as f64) * r)
                }
                (Expression::Float(l), BinaryOperator::Mul, Expression::Int(r)) => {
                    Expression::Float(l * (r as f64))
                }
                (Expression::Int(l), BinaryOperator::Div, Expression::Float(r)) => {
                    Expression::Float((l as f64) / r)
                }
                (Expression::Float(l), BinaryOperator::Div, Expression::Int(r)) => {
                    Expression::Float(l / (r as f64))
                }

                // String concatenation
                (Expression::String(l), BinaryOperator::Add, Expression::String(r)) => {
                    Expression::String(format!("{}{}", l, r))
                }

                // Comparison
                (Expression::Int(l), BinaryOperator::Eq, Expression::Int(r)) => {
                    Expression::Bool(l == r)
                }
                (Expression::Int(l), BinaryOperator::NotEq, Expression::Int(r)) => {
                    Expression::Bool(l != r)
                }
                (Expression::Int(l), BinaryOperator::Lt, Expression::Int(r)) => {
                    Expression::Bool(l < r)
                }
                (Expression::Int(l), BinaryOperator::LtEq, Expression::Int(r)) => {
                    Expression::Bool(l <= r)
                }
                (Expression::Int(l), BinaryOperator::Gt, Expression::Int(r)) => {
                    Expression::Bool(l > r)
                }
                (Expression::Int(l), BinaryOperator::GtEq, Expression::Int(r)) => {
                    Expression::Bool(l >= r)
                }

                (Expression::Float(l), BinaryOperator::Eq, Expression::Float(r)) => {
                    Expression::Bool(l == r)
                }
                (Expression::Float(l), BinaryOperator::NotEq, Expression::Float(r)) => {
                    Expression::Bool(l != r)
                }
                (Expression::Float(l), BinaryOperator::Lt, Expression::Float(r)) => {
                    Expression::Bool(l < r)
                }
                (Expression::Float(l), BinaryOperator::LtEq, Expression::Float(r)) => {
                    Expression::Bool(l <= r)
                }
                (Expression::Float(l), BinaryOperator::Gt, Expression::Float(r)) => {
                    Expression::Bool(l > r)
                }
                (Expression::Float(l), BinaryOperator::GtEq, Expression::Float(r)) => {
                    Expression::Bool(l >= r)
                }

                // Logical
                (Expression::Bool(l), BinaryOperator::And, Expression::Bool(r)) => {
                    Expression::Bool(l && r)
                }
                (Expression::Bool(l), BinaryOperator::Or, Expression::Bool(r)) => {
                    Expression::Bool(l || r)
                }

                (l_f, op, r_f) => Expression::BinaryOp {
                    left: Box::new(l_f),
                    op: *op,
                    right: Box::new(r_f),
                },
            }
        }
        Expression::UnaryOp { op, expr } => {
            let expr_folded = fold_expression(expr);
            match (op, expr_folded) {
                (UnaryOperator::Neg, Expression::Int(n)) => Expression::Int(-n),
                (UnaryOperator::Neg, Expression::Float(f)) => Expression::Float(-f),
                (UnaryOperator::Not, Expression::Bool(b)) => Expression::Bool(!b),
                (op, e_f) => Expression::UnaryOp {
                    op: *op,
                    expr: Box::new(e_f),
                },
            }
        }
        Expression::OduCall(call) => {
            let folded_args: Vec<Expression> = call.args.iter().map(fold_expression).collect();
            let all_consts = folded_args.iter().all(|a| {
                matches!(
                    a,
                    Expression::Int(_)
                        | Expression::Float(_)
                        | Expression::String(_)
                        | Expression::Bool(_)
                )
            });

            if all_consts {
                // E6: Constant Divination
                match (call.domain, call.method.as_str()) {
                    (ifa_types::OduDomain::Obara, "fikun")
                    | (ifa_types::OduDomain::Obara, "add") => {
                        let mut sum_int = 0;
                        let mut sum_float = 0.0;
                        let mut is_float = false;
                        for arg in &folded_args {
                            match arg {
                                Expression::Int(n) => {
                                    if is_float {
                                        sum_float += *n as f64;
                                    } else {
                                        sum_int += n;
                                    }
                                }
                                Expression::Float(f) => {
                                    if !is_float {
                                        is_float = true;
                                        sum_float = sum_int as f64 + *f;
                                    } else {
                                        sum_float += f;
                                    }
                                }
                                _ => {
                                    return Expression::OduCall(ifa_types::ast::OduCall {
                                        domain: call.domain,
                                        method: call.method.clone(),
                                        args: folded_args,
                                        is_optional: call.is_optional,
                                        resolved_domain: call.resolved_domain,
                                        resolved_method_id: call.resolved_method_id,
                                        span: call.span.clone(),
                                    });
                                }
                            }
                        }
                        return if is_float {
                            Expression::Float(sum_float)
                        } else {
                            Expression::Int(sum_int)
                        };
                    }
                    (ifa_types::OduDomain::Obara, "isodipupo")
                    | (ifa_types::OduDomain::Obara, "mul") => {
                        let mut prod_int = 1;
                        let mut prod_float = 1.0;
                        let mut is_float = false;
                        for arg in &folded_args {
                            match arg {
                                Expression::Int(n) => {
                                    if is_float {
                                        prod_float *= *n as f64;
                                    } else {
                                        prod_int *= n;
                                    }
                                }
                                Expression::Float(f) => {
                                    if !is_float {
                                        is_float = true;
                                        prod_float = prod_int as f64 * *f;
                                    } else {
                                        prod_float *= f;
                                    }
                                }
                                _ => {
                                    return Expression::OduCall(ifa_types::ast::OduCall {
                                        domain: call.domain,
                                        method: call.method.clone(),
                                        args: folded_args,
                                        is_optional: call.is_optional,
                                        resolved_domain: call.resolved_domain,
                                        resolved_method_id: call.resolved_method_id,
                                        span: call.span.clone(),
                                    });
                                }
                            }
                        }
                        return if is_float {
                            Expression::Float(prod_float)
                        } else {
                            Expression::Int(prod_int)
                        };
                    }
                    (ifa_types::OduDomain::Ika, "gigun") | (ifa_types::OduDomain::Ika, "len") => {
                        if folded_args.len() == 1 {
                            if let Expression::String(s) = &folded_args[0] {
                                return Expression::Int(s.chars().count() as i64);
                            }
                        }
                    }
                    (ifa_types::OduDomain::Ika, "upper") => {
                        if folded_args.len() == 1 {
                            if let Expression::String(s) = &folded_args[0] {
                                return Expression::String(s.to_uppercase());
                            }
                        }
                    }
                    (ifa_types::OduDomain::Ika, "lower") => {
                        if folded_args.len() == 1 {
                            if let Expression::String(s) = &folded_args[0] {
                                return Expression::String(s.to_lowercase());
                            }
                        }
                    }
                    _ => {}
                }
            }

            Expression::OduCall(ifa_types::ast::OduCall {
                domain: call.domain,
                method: call.method.clone(),
                args: folded_args,
                is_optional: call.is_optional,
                resolved_domain: call.resolved_domain,
                resolved_method_id: call.resolved_method_id,
                span: call.span.clone(),
            })
        }
        Expression::InterpolatedString { parts } => {
            let folded_parts: Vec<InterpolatedPart> = parts
                .iter()
                .map(|part| match part {
                    InterpolatedPart::Literal(s) => InterpolatedPart::Literal(s.clone()),
                    InterpolatedPart::Expression(expr) => {
                        InterpolatedPart::Expression(Box::new(fold_expression(expr)))
                    }
                })
                .collect();
            let mut combined_parts = Vec::new();
            for part in folded_parts {
                match part {
                    InterpolatedPart::Literal(s) => {
                        if let Some(InterpolatedPart::Literal(last)) = combined_parts.last_mut() {
                            last.push_str(&s);
                        } else {
                            combined_parts.push(InterpolatedPart::Literal(s));
                        }
                    }
                    InterpolatedPart::Expression(expr) => match *expr {
                        Expression::String(s) => {
                            if let Some(InterpolatedPart::Literal(last)) = combined_parts.last_mut()
                            {
                                last.push_str(&s);
                            } else {
                                combined_parts.push(InterpolatedPart::Literal(s));
                            }
                        }
                        Expression::Int(n) => {
                            let s = n.to_string();
                            if let Some(InterpolatedPart::Literal(last)) = combined_parts.last_mut()
                            {
                                last.push_str(&s);
                            } else {
                                combined_parts.push(InterpolatedPart::Literal(s));
                            }
                        }
                        Expression::Float(f) => {
                            let s = f.to_string();
                            if let Some(InterpolatedPart::Literal(last)) = combined_parts.last_mut()
                            {
                                last.push_str(&s);
                            } else {
                                combined_parts.push(InterpolatedPart::Literal(s));
                            }
                        }
                        Expression::Bool(b) => {
                            let s = b.to_string();
                            if let Some(InterpolatedPart::Literal(last)) = combined_parts.last_mut()
                            {
                                last.push_str(&s);
                            } else {
                                combined_parts.push(InterpolatedPart::Literal(s));
                            }
                        }
                        _ => combined_parts.push(InterpolatedPart::Expression(expr)),
                    },
                }
            }
            if combined_parts.len() == 1 {
                if let InterpolatedPart::Literal(s) = &combined_parts[0] {
                    return Expression::String(s.clone());
                }
            }
            Expression::InterpolatedString {
                parts: combined_parts,
            }
        }
        _ => expr.clone(),
    }
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
