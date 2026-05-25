//! # Compile-Time Checks
//!
//! Static analysis checks for Ifá-Lang programs.
//! Ported from legacy/src/linter.py and legacy/src/validator.py

use crate::diagnose::Babalawo;
use crate::iwa::IwaEngine;
use crate::movement::{MoveTracker, MoveCheckResult, is_copy_eligible, move_args_from_odu_call};
use crate::taboo::TabooEnforcer;
use ifa_types::ast::{Expression, Program, Statement, TypeHint, Visibility};
use crate::Severity;
use std::collections::{HashMap, HashSet};

/// Context for linting - tracks state as we walk the AST
#[derive(Debug)]
pub struct LintContext {
    /// Variables that have been defined (with their declaration span)
    pub defined_vars: HashMap<String, ifa_types::ast::Span>,
    /// Variables that have been used
    pub used_vars: HashSet<String>,
    /// Variable types (for static type checking)
    /// Key: variable name, Value: declared type
    pub var_types: HashMap<String, TypeHint>,
    /// Variable and function visibility
    pub var_visibility: HashMap<String, Visibility>,
    /// The domain (Odu) where the variable was defined, if any
    pub var_domain: HashMap<String, Option<String>>,
    /// Imports
    pub imports: HashSet<String>,
    /// Current function name (if inside one)
    pub current_function: Option<String>,
    /// Whether we've seen a return in current function
    pub has_return: bool,
    /// Resource lifecycle tracking (open -> close)
    pub open_resources: HashMap<String, (usize, usize)>, // resource -> (line, col)
    /// Ìwà Engine - resource lifecycle validation
    pub iwa_engine: IwaEngine,
    /// Èèwọ̀ Enforcer - architectural constraints
    pub taboo_enforcer: TabooEnforcer,
    /// Whether we're inside an ailewu (unsafe) block
    pub in_ailewu: bool,
    /// Active #opon directive size (if declared)
    pub opon_size: Option<String>,
    /// Whether currently inside an async (daro) function
    pub in_async_function: bool,
    /// Current domain (class/odu) name, for visibility scoping
    pub current_domain: Option<String>,
    /// H1: Move tracker — enforces linear type discipline at actor boundaries.
    pub move_tracker: MoveTracker,
    /// H4: Whether currently inside a parallel execution body (e.g. iwori.yipo.ori)
    pub in_parallel_body: bool,
    /// H4: Variables declared locally inside the parallel body
    pub parallel_locals: HashSet<String>,
}

impl Default for LintContext {
    fn default() -> Self {
        Self::new()
    }
}

impl LintContext {
    pub fn new() -> Self {
        Self {
            defined_vars: HashMap::new(),
            used_vars: HashSet::new(),
            var_types: HashMap::new(),
            var_visibility: HashMap::new(),
            var_domain: HashMap::new(),
            imports: HashSet::new(),
            current_function: None,
            has_return: false,
            open_resources: HashMap::new(),
            iwa_engine: IwaEngine::new(true),
            taboo_enforcer: TabooEnforcer::new(),
            in_ailewu: false,
            opon_size: None,
            in_async_function: false,
            current_domain: None,
            move_tracker: MoveTracker::new(),
            in_parallel_body: false,
            parallel_locals: HashSet::new(),
        }
    }

    pub fn define_var(&mut self, name: &str, span: ifa_types::ast::Span, visibility: Visibility) {
        self.defined_vars.insert(name.to_string(), span);
        self.var_visibility.insert(name.to_string(), visibility);
        self.var_domain.insert(name.to_string(), self.current_domain.clone());
        self.move_tracker.declare(name);
    }

    /// Define a variable with a type hint
    pub fn define_var_typed(&mut self, name: &str, type_hint: TypeHint, span: ifa_types::ast::Span, visibility: Visibility) {
        self.defined_vars.insert(name.to_string(), span);
        self.var_types.insert(name.to_string(), type_hint);
        self.var_visibility.insert(name.to_string(), visibility);
        self.var_domain.insert(name.to_string(), self.current_domain.clone());
        self.move_tracker.declare(name);
    }

    pub fn use_var(&mut self, name: &str) {
        self.used_vars.insert(name.to_string());
    }

    /// Get the declared type of a variable (if statically typed)
    pub fn get_var_type(&self, name: &str) -> Option<&TypeHint> {
        self.var_types.get(name)
    }

    /// Get the visibility of a variable
    pub fn get_var_visibility(&self, name: &str) -> Option<&Visibility> {
        self.var_visibility.get(name)
    }

    /// Get the domain where a variable was defined
    pub fn get_var_domain(&self, name: &str) -> Option<&Option<String>> {
        self.var_domain.get(name)
    }

    /// Check if a symbol is accessible from the current context
    pub fn is_accessible(&self, visibility: &Visibility, target_domain: Option<&String>) -> bool {
        match visibility {
            Visibility::Public | Visibility::Crate => true,
            Visibility::Private => {
                if let Some(target) = target_domain {
                    // Must be in the same domain to access private members
                    self.current_domain.as_ref() == Some(target)
                } else {
                    // Top-level privates are accessible within the same file (linting unit)
                    true
                }
            }
        }
    }

    pub fn enter_function(&mut self, name: &str, is_async: bool) {
        self.current_function = Some(name.to_string());
        self.has_return = false;
        self.in_async_function = is_async;
    }

    pub fn exit_function(&mut self) {
        self.current_function = None;
        self.has_return = false;
        self.in_async_function = false;
    }

    pub fn enter_domain(&mut self, name: &str) {
        self.current_domain = Some(name.to_string());
    }

    pub fn exit_domain(&mut self) {
        self.current_domain = None;
    }
}

