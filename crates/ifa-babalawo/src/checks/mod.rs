pub mod expressions;
pub mod resources;
pub mod statements;
pub mod types;

pub(crate) use expressions::*;
pub(crate) use resources::*;
pub(crate) use statements::*;
pub(crate) use types::*;

// # Compile-Time Checks
//
// Static analysis checks for Ifá-Lang programs.
// Ported from legacy/src/linter.py and legacy/src/validator.py

use crate::Severity;
use crate::diagnose::Babalawo;
use crate::effects::EffectChecker;
use crate::iwa::IwaEngine;
use crate::movement::{MoveCheckResult, MoveTracker};
use crate::taboo::TabooEnforcer;
use ifa_types::ast::{ExprKind, Expression, Program, Statement, TypeHint, Visibility};
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
    /// Whether strict mode is active (abo; directive)
    pub is_strict: bool,
    /// Effect checker — enforces side-effect boundaries
    pub effect_checker: EffectChecker,
    /// Aliases defined in the program
    pub aliases: HashMap<String, Box<Expression>>,
    /// Target environment for feature validation
    pub target: Target,
    /// Registered Iwa definitions (Protocol/Trait)
    pub iwa_defs: HashMap<String, ifa_types::ast::IwaDef>,
    /// Variables that are strictly isolated (iso capability)
    pub iso_vars: HashSet<String>,
    /// Whether the current control flow path has diverged (return, break, continue)
    pub is_divergent: bool,
}

impl Default for LintContext {
    fn default() -> Self {
        Self::new(Target::default())
    }
}

