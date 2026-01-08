//! # Compile-Time Checks
//!
//! Static analysis checks for Ifá-Lang programs.
//! Ported from legacy/src/linter.py and legacy/src/validator.py

use crate::diagnose::Babalawo;
use crate::iwa::IwaEngine;
use crate::taboo::TabooEnforcer;
use ifa_core::ast::{Program, Statement, Expression};
use std::collections::{HashSet, HashMap};

/// Context for linting - tracks state as we walk the AST
#[derive(Debug)]
pub struct LintContext {
    /// Variables that have been defined
    pub defined_vars: HashSet<String>,
    /// Variables that have been used
    pub used_vars: HashSet<String>,
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
}

impl Default for LintContext {
    fn default() -> Self {
        Self::new()
    }
}

impl LintContext {
    pub fn new() -> Self {
        Self {
            defined_vars: HashSet::new(),
            used_vars: HashSet::new(),
            imports: HashSet::new(),
            current_function: None,
            has_return: false,
            open_resources: HashMap::new(),
            iwa_engine: IwaEngine::new(true),
            taboo_enforcer: TabooEnforcer::new(),
        }
    }
    
    pub fn define_var(&mut self, name: &str) {
        self.defined_vars.insert(name.to_string());
    }
    
    pub fn use_var(&mut self, name: &str) {
        self.used_vars.insert(name.to_string());
    }
    
    pub fn enter_function(&mut self, name: &str) {
        self.current_function = Some(name.to_string());
        self.has_return = false;
    }
    
    pub fn exit_function(&mut self) {
        self.current_function = None;
        self.has_return = false;
    }
}

/// Check a program and return diagnostics
pub fn check_program(program: &Program, filename: &str) -> Babalawo {
    let mut babalawo = Babalawo::new();
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
                &format!("Resource '{}' opened at line {} was never closed (needs '{}')",
                    debt.opener, debt.line, debt.required),
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
                &format!("Forbidden dependency: '{}' cannot call '{}'", v.caller, v.callee),
                filename,
                v.line,
                v.column,
            );
        }
    }
    
    babalawo
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
                &format!("Resource '{}' was never closed (needs '{}')", debt.opener, debt.required),
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
        Statement::VarDecl { name, .. } => {
            ctx.define_var(name);
        }
        Statement::EseDef { name, params, body, .. } => {
            ctx.define_var(name);
            // Parameters are also definitions within the function
            for param in params {
                ctx.define_var(&param.name);
            }
            for s in body {
                collect_definitions(s, ctx);
            }
        }
        Statement::OduDef { name, body, .. } => {
            ctx.define_var(name);
            for s in body {
                collect_definitions(s, ctx);
            }
        }
        Statement::For { var, body, .. } => {
            ctx.define_var(var);
            for s in body {
                collect_definitions(s, ctx);
            }
        }
        Statement::If { then_body, else_body, .. } => {
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
        // Opon directives - currently informational only
        Statement::Opon { size, .. } => {
            // Could store opon size in context for memory limit checks
            let _ = size;
        }
        _ => {}
    }
}