/// Configuration for the Babalawo linter
#[derive(Debug, Clone, Copy)]
pub struct BabalawoConfig {
    /// Include wisdom/proverbs in diagnostics (slower)
    pub include_wisdom: bool,
}

impl Default for BabalawoConfig {
    fn default() -> Self {
        Self {
            include_wisdom: true,
        }
    }
}

/// Check a program with default configuration
pub fn check_program(program: &Program, filename: &str) -> Babalawo {
    check_program_with_config(program, filename, BabalawoConfig::default())
}

/// Check a program with custom configuration (returns diagnostics only)
pub fn check_program_with_config(
    program: &Program,
    filename: &str,
    config: BabalawoConfig,
) -> Babalawo {
    let (babalawo, _) = analyze_program(program, filename, config);
    babalawo
}

/// Analyze a program returning both diagnostics and symbol context
pub fn analyze_program(
    program: &Program,
    filename: &str,
    config: BabalawoConfig,
) -> (Babalawo, LintContext) {
    let mut babalawo = Babalawo::new();
    if !config.include_wisdom {
        babalawo = babalawo.fast();
    }
    let mut ctx = LintContext::new();

    // First pass: collect definitions
    for stmt in &program.statements {
        collect_definitions(stmt, &mut ctx);
    }

    // Second pass: check for issues (including Ìwà and Èèwọ̀)
    for stmt in &program.statements {
        check_statement(stmt, &mut ctx, &mut babalawo, filename);
    }

    // Final checks
    check_unused_vars(&ctx, &mut babalawo, filename);
    check_unclosed_resources(&ctx, &mut babalawo, filename);

    // Ìwà Engine: check resource balance
    if !ctx.iwa_engine.check_balance() {
        for debt in ctx.iwa_engine.unclosed_resources() {
            babalawo.error(
                "UNCLOSED_RESOURCE",
                &format!(
                    "Resource '{}' opened at line {} was never closed (needs '{}')",
                    debt.opener, debt.line, debt.required
                ),
                filename,
                debt.line,
                debt.column,
            );
        }
    }

    // Èèwọ̀ Enforcer: check taboo violations
    if !ctx.taboo_enforcer.is_clean() {
        for v in ctx.taboo_enforcer.get_violations() {
            babalawo.error(
                "TABOO_VIOLATION",
                &format!(
                    "Forbidden dependency: '{}' cannot call '{}'",
                    v.caller, v.callee
                ),
                filename,
                v.line,
                v.column,
            );
        }
    }

    // #opon ailopin check — warn about embedded incompatibility
    if ctx.opon_size.as_deref() == Some("ailopin") {
        babalawo.warning(
            "OPON_AILOPIN_UNBOUNDED",
            "#opon ailopin declares unbounded memory — this is incompatible with embedded targets (Ilẹ̀ tier). \
             Use #opon kekere, arinrin, or nla for bare-metal deployments.",
            filename,
            1,
            1,
        );
    }

    (babalawo, ctx)
}

/// Check a program with custom taboos
#[allow(dead_code)]
pub fn check_program_with_taboos(
    program: &Program,
    filename: &str,
    taboos: Vec<(&str, &str)>, // (source, target) forbidden pairs
) -> Babalawo {
    let mut babalawo = Babalawo::new();
    let mut ctx = LintContext::new();

    // Register taboos
    for (source, target) in taboos {
        ctx.taboo_enforcer.add_taboo(source, "", target, "", false);
    }

    // First pass: collect definitions
    for stmt in &program.statements {
        collect_definitions(stmt, &mut ctx);
    }

    // Second pass: check for issues
    for stmt in &program.statements {
        check_statement(stmt, &mut ctx, &mut babalawo, filename);
    }

    // Final checks
    check_unused_vars(&ctx, &mut babalawo, filename);
    check_iwa_balance(&mut ctx, &mut babalawo, filename);
    check_taboo_violations(&ctx, &mut babalawo, filename);

    babalawo
}

/// Check Ìwà Engine balance
#[allow(dead_code)]
fn check_iwa_balance(ctx: &mut LintContext, baba: &mut Babalawo, file: &str) {
    if !ctx.iwa_engine.check_balance() {
        for debt in ctx.iwa_engine.unclosed_resources() {
            baba.error(
                "UNCLOSED_RESOURCE",
                &format!(
                    "Resource '{}' was never closed (needs '{}')",
                    debt.opener, debt.required
                ),
                file,
                debt.line,
                debt.column,
            );
        }
    }
}

/// Check Èèwọ̀ taboo violations
#[allow(dead_code)]
fn check_taboo_violations(ctx: &LintContext, baba: &mut Babalawo, file: &str) {
    for v in ctx.taboo_enforcer.get_violations() {
        baba.error(
            "TABOO_VIOLATION",
            &format!("Èèwọ̀: '{}' cannot call '{}'", v.caller, v.callee),
            file,
            v.line,
            v.column,
        );
    }
}