impl LintContext {
    pub fn new(target: Target) -> Self {
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
            is_strict: false,
            effect_checker: EffectChecker::new(),
            aliases: HashMap::new(),
            target,
            iwa_defs: HashMap::new(),
            iso_vars: HashSet::new(),
            is_divergent: false,
        }
    }

    pub fn define_var(&mut self, name: &str, span: ifa_types::ast::Span, visibility: Visibility) {
        self.defined_vars.insert(name.to_string(), span);
        self.var_visibility.insert(name.to_string(), visibility);
        self.var_domain
            .insert(name.to_string(), self.current_domain.clone());
        self.move_tracker.declare(name);
    }

    /// Define a variable with a type hint
    pub fn define_var_typed(
        &mut self,
        name: &str,
        type_hint: TypeHint,
        span: ifa_types::ast::Span,
        visibility: Visibility,
    ) {
        self.defined_vars.insert(name.to_string(), span);
        self.var_types.insert(name.to_string(), type_hint);
        self.var_visibility.insert(name.to_string(), visibility);
        self.var_domain
            .insert(name.to_string(), self.current_domain.clone());
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

use crate::target_check::{check_expression_target, check_statement_target};
use ifa_types::target::Target;

/// Configuration for the Babalawo linter
#[derive(Debug, Clone)]
pub struct BabalawoConfig {
    /// Include wisdom/proverbs in diagnostics (slower)
    pub include_wisdom: bool,
    /// Custom taboos (caller, callee) to enforce during checking
    pub taboos: Vec<(String, String)>,
    /// Target environment for capability checking
    pub target: Target,
}

impl Default for BabalawoConfig {
    fn default() -> Self {
        Self {
            include_wisdom: true,
            taboos: Vec::new(),
            target: Target::default(),
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
    let mut ctx = LintContext::new(config.target);

    // Register taboos from config
    for (source, target) in &config.taboos {
        ctx.taboo_enforcer.add_taboo(source, "", target, "", false);
    }

    // First pass: collect definitions
    for stmt in &program.statements {
        collect_definitions(stmt, &mut ctx);
    }
    babalawo.is_strict = ctx.is_strict;

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

/// Collect variable and function definitions + Taboos and Opon directives
fn collect_definitions(stmt: &Statement, ctx: &mut LintContext) {
    match stmt {
        Statement::VarDecl {
            name,
            type_hint,
            span,
            visibility,
            ..
        } => {
            if let Some(th) = type_hint {
                ctx.define_var_typed(name, th.clone(), span.clone(), *visibility);
            } else {
                ctx.define_var(name, span.clone(), *visibility);
            }
        }
        Statement::Const {
            name,
            value: _,
            visibility,
            span,
        } => {
            ctx.define_var(name, span.clone(), *visibility);
        }
        Statement::Alias { name, target, span } => {
            ctx.aliases.insert(name.clone(), target.clone());
            // An alias also reserves a local name
            ctx.define_var(name, span.clone(), Visibility::Private);
        }
        Statement::EseDef {
            name,
            params,
            body,
            span,
            visibility,
            effects: _,
            return_type: _,
            is_iranti: _,
        } => {
            ctx.define_var(name, span.clone(), *visibility);
            // Parameters are also definitions within the function (private by default)
            for param in params {
                if let Some(th) = &param.type_hint {
                    ctx.define_var_typed(
                        &param.name,
                        th.clone(),
                        span.clone(),
                        Visibility::Private,
                    ); // Simplification: param uses Ese span
                } else {
                    ctx.define_var(&param.name, span.clone(), Visibility::Private);
                }
            }
            for s in body {
                collect_definitions(s, ctx);
            }
        }
        Statement::OduDef {
            name,
            body,
            span,
            visibility,
        } => {
            ctx.define_var(name, span.clone(), *visibility);
            for s in body {
                collect_definitions(s, ctx);
            }
        }
        Statement::For {
            var,
            iterable: _,
            body,
            span,
        } => {
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
        // Strict mode directive
        Statement::Abo { .. } => {
            ctx.is_strict = true;
        }
        Statement::Import { path, names, span } => {
            if let Some(n) = names {
                for name in n {
                    ctx.define_var(name, span.clone(), Visibility::Private);
                }
            } else if let Some(last) = path.last() {
                ctx.define_var(last, span.clone(), Visibility::Private);
            }
            ctx.imports.insert(path.join("."));
        }
        _ => {}
    }
}

use ifa_types::ast::{AssignTarget, Span};

/// Check if a name is a builtin
fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "true" | "false" | "nil" | "otito" | "iro" | "ohunkohun"
    )
}

#[allow(clippy::needless_range_loop)]
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    let mut dp = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        dp[i][0] = i;
    }
    for j in 0..=len2 {
        dp[0][j] = j;
    }

    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            if c1 == c2 {
                dp[i + 1][j + 1] = dp[i][j];
            } else {
                dp[i + 1][j + 1] = std::cmp::min(
                    dp[i][j] + 1,
                    std::cmp::min(dp[i][j + 1] + 1, dp[i + 1][j] + 1),
                );
            }
        }
    }

    dp[len1][len2]
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
    fn test_undefined_variable_did_you_mean() {
        let src = r#"
            ayanmo my_special_variable = 42;
            Irosu.fo(my_specal_variable);
        "#;
        if let Ok(program) = parse(src) {
            let baba = check_program(&program, "test.ifa");
            assert!(baba.has_errors());
            let error_msg = &baba.diagnostics[0].error.message;
            assert!(
                error_msg.contains("Did you mean 'my_special_variable'?"),
                "Expected suggestion in error message: {}",
                error_msg
            );
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
            let has_await_error = baba
                .diagnostics
                .iter()
                .any(|d| d.error.code == "AWAIT_OUTSIDE_ASYNC");
            assert!(
                has_await_error,
                "Expected AWAIT_OUTSIDE_ASYNC error but got: {:?}",
                baba.diagnostics
            );
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
            let has_await_error = baba
                .diagnostics
                .iter()
                .any(|d| d.error.code == "AWAIT_OUTSIDE_ASYNC");
            assert!(
                !has_await_error,
                "Unexpected AWAIT_OUTSIDE_ASYNC in async function"
            );
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
            let has_warn = baba
                .diagnostics
                .iter()
                .any(|d| d.error.code == "NON_ITERABLE");
            assert!(
                has_warn,
                "Expected NON_ITERABLE warning but got: {:?}",
                baba.diagnostics
            );
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
            let has_warn = baba
                .diagnostics
                .iter()
                .any(|d| d.error.code == "NON_ITERABLE");
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
            let has_error = baba
                .diagnostics
                .iter()
                .any(|d| d.error.code == "VISIBILITY_VIOLATION");
            assert!(
                has_error,
                "Expected VISIBILITY_VIOLATION error but got: {:?}",
                baba.diagnostics
            );
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
            let has_error = baba
                .diagnostics
                .iter()
                .any(|d| d.error.code == "VISIBILITY_VIOLATION");
            assert!(
                !has_error,
                "Unexpected VISIBILITY_VIOLATION on public member"
            );
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
            let has_error = baba
                .diagnostics
                .iter()
                .any(|d| d.error.code == "VISIBILITY_VIOLATION");
            assert!(
                !has_error,
                "Unexpected VISIBILITY_VIOLATION on internal member access"
            );
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
            matches!(
                result,
                Some(crate::movement::MoveCheckResult::UseAfterMove { .. })
            ),
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
            matches!(
                merged.check_use("data"),
                Some(crate::movement::MoveCheckResult::MaybeUseAfterMove { .. })
            ),
            "Expected MAYBE_USE_AFTER_MOVE after divergent branch"
        );
    }
}