/// Check a statement for issues
fn check_statement(stmt: &Statement, ctx: &mut LintContext, baba: &mut Babalawo, file: &str) {
    match stmt {
        Statement::VarDecl { name, value, span, .. } => {
            check_expression(value, ctx, baba, file);
            
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
        }
        
        Statement::Assignment { target, value, span } => {
            check_expression(value, ctx, baba, file);
            
            // Check if target variable is defined
            if let ifa_core::ast::AssignTarget::Variable(name) = target {
                if !ctx.defined_vars.contains(name) {
                    baba.error(
                        "UNDEFINED_VARIABLE",
                        &format!("Variable '{}' assigned before declaration", name),
                        file,
                        span.line,
                        span.column,
                    );
                }
            }
        }
        
        Statement::Instruction { call, span } => {
            // Check for division by zero
            if call.method == "pin" || call.method == "div" {
                if let Some(Expression::Int(0)) = call.args.get(1) {
                    baba.error(
                        "DIVISION_BY_ZERO",
                        "Division by zero detected",
                        file,
                        span.line,
                        span.column,
                    );
                }
            }
            
            // Track resource lifecycle
            let domain = format!("{:?}", call.domain).to_lowercase();
            if call.method == "si" || call.method == "open" {
                ctx.open_resources.insert(format!("{}:{}", domain, span.line), (span.line, span.column));
            }
            if call.method == "pa" || call.method == "close" {
                ctx.open_resources.remove(&format!("{}:{}", domain, span.line));
            }
            
            // Check taboo violations - get current context (caller) from function or "global"
            let caller = ctx.current_function.clone().unwrap_or_else(|| "global".to_string());
            let callee = format!("{:?}", call.domain).to_lowercase();
            ctx.taboo_enforcer.check_call(&caller, &callee, span.line, span.column);
            
            // Check arguments
            for arg in &call.args {
                check_expression(arg, ctx, baba, file);
            }
        }
        
        Statement::EseDef { name, body, span, visibility: _, .. } => {
            ctx.enter_function(name);
            
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
        
        Statement::OduDef { body, .. } => {
            for s in body {
                check_statement(s, ctx, baba, file);
            }
        }
        
        Statement::If { condition, then_body, else_body, .. } => {
            check_expression(condition, ctx, baba, file);
            
            for s in then_body {
                check_statement(s, ctx, baba, file);
            }
            
            if let Some(else_stmts) = else_body {
                for s in else_stmts {
                    check_statement(s, ctx, baba, file);
                }
            }
        }
        
        Statement::While { condition, body, .. } => {
            check_expression(condition, ctx, baba, file);
            
            for s in body {
                check_statement(s, ctx, baba, file);
            }
        }
        
        Statement::For { var, iterable, body, span: _ } => {
            check_expression(iterable, ctx, baba, file);
            ctx.use_var(var);
            
            for s in body {
                check_statement(s, ctx, baba, file);
            }
        }
        
        Statement::Return { value, .. } => {
            ctx.has_return = true;
            if let Some(v) = value {
                check_expression(v, ctx, baba, file);
            }
        }
        
        _ => {}
    }
}

/// Check an expression for issues
fn check_expression(expr: &Expression, ctx: &mut LintContext, baba: &mut Babalawo, file: &str) {
    match expr {
        Expression::Identifier(name) => {
            ctx.use_var(name);
            
            // Check if variable is defined
            if !ctx.defined_vars.contains(name) && !is_builtin(name) {
                baba.error(
                    "UNDEFINED_VARIABLE",
                    &format!("Variable '{}' used before declaration", name),
                    file,
                    1, // TODO: get proper line from expression
                    1,
                );
            }
        }
        
        Expression::BinaryOp { left, right, op, .. } => {
            check_expression(left, ctx, baba, file);
            check_expression(right, ctx, baba, file);
            
            // Check for division by zero in binary op
            if matches!(op, ifa_core::ast::BinaryOperator::Div | ifa_core::ast::BinaryOperator::Mod) {
                if let Expression::Int(0) = **right {
                    baba.error(
                        "DIVISION_BY_ZERO",
                        "Division by zero in expression",
                        file,
                        1,
                        1,
                    );
                }
            }
        }
        
        Expression::List(items) => {
            for item in items {
                check_expression(item, ctx, baba, file);
            }
        }
        
        Expression::Map(entries) => {
            for (k, v) in entries {
                check_expression(k, ctx, baba, file);
                check_expression(v, ctx, baba, file);
            }
        }
        
        Expression::Index { object, index } => {
            check_expression(object, ctx, baba, file);
            check_expression(index, ctx, baba, file);
        }
        
        Expression::MethodCall { object, args, .. } => {
            check_expression(object, ctx, baba, file);
            for arg in args {
                check_expression(arg, ctx, baba, file);
            }
        }
        
        Expression::OduCall(call) => {
            for arg in &call.args {
                check_expression(arg, ctx, baba, file);
            }
        }
        
        _ => {}
    }
}

/// Check for unused variables
fn check_unused_vars(ctx: &LintContext, baba: &mut Babalawo, file: &str) {
    for var in &ctx.defined_vars {
        if !ctx.used_vars.contains(var) && !var.starts_with('_') {
            baba.warning(
                "UNUSED_VARIABLE",
                &format!("Variable '{}' is defined but never used", var),
                file,
                1,
                1,
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
        Expression::Index { object, index } => {
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

/// Check if a name is a builtin
fn is_builtin(name: &str) -> bool {
    matches!(name, "true" | "false" | "nil" | "otito" | "iro" | "ohunkohun")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ifa_core::parser::parse;
    
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
}