/// Collect variable and function definitions + Taboos and Opon directives
fn collect_definitions(stmt: &Statement, ctx: &mut LintContext) {
    match stmt {
        Statement::VarDecl {
            name, type_hint, span, visibility, ..
        } => {
            if let Some(th) = type_hint {
                ctx.define_var_typed(name, th.clone(), span.clone(), *visibility);
            } else {
                ctx.define_var(name, span.clone(), *visibility);
            }
        }
        Statement::Const {
            name, value: _, visibility, span,
        } => {
            ctx.define_var(name, span.clone(), *visibility);
        }
        Statement::EseDef {
            name,
            params,
            body,
            span,
            visibility,
            is_async: _,
        } => {
            ctx.define_var(name, span.clone(), *visibility);
            // Parameters are also definitions within the function (private by default)
            for param in params {
                if let Some(th) = &param.type_hint {
                    ctx.define_var_typed(&param.name, th.clone(), span.clone(), Visibility::Private); // Simplification: param uses Ese span
                } else {
                    ctx.define_var(&param.name, span.clone(), Visibility::Private);
                }
            }
            for s in body {
                collect_definitions(s, ctx);
            }
        }
        Statement::OduDef { name, body, span, visibility } => {
            ctx.define_var(name, span.clone(), *visibility);
            for s in body {
                collect_definitions(s, ctx);
            }
        }
        Statement::For { var, iterable: _, body, span } => {
            ctx.define_var(var, span.clone(), Visibility::Private);
            for s in body {
                collect_definitions(s, ctx);
            }
        }
        Statement::If {
            then_body,
            else_body,
            ..
        } => {
            for s in then_body {
                collect_definitions(s, ctx);
            }
            if let Some(else_stmts) = else_body {
                for s in else_stmts {
                    collect_definitions(s, ctx);
                }
            }
        }
        Statement::While { body, .. } => {
            for s in body {
                collect_definitions(s, ctx);
            }
        }
        // Register taboo declarations for enforcement
        Statement::Taboo { source, target, .. } => {
            ctx.taboo_enforcer.add_taboo(source, "", target, "", false);
        }
        // Opon directives — store for cross-check
        Statement::Opon { size, .. } => {
            ctx.opon_size = Some(size.clone());
        }
        _ => {}
    }
}

/// Check a statement for issues
fn check_statement(stmt: &Statement, ctx: &mut LintContext, baba: &mut Babalawo, file: &str) {
    match stmt {
        Statement::VarDecl {
            name,
            value,
            span,
            type_hint,
            ..
        } => {
            check_expression(value, ctx, baba, file, span);

            // Check for self-referencing initialization
            if expression_uses_var(value, name) {
                baba.error(
                    "UNINITIALIZED",
                    &format!("Variable '{}' used in its own initialization", name),
                    file,
                    span.line,
                    span.column,
                );
            }

            // H4: Track parallel locals
            if ctx.in_parallel_body {
                ctx.parallel_locals.insert(name.clone());
            }

            // Type checking for statically typed variables
            if let Some(th) = type_hint {
                // Check if low-level type requires ailewu context
                if th.requires_ailewu() && !ctx.in_ailewu {
                    baba.error(
                        "UNSAFE_OUTSIDE_AILEWU",
                        &format!("Pointer type '{:?}' requires 'ailewu' (unsafe) block", th),
                        file,
                        span.line,
                        span.column,
                    );
                }

                // Check expression type matches declared type (basic check)
                if let Some(inferred) = infer_expression_type(value, ctx) {
                    if !types_compatible(th, &inferred) {
                        baba.error(
                            "TYPE_MISMATCH",
                            &format!(
                                "Type mismatch: variable '{}' declared as '{:?}' but assigned '{:?}'",
                                name, th, inferred
                            ),
                            file,
                            span.line,
                            span.column,
                        );
                    }
                }
            }
        }

        Statement::Assignment {
            target,
            value,
            span,
        } => {
            check_expression(value, ctx, baba, file, span);
            check_assign_target(target, ctx, baba, file, span);

            if let ifa_types::ast::AssignTarget::Variable(name) = target {
                // Check type compatibility for static types
                if let Some(declared_type) = ctx.get_var_type(name) {
                    if let Some(inferred_type) = infer_expression_type(value, ctx) {
                        if !types_compatible(declared_type, &inferred_type) {
                            baba.error(
                                "TYPE_MISMATCH",
                                &format!(
                                    "Type mismatch: variable '{}' is type '{:?}' but assigned '{:?}'",
                                    name, declared_type, inferred_type
                                ),
                                file,
                                span.line,
                                span.column,
                            );
                        }
                    }
                }

                // Check visibility
                if let Some(visibility) = ctx.get_var_visibility(name) {
                    let target_domain = ctx.get_var_domain(name).as_ref().and_then(|d| d.as_ref());
                    if !ctx.is_accessible(visibility, target_domain) {
                        baba.error(
                            "VISIBILITY_VIOLATION",
                            &format!("Èèwọ̀: Cannot access private variable '{}' from outside its domain", name),
                            file,
                            span.line,
                            span.column,
                        );
                    }
                }

                // H1: Re-assignment revives a moved variable. After `x = new_value;`
                // the binding is live again regardless of prior move state.
                ctx.move_tracker.revive(name);
            }
        }

        Statement::Update {
            target,
            value,
            span,
            ..
        } => {
            if let Some(v) = value {
                check_expression(v, ctx, baba, file, span);
            }
            check_assign_target(target, ctx, baba, file, span);

            if let ifa_types::ast::AssignTarget::Variable(name) = target {
                // Read-and-write: if name was moved, it is use-after-move!
                if let Some(result) = ctx.move_tracker.check_use(name) {
                    match result {
                        MoveCheckResult::UseAfterMove { moved_at_line, moved_at_col, .. } => {
                            baba.error(
                                "USE_AFTER_MOVE",
                                &format!(
                                    "Variable '{}' used in update after being moved (moved at {}:{})",
                                    name, moved_at_line, moved_at_col
                                ),
                                file,
                                span.line,
                                span.column,
                            );
                        }
                        MoveCheckResult::MaybeUseAfterMove { moved_at_line, moved_at_col, .. } => {
                            baba.warning(
                                "MAYBE_USE_AFTER_MOVE",
                                &format!(
                                    "Variable '{}' may have been moved on a prior branch before update (moved at {}:{})",
                                    name, moved_at_line, moved_at_col
                                ),
                                file,
                                span.line,
                                span.column,
                            );
                        }
                    }
                }

                // Check visibility
                if let Some(visibility) = ctx.get_var_visibility(name) {
                    let target_domain = ctx.get_var_domain(name).as_ref().and_then(|d| d.as_ref());
                    if !ctx.is_accessible(visibility, target_domain) {
                        baba.error(
                            "VISIBILITY_VIOLATION",
                            &format!("Èèwọ̀: Cannot access private variable '{}' from outside its domain", name),
                            file,
                            span.line,
                            span.column,
                        );
                    }
                }

                // Revive it now that it has a new value
                ctx.move_tracker.revive(name);
            }
        }

        Statement::Const { value, span, .. } => {
            check_expression(value, ctx, baba, file, span);
        }

        Statement::Instruction { call, span } => {
            check_unsafe_ffi_call(call, baba, file, span);

            // Check for division by zero
            if (call.method == "pin" || call.method == "div")
                && let Some(Expression::Int(0)) = call.args.get(1)
            {
                baba.error(
                    "DIVISION_BY_ZERO",
                    "Division by zero detected",
                    file,
                    span.line,
                    span.column,
                );
            }

            // #opon kekere + async domain call warning
            if ctx.opon_size.as_deref() == Some("kekere") {
                let domain_name = format!("{:?}", call.domain).to_lowercase();
                if domain_name == "osa" || call.method.contains("async") {
                    baba.warning(
                        "OPON_KEKERE_ASYNC",
                        &format!(
                            "#opon kekere (64 call frames) used with async domain call '{}.{}' — consider #opon arinrin or larger",
                            domain_name, call.method
                        ),
                        file,
                        span.line,
                        span.column,
                    );
                }
            }

            // Track resource lifecycle
            let domain = format!("{:?}", call.domain).to_lowercase();
            if call.method == "si" || call.method == "open" {
                ctx.open_resources.insert(
                    format!("{}:{}", domain, span.line),
                    (span.line, span.column),
                );
            }
            if call.method == "pa" || call.method == "close" {
                ctx.open_resources
                    .remove(&format!("{}:{}", domain, span.line));
            }

            // Check taboo violations - get current context (caller) from function or "global"
            let caller = ctx
                .current_function
                .clone()
                .unwrap_or_else(|| "global".to_string());
            let callee = format!("{:?}", call.domain).to_lowercase();
            ctx.taboo_enforcer
                .check_call(&caller, &callee, span.line, span.column);

            // Check arguments
            for arg in &call.args {
                check_expression(arg, ctx, baba, file, span);
            }
        }

        Statement::EseDef {
            name,
            params,
            body,
            span,
            visibility: _,
            is_async,
        } => {
            // Register params as used (they are implicitly used by the caller)
            for param in params {
                ctx.use_var(&param.name);
            }

            ctx.enter_function(name, *is_async);

            for s in body {
                check_statement(s, ctx, baba, file);
            }

            // Check for missing return (only warn, not error)
            if !ctx.has_return && !body.is_empty() {
                // Only warn if function seems to return something
                if function_should_return(body) {
                    baba.warning(
                        "MISSING_RETURN",
                        &format!("Function '{}' may not return on all paths", name),
                        file,
                        span.line,
                        span.column,
                    );
                }
            }

            ctx.exit_function();
        }

        Statement::OduDef { name, body, .. } => {
            ctx.enter_domain(name);
            for s in body {
                check_statement(s, ctx, baba, file);
            }
            ctx.exit_domain();
        }

        Statement::If {
            condition,
            then_body,
            else_body,
            span,
        } => {
            check_expression(condition, ctx, baba, file, span);

            // H1: Snapshot move tracker before each branch, then merge.
            // A move on only one branch produces MaybeMoved; on both produces Moved.
            let pre_if_snapshot = ctx.move_tracker.snapshot();

            // Walk then-branch on a clone so its moves don't contaminate else-branch.
            let mut then_tracker = pre_if_snapshot.clone();
            std::mem::swap(&mut ctx.move_tracker, &mut then_tracker);
            for s in then_body {
                check_statement(s, ctx, baba, file);
            }
            // Recover the then-branch final state.
            std::mem::swap(&mut ctx.move_tracker, &mut then_tracker);

            let mut else_tracker = pre_if_snapshot.clone();
            if let Some(else_stmts) = else_body {
                std::mem::swap(&mut ctx.move_tracker, &mut else_tracker);
                for s in else_stmts {
                    check_statement(s, ctx, baba, file);
                }
                std::mem::swap(&mut ctx.move_tracker, &mut else_tracker);
            }

            // Merge branch outcomes back into context.
            let merged = crate::movement::MoveTracker::merge_branches(&then_tracker, &else_tracker);
            ctx.move_tracker.apply(&merged);
        }

        Statement::While {
            condition,
            body,
            span,
        } => {
            check_expression(condition, ctx, baba, file, span);

            for s in body {
                check_statement(s, ctx, baba, file);
            }
        }

        Statement::For {
            var,
            iterable,
            body,
            span,
        } => {
            check_expression(iterable, ctx, baba, file, span);
            ctx.use_var(var);

            // Iterable validation: warn if the expression is statically known
            // to not be a collection type.
            if let Some(inferred) = infer_expression_type(iterable, ctx) {
                let is_iterable = matches!(
                    inferred,
                    TypeHint::List | TypeHint::Map | TypeHint::Str | TypeHint::Array { .. }
                );
                if !is_iterable {
                    baba.warning(
                        "NON_ITERABLE",
                        &format!(
                            "For loop iterates over '{:?}', which is not a collection type. Expected List, Map, Str, or Array.",
                            inferred
                        ),
                        file,
                        span.line,
                        span.column,
                    );
                }
            }

            for s in body {
                check_statement(s, ctx, baba, file);
            }
        }

        Statement::Return { value, span } => {
            ctx.has_return = true;
            if let Some(v) = value {
                check_expression(v, ctx, baba, file, span);
            }
        }

        Statement::Ebo { offering: _offering, body: None, .. } => {
            // Ebo without body: semantic directive, no checks needed
        }

        Statement::Ebo { offering: _offering, body: Some(body), span } => {
            // Ebo with body: scoped memory epoch — warn if return/break/continue
            // could bypass epoch cleanup
            for stmt in body {
                match stmt {
                    Statement::Return { .. } => {
                        baba.warning(
                            "EBO_RETURN",
                            "return inside ẹbọ epoch will release epoch memory before returning",
                            file,
                            span.line,
                            span.column,
                        );
                    }
                    _ => {}
                }
            }
            for s in body {
                check_statement(s, ctx, baba, file);
            }
        }

        Statement::Defer { body, span } => {
            // Deferred cleanup — check body for forbidden control flow
            for stmt in body {
                match stmt {
                    Statement::Return { .. } => {
                        baba.warning(
                            "DEFER_RETURN",
                            "return inside defer block will run deferred cleanup first, then return the function",
                            file,
                            span.line,
                            span.column,
                        );
                    }
                    Statement::Break { .. } => {
                        baba.warning(
                            "DEFER_BREAK",
                            "break inside defer block has no effect on the deferred cleanup scope",
                            file,
                            span.line,
                            span.column,
                        );
                    }
                    Statement::Continue { .. } => {
                        baba.warning(
                            "DEFER_CONTINUE",
                            "continue inside defer block has no effect on the deferred cleanup scope",
                            file,
                            span.line,
                            span.column,
                        );
                    }
                    _ => {}
                }
            }
            for s in body {
                check_statement(s, ctx, baba, file);
            }
        }

        Statement::Ailewu { body, span } => {
            // Enter ailewu (unsafe) context
            let was_in_ailewu = ctx.in_ailewu;
            ctx.in_ailewu = true;

            // Warn about entering unsafe code
            baba.warning(
                "AILEWU_BLOCK",
                "Entering ailewu (unsafe) block - low-level operations enabled",
                file,
                span.line,
                span.column,
            );

            // Check body
            for s in body {
                check_statement(s, ctx, baba, file);
            }

            // Restore previous context
            ctx.in_ailewu = was_in_ailewu;
        }

        Statement::Expr { expr, span } => {
            check_expression(expr, ctx, baba, file, span);
        }

        _ => {}
    }
}

use ifa_types::ast::{Span, AssignTarget};

fn check_assign_target(
    target: &AssignTarget,
    ctx: &mut LintContext,
    baba: &mut Babalawo,
    file: &str,
    span: &Span,
) {
    match target {
        AssignTarget::Variable(name) => {
            if !ctx.defined_vars.contains_key(name) {
                baba.error(
                    "UNDEFINED_VARIABLE",
                    &format!("Variable '{}' assigned before declaration", name),
                    file,
                    span.line,
                    span.column,
                );
            } else if ctx.in_parallel_body && !ctx.parallel_locals.contains(name) {
                baba.error(
                    "PARALLEL_MUTATION",
                    &format!("Cannot mutate captured variable '{}' inside parallel body", name),
                    file,
                    span.line,
                    span.column,
                );
            }
        }
        AssignTarget::Index { name, index } => {
            check_expression(index, ctx, baba, file, span);
            if !ctx.defined_vars.contains_key(name) {
                baba.error(
                    "UNDEFINED_VARIABLE",
                    &format!("Variable '{}' used before declaration", name),
                    file,
                    span.line,
                    span.column,
                );
            } else if ctx.in_parallel_body && !ctx.parallel_locals.contains(name) {
                baba.error(
                    "PARALLEL_MUTATION",
                    &format!("Cannot mutate captured variable '{}' inside parallel body", name),
                    file,
                    span.line,
                    span.column,
                );
            }
        }
        AssignTarget::Dereference(expr) => {
            check_expression(expr, ctx, baba, file, span);
        }
    }
}

/// Check an expression for issues
fn check_expression(
    expr: &Expression,
    ctx: &mut LintContext,
    baba: &mut Babalawo,
    file: &str,
    span: &Span,
) {
    match expr {
        Expression::Identifier(name) => {
            ctx.use_var(name);

            // H1: Check for use-after-move before any other checks.
            if let Some(result) = ctx.move_tracker.check_use(name) {
                match result {
                    MoveCheckResult::UseAfterMove { moved_at_line, moved_at_col, .. } => {
                        baba.error(
                            "USE_AFTER_MOVE",
                            &format!(
                                "Variable '{}' used after being moved (moved at {}:{})",
                                name, moved_at_line, moved_at_col
                            ),
                            file,
                            span.line,
                            span.column,
                        );
                    }
                    MoveCheckResult::MaybeUseAfterMove { moved_at_line, moved_at_col, .. } => {
                        baba.warning(
                            "MAYBE_USE_AFTER_MOVE",
                            &format!(
                                "Variable '{}' may have been moved on a prior branch (moved at {}:{})",
                                name, moved_at_line, moved_at_col
                            ),
                            file,
                            span.line,
                            span.column,
                        );
                    }
                }
            }

            // Check if variable is defined
            if !ctx.defined_vars.contains_key(name) && !is_builtin(name) {
                baba.error(
                    "UNDEFINED_VARIABLE",
                    &format!("Variable '{}' used before declaration", name),
                    file,
                    span.line,
                    span.column,
                );
            } else if !is_builtin(name) {
                // Check visibility
                if let Some(visibility) = ctx.get_var_visibility(name) {
                    let target_domain = ctx.get_var_domain(name).as_ref().and_then(|d| d.as_ref());
                    if !ctx.is_accessible(visibility, target_domain) {
                        baba.error(
                            "VISIBILITY_VIOLATION",
                            &format!("Èèwọ̀: Cannot access private symbol '{}' from outside its domain", name),
                            file,
                            span.line,
                            span.column,
                        );
                    }
                }
            }
        }

        Expression::BinaryOp {
            left, right, op, ..
        } => {
            check_expression(left, ctx, baba, file, span);
            check_expression(right, ctx, baba, file, span);

            // Check for division by zero in binary op
            if matches!(
                op,
                ifa_types::ast::BinaryOperator::Div | ifa_types::ast::BinaryOperator::Mod
            ) && let Expression::Int(0) = **right
            {
                baba.error(
                    "DIVISION_BY_ZERO",
                    "Division by zero in expression",
                    file,
                    span.line,
                    span.column,
                );
            }
        }

        Expression::List(items) => {
            for item in items {
                check_expression(item, ctx, baba, file, span);
            }
        }

        Expression::Map(entries) => {
            for (k, v) in entries {
                check_expression(k, ctx, baba, file, span);
                check_expression(v, ctx, baba, file, span);
            }
        }

        Expression::Index { object, index, .. } => {
            check_expression(object, ctx, baba, file, span);
            check_expression(index, ctx, baba, file, span);
        }

        Expression::MethodCall { object, args, .. } => {
            check_expression(object, ctx, baba, file, span);
            for arg in args {
                check_expression(arg, ctx, baba, file, span);
            }
        }

        Expression::OduCall(call) => {
            check_unsafe_ffi_call(call, baba, file, span);

            // H1: Record actor-boundary moves (Osa domain calls move non-copy args).
            for (moved_name, line, col) in move_args_from_odu_call(call) {
                // Guard: warn if borrowed at point of move.
                if ctx.iwa_engine.is_borrowed(moved_name) {
                    baba.error(
                        "MOVE_WHILE_BORROWED",
                        &format!(
                            "Cannot move '{}' into actor boundary while it is borrowed",
                            moved_name
                        ),
                        file,
                        line,
                        col,
                    );
                } else {
                    ctx.move_tracker.record_move(moved_name, line, col);
                }
            }

            // H4: Parallel-For Gate
            if ctx.in_parallel_body && call.domain.has_side_effects() {
                baba.error(
                    "PARALLEL_SIDE_EFFECT",
                    &format!("Cannot call side-effecting domain '{}' inside parallel body", call.domain.yoruba_name()),
                    file,
                    span.line,
                    span.column,
                );
            }

            let is_parallel_for = call.domain == ifa_types::OduDomain::Iwori && call.method == "yipo.ori";
            if is_parallel_for {
                ctx.in_parallel_body = true;
            }

            for arg in &call.args {
                check_expression(arg, ctx, baba, file, span);
            }

            if is_parallel_for {
                ctx.in_parallel_body = false;
                ctx.parallel_locals.clear();
            }
        }

        Expression::Await(inner) => {
            // §ASYNC_SAFETY: reti (await) is only valid inside a daro (async) function.
            if !ctx.in_async_function {
                baba.error(
                    "AWAIT_OUTSIDE_ASYNC",
                    "'reti' (await) used outside an async function. Declare function with 'daro ese' to use await.",
                    file,
                    span.line,
                    span.column,
                );
            }

            // H3: daro Async Enforcement
            // Enforce that &mut borrows do not cross daro suspension points.
            for (var_name, var_type) in &ctx.var_types {
                if let TypeHint::RefMut(_) = var_type {
                    baba.error(
                        "MUTABLE_BORROW_ACROSS_DARO",
                        &format!("Mutable borrow '{}' cannot be held across an await suspension point", var_name),
                        file,
                        span.line,
                        span.column,
                    );
                }
            }

            check_expression(inner, ctx, baba, file, span);
        }

        Expression::Get { object, .. } => {
            check_expression(object, ctx, baba, file, span);
        }

        Expression::Call { name, args } => {
            ctx.use_var(name);
            
            // Check visibility
            if let Some(visibility) = ctx.get_var_visibility(name) {
                let target_domain = ctx.get_var_domain(name).as_ref().and_then(|d| d.as_ref());
                if !ctx.is_accessible(visibility, target_domain) {
                    baba.error(
                        "VISIBILITY_VIOLATION",
                        &format!("Èèwọ̀: Cannot call private function '{}' from outside its domain", name),
                        file,
                        span.line,
                        span.column,
                    );
                }
            }

            for arg in args {
                check_expression(arg, ctx, baba, file, span);
            }
        }

        Expression::InterpolatedString { parts } => {
            for part in parts {
                if let ifa_types::ast::InterpolatedPart::Expression(expr) = part {
                    check_expression(expr, ctx, baba, file, span);
                }
            }
        }

        Expression::UnaryOp { expr, .. } => {
            check_expression(expr, ctx, baba, file, span);
        }

        Expression::Lambda { params, body } => {
            for param in params {
                // If in parallel body, lambda params are parallel locals
                if ctx.in_parallel_body {
                    ctx.parallel_locals.insert(param.clone());
                }
                ctx.define_var(param, span.clone(), Visibility::Private);
            }
            for stmt in body {
                check_statement(stmt, ctx, baba, file);
            }
        }

        _ => {}
    }
}

fn check_unsafe_ffi_call(call: &ifa_types::ast::OduCall, baba: &mut Babalawo, file: &str, span: &Span) {
    if call.domain == ifa_types::OduDomain::Coop
        && (call.method.eq_ignore_ascii_case("itumo")
            || call.method.eq_ignore_ascii_case("summon"))
    {
        baba.error(
            "TABOO_UNSAFE_FFI",
            "ffi.itumo() requires explicit sanctification; hidden bridges are forbidden",
            file,
            span.line,
            span.column,
        );
    }
}

/// Check for unused variables
fn check_unused_vars(ctx: &LintContext, baba: &mut Babalawo, file: &str) {
    for (var, span) in &ctx.defined_vars {
        if !ctx.used_vars.contains(var) && !var.starts_with('_') {
            baba.add_full(
                Severity::Warning,
                "UNUSED_VARIABLE",
                &format!("Variable '{}' is defined but never used", var),
                file,
                span.clone(),
            );
        }
    }
}

/// Check for unclosed resources
fn check_unclosed_resources(ctx: &LintContext, baba: &mut Babalawo, file: &str) {
    for (resource, (line, col)) in &ctx.open_resources {
        baba.warning(
            "UNCLOSED_RESOURCE",
            &format!("Resource '{}' opened but never closed", resource),
            file,
            *line,
            *col,
        );
    }
}

/// Check if an expression uses a variable
fn expression_uses_var(expr: &Expression, var_name: &str) -> bool {
    match expr {
        Expression::Identifier(name) => name == var_name,
        Expression::BinaryOp { left, right, .. } => {
            expression_uses_var(left, var_name) || expression_uses_var(right, var_name)
        }
        Expression::List(items) => items.iter().any(|i| expression_uses_var(i, var_name)),
        Expression::Index { object, index, .. } => {
            expression_uses_var(object, var_name) || expression_uses_var(index, var_name)
        }
        _ => false,
    }
}

/// Check if a function body suggests it should return something
fn function_should_return(body: &[Statement]) -> bool {
    // Simple heuristic: if there's any expression statement, it probably should return
    for stmt in body {
        if matches!(stmt, Statement::Return { value: Some(_), .. }) {
            return true;
        }
    }
    false
}

/// Infer the type of an expression (returns None for dynamic/unknown types)
fn infer_expression_type(expr: &Expression, ctx: &LintContext) -> Option<TypeHint> {
    match expr {
        Expression::Int(_) => Some(TypeHint::Int),
        Expression::Float(_) => Some(TypeHint::Float),
        Expression::String(_) => Some(TypeHint::Str),
        Expression::Bool(_) => Some(TypeHint::Bool),
        Expression::Nil => None, // Nil is compatible with any type
        Expression::List(_) => Some(TypeHint::List),
        Expression::Map(_) => Some(TypeHint::Map),

        Expression::Identifier(name) => {
            // Look up variable type in context
            ctx.get_var_type(name).cloned()
        }

        Expression::BinaryOp {
            left, right, op: _, ..
        } => {
            let left_type = infer_expression_type(left, ctx)?;
            let right_type = infer_expression_type(right, ctx)?;

            // Basic inference rules
            if types_compatible(&left_type, &right_type)
                || types_compatible(&right_type, &left_type)
            {
                // If one is float and other is int, result is float (usually)
                // If both same, result is same.
                // Simplified:
                if matches!(left_type, TypeHint::Float | TypeHint::F32 | TypeHint::F64)
                    || matches!(right_type, TypeHint::Float | TypeHint::F32 | TypeHint::F64)
                {
                    // Return the float one
                    if matches!(left_type, TypeHint::Float | TypeHint::F32 | TypeHint::F64) {
                        Some(left_type)
                    } else {
                        Some(right_type)
                    }
                } else {
                    // Assume left type dominates (e.g. i32 + i32 -> i32)
                    Some(left_type)
                }
            } else {
                None // Incompatible types in binary op
            }
        }

        _ => None, // Cannot infer type for complex expressions
    }
}

/// Check if two types are compatible for assignment
fn types_compatible(declared: &TypeHint, inferred: &TypeHint) -> bool {
    // Dynamic types are compatible with each other
    if matches!(declared, TypeHint::Any) {
        return true;
    }

    // Exact match
    if declared == inferred {
        return true;
    }

    // Int/Float compatibility with sized versions
    match (declared, inferred) {
        // Dynamic Int is compatible with any integer literal
        (TypeHint::Int, TypeHint::Int) => true,
        (TypeHint::I64, TypeHint::Int) => true,
        (TypeHint::I32, TypeHint::Int) => true,
        (TypeHint::I16, TypeHint::Int) => true,
        (TypeHint::I8, TypeHint::Int) => true,

        // Dynamic Float is compatible with float literal
        (TypeHint::Float, TypeHint::Float) => true,
        (TypeHint::F64, TypeHint::Float) => true,
        (TypeHint::F32, TypeHint::Float) => true,

        // Allow Int -> Float promotion
        (TypeHint::Float, TypeHint::Int) => true,
        (TypeHint::F64, TypeHint::Int) => true,
        (TypeHint::F32, TypeHint::Int) => true,

        _ => false,
    }
}

/// Check if a name is a builtin
fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "true" | "false" | "nil" | "otito" | "iro" | "ohunkohun"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ifa_parser::parse;

    #[test]
    fn test_undefined_variable() {
        let src = "Irosu.fo(x);";
        if let Ok(program) = parse(src) {
            let baba = check_program(&program, "test.ifa");
            assert!(baba.has_errors());
        }
    }

    #[test]
    fn test_unused_variable() {
        let src = "ayanmo x = 42;";
        if let Ok(program) = parse(src) {
            let baba = check_program(&program, "test.ifa");
            assert!(baba.warning_count() > 0);
        }
    }

    // §AWAIT_OUTSIDE_ASYNC: reti in a non-async function must trigger AWAIT_OUTSIDE_ASYNC
    #[test]
    fn test_await_outside_async_errors() {
        let src = r#"
            ese sync_fn() {
                ayanmo result = reti Osa.ise("task");
                pada result;
            }
        "#;
        if let Ok(program) = parse(src) {
            let baba = check_program(&program, "test.ifa");
            let has_await_error = baba.diagnostics.iter().any(|d| d.error.code == "AWAIT_OUTSIDE_ASYNC");
            assert!(has_await_error, "Expected AWAIT_OUTSIDE_ASYNC error but got: {:?}", baba.diagnostics);
        }
    }

    // §AWAIT_OUTSIDE_ASYNC: reti inside a daro (async) function must be clean
    #[test]
    fn test_await_inside_async_is_clean() {
        let src = r#"
            daro ese async_fn() {
                ayanmo result = reti Osa.ise("task");
                pada result;
            }
        "#;
        if let Ok(program) = parse(src) {
            let baba = check_program(&program, "test.ifa");
            let has_await_error = baba.diagnostics.iter().any(|d| d.error.code == "AWAIT_OUTSIDE_ASYNC");
            assert!(!has_await_error, "Unexpected AWAIT_OUTSIDE_ASYNC in async function");
        }
    }

    // §NON_ITERABLE: For loop over a typed Int variable must warn NON_ITERABLE
    #[test]
    fn test_for_loop_over_non_iterable_warns() {
        let src = r#"
            ese bad_loop() {
                ayanmo n: Int = 5;
                fun x ninu n {
                    Irosu.ko(x);
                }
            }
        "#;
        if let Ok(program) = parse(src) {
            let baba = check_program(&program, "test.ifa");
            let has_warn = baba.diagnostics.iter().any(|d| d.error.code == "NON_ITERABLE");
            assert!(has_warn, "Expected NON_ITERABLE warning but got: {:?}", baba.diagnostics);
        }
    }

    // §NON_ITERABLE: For loop over a typed List must be clean
    #[test]
    fn test_for_loop_over_list_is_clean() {
        let src = r#"
            ese good_loop() {
                ayanmo items: List = [1, 2, 3];
                fun x ninu items {
                    Irosu.ko(x);
                }
            }
        "#;
        if let Ok(program) = parse(src) {
            let baba = check_program(&program, "test.ifa");
            let has_warn = baba.diagnostics.iter().any(|d| d.error.code == "NON_ITERABLE");
            assert!(!has_warn, "Unexpected NON_ITERABLE on a List variable");
        }
    }

    #[test]
    fn test_private_member_access_fails() {
        let src = r#"
            odu SecretHouse {
                ikoko ayanmo key: Int = 123;
                
                gbangba ese getKey() { 
                    pada key; 
                }
            }
            
            ese bad_access() {
                ayanmo stolen = key;
            }
        "#;
        if let Ok(program) = parse(src) {
            let baba = check_program(&program, "test.ifa");
            let has_error = baba.diagnostics.iter().any(|d| d.error.code == "VISIBILITY_VIOLATION");
            assert!(has_error, "Expected VISIBILITY_VIOLATION error but got: {:?}", baba.diagnostics);
        }
    }

    #[test]
    fn test_public_member_access_passes() {
        let src = r#"
            odu OpenHouse {
                gbangba ayanmo key: Int = 123;
            }
            
            ese good_access() {
                ayanmo found = key;
            }
        "#;
        if let Ok(program) = parse(src) {
            let baba = check_program(&program, "test.ifa");
            let has_error = baba.diagnostics.iter().any(|d| d.error.code == "VISIBILITY_VIOLATION");
            assert!(!has_error, "Unexpected VISIBILITY_VIOLATION on public member");
        }
    }

    #[test]
    fn test_internal_access_passes() {
        let src = r#"
            odu MyHouse {
                ikoko ayanmo key: Int = 123;
                
                gbangba ese getKey() { 
                    pada key; 
                }
            }
        "#;
        if let Ok(program) = parse(src) {
            let baba = check_program(&program, "test.ifa");
            let has_error = baba.diagnostics.iter().any(|d| d.error.code == "VISIBILITY_VIOLATION");
            assert!(!has_error, "Unexpected VISIBILITY_VIOLATION on internal member access");
        }
    }

    // §H1: USE_AFTER_MOVE — passing a list to Osa and then reading it must error.
    // NOTE: This test validates the move-tracker data structures directly since
    // the parser does not yet emit Osa.ise() calls that the checker can observe.
    #[test]
    fn test_move_tracker_use_after_move() {
        let mut tracker = crate::movement::MoveTracker::new();
        tracker.declare("payload");
        tracker.record_move("payload", 5, 3);
        let result = tracker.check_use("payload");
        assert!(
            matches!(result, Some(crate::movement::MoveCheckResult::UseAfterMove { .. })),
            "Expected USE_AFTER_MOVE for moved variable"
        );
    }

    // §H1: MAYBE_USE_AFTER_MOVE — merged branch where only one branch moves.
    #[test]
    fn test_move_tracker_maybe_move_on_branch() {
        let mut then_t = crate::movement::MoveTracker::new();
        then_t.declare("data");
        then_t.record_move("data", 10, 1);

        let else_t = crate::movement::MoveTracker::new(); // data alive

        let merged = crate::movement::MoveTracker::merge_branches(&then_t, &else_t);
        assert!(
            matches!(merged.check_use("data"), Some(crate::movement::MoveCheckResult::MaybeUseAfterMove { .. })),
            "Expected MAYBE_USE_AFTER_MOVE after divergent branch"
        );
    }
}
