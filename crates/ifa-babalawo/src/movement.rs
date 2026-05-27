//! # H1 — Move Tracker
//!
//! Lightweight linear-type enforcement for Ifá-Lang.
//!
//! ## Model
//! A variable is *moved* when its ownership transfers to another binding or
//! crosses an actor/async boundary. After a move the original name is
//! *dead* — any subsequent read or re-move triggers `USE_AFTER_MOVE`.
//!
//! ## Triggers (what counts as a move)
//! 1. **Actor send** — passing an identifier as an argument to `Osa.*`
//!    (the concurrency domain). Data crossing actor boundaries MUST be moved.
//! 2. **Async spawn** — same rule applies to any `daro` call argument that
//!    is a bare identifier (not a copy-eligible primitive literal).
//! 3. **Explicit `move(x)` expression** — not yet in the parser; reserved
//!    as a future language keyword. We pre-wire the infra here.
//! 4. **Move-assignment** — `ayanmo y = x;` where x is NOT copy-eligible
//!    (i.e. not Int/Float/Bool/Nil literal). This is conservative: if the
//!    RHS *could* be a heap value, it is treated as a move.
//!
//! ## Copy-eligible types
//! Int, Float, Bool, Nil, and their sized variants are always copied, never
//! moved. Strings, Lists, Maps, and custom objects are move-by-default.
//!
//! ## Branch merge rule
//! If a variable is moved on ANY branch (then OR else), it is considered
//! potentially moved after the `if` statement. A subsequent use emits a
//! `MAYBE_USE_AFTER_MOVE` warning (not an error) because we cannot prove
//! which branch runs at compile time.

use ifa_types::OduDomain;
use ifa_types::ast::{Expression, OduCall, TypeHint};
use std::collections::{HashMap, HashSet};

/// Per-variable move state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveState {
    /// Variable is alive; has not been moved.
    Alive,
    /// Variable has been definitively moved (error on use).
    Moved { line: usize, col: usize },
    /// Variable may have been moved on some branch (warning on use).
    MaybeMoved { line: usize, col: usize },
}

/// H1 move tracker, designed to slot into `LintContext`.
#[derive(Debug, Default, Clone)]
pub struct MoveTracker {
    /// Current move state per variable name.
    state: HashMap<String, MoveState>,
}

impl MoveTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Declare a new variable — it starts `Alive`.
    pub fn declare(&mut self, name: &str) {
        self.state.insert(name.to_string(), MoveState::Alive);
    }

    /// Record a definitive move of `name` at the given location.
    pub fn record_move(&mut self, name: &str, line: usize, col: usize) {
        self.state
            .insert(name.to_string(), MoveState::Moved { line, col });
    }

    /// Check a use of `name`. Returns an error description if the variable
    /// has been moved.
    pub fn check_use(&self, name: &str) -> Option<MoveCheckResult> {
        match self.state.get(name) {
            Some(MoveState::Moved { line, col }) => Some(MoveCheckResult::UseAfterMove {
                var: name.to_string(),
                moved_at_line: *line,
                moved_at_col: *col,
            }),
            Some(MoveState::MaybeMoved { line, col }) => Some(MoveCheckResult::MaybeUseAfterMove {
                var: name.to_string(),
                moved_at_line: *line,
                moved_at_col: *col,
            }),
            _ => None,
        }
    }

    /// Merge two divergent branch states (conservative union).
    /// After an `if/else`, a variable moved on ANY branch becomes `MaybeMoved`.
    /// If moved on BOTH branches it becomes `Moved`.
    pub fn merge_branches(then_tracker: &Self, else_tracker: &Self) -> Self {
        let mut merged = HashMap::new();
        let all_keys: HashSet<&String> = then_tracker
            .state
            .keys()
            .chain(else_tracker.state.keys())
            .collect();

        for key in all_keys {
            let then_state = then_tracker.state.get(key).unwrap_or(&MoveState::Alive);
            let else_state = else_tracker.state.get(key).unwrap_or(&MoveState::Alive);

            let new_state = match (then_state, else_state) {
                (MoveState::Moved { line, col }, MoveState::Moved { .. }) => MoveState::Moved {
                    line: *line,
                    col: *col,
                },
                (MoveState::Moved { line, col }, _) | (_, MoveState::Moved { line, col }) => {
                    MoveState::MaybeMoved {
                        line: *line,
                        col: *col,
                    }
                }
                (MoveState::MaybeMoved { line, col }, _)
                | (_, MoveState::MaybeMoved { line, col }) => MoveState::MaybeMoved {
                    line: *line,
                    col: *col,
                },
                _ => MoveState::Alive,
            };
            merged.insert(key.clone(), new_state);
        }

        Self { state: merged }
    }

    /// Apply states from another tracker into self (used after merge).
    pub fn apply(&mut self, other: &Self) {
        for (k, v) in &other.state {
            self.state.insert(k.clone(), v.clone());
        }
    }

    /// Clone the current tracker snapshot for branch analysis.
    pub fn snapshot(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }

    /// Revive a moved variable (e.g. after re-assignment: `x = new_value`).
    pub fn revive(&mut self, name: &str) {
        self.state.insert(name.to_string(), MoveState::Alive);
    }

    /// Check if a variable is currently moved.
    pub fn is_moved(&self, name: &str) -> bool {
        matches!(
            self.state.get(name),
            Some(MoveState::Moved { .. }) | Some(MoveState::MaybeMoved { .. })
        )
    }
}

/// Result of a use-check.
#[derive(Debug)]
pub enum MoveCheckResult {
    UseAfterMove {
        var: String,
        moved_at_line: usize,
        moved_at_col: usize,
    },
    MaybeUseAfterMove {
        var: String,
        moved_at_line: usize,
        moved_at_col: usize,
    },
}

/// Returns true if an expression is *copy-eligible* and therefore does NOT
/// trigger move semantics when passed as an argument.
///
/// Scalars (Int, Float, Bool, Nil) are always copied. Everything else — Str,
/// List, Map, custom objects — is moved on transfer to an actor boundary.
pub fn is_copy_eligible(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::Int(_) | Expression::Float(_) | Expression::Bool(_) | Expression::Nil
    )
}

/// Returns true if an expression is a bare identifier (a potential move source).
pub fn as_identifier(expr: &Expression) -> Option<&str> {
    if let Expression::Identifier(name) = expr {
        Some(name.as_str())
    } else {
        None
    }
}

/// Determine which arguments to an OduCall are "move triggers" — i.e. the call
/// crosses a concurrency boundary and the argument must be consumed.
///
/// Currently: any call into the Osa (concurrency) domain is a move boundary.
/// All non-copy-eligible identifier arguments are considered moved.
pub fn move_args_from_odu_call<'a>(
    call: &'a OduCall,
) -> impl Iterator<Item = (&'a str, usize, usize)> {
    let is_actor_boundary = call.domain == OduDomain::Osa;
    call.args.iter().enumerate().filter_map(move |(_, arg)| {
        if is_actor_boundary && !is_copy_eligible(arg) {
            as_identifier(arg).map(|name| (name, call.span.line, call.span.column))
        } else {
            None
        }
    })
}

/// Check an expression for uses of moved variables, reporting any violations.
/// Returns a list of `MoveCheckResult` violations found.
pub fn check_expr_for_moved_uses<'a>(
    expr: &'a Expression,
    tracker: &MoveTracker,
) -> Vec<MoveCheckResult> {
    let mut violations = Vec::new();
    collect_expr_violations(expr, tracker, &mut violations);
    violations
}

fn collect_expr_violations(
    expr: &Expression,
    tracker: &MoveTracker,
    out: &mut Vec<MoveCheckResult>,
) {
    match expr {
        Expression::Identifier(name) => {
            if let Some(v) = tracker.check_use(name) {
                out.push(v);
            }
        }
        Expression::BinaryOp { left, right, .. } => {
            collect_expr_violations(left, tracker, out);
            collect_expr_violations(right, tracker, out);
        }
        Expression::UnaryOp { expr, .. } => collect_expr_violations(expr, tracker, out),
        Expression::OduCall(call) => {
            for arg in &call.args {
                collect_expr_violations(arg, tracker, out);
            }
        }
        Expression::Call { args, .. } => {
            for arg in args {
                collect_expr_violations(arg, tracker, out);
            }
        }
        Expression::MethodCall { object, args, .. } => {
            collect_expr_violations(object, tracker, out);
            for arg in args {
                collect_expr_violations(arg, tracker, out);
            }
        }
        Expression::Get { object, .. } => collect_expr_violations(object, tracker, out),
        Expression::Index { object, index, .. } => {
            collect_expr_violations(object, tracker, out);
            collect_expr_violations(index, tracker, out);
        }
        Expression::List(items) => {
            for item in items {
                collect_expr_violations(item, tracker, out);
            }
        }
        Expression::Map(entries) => {
            for (k, v) in entries {
                collect_expr_violations(k, tracker, out);
                collect_expr_violations(v, tracker, out);
            }
        }
        Expression::Await(inner) => collect_expr_violations(inner, tracker, out),
        Expression::Try(inner) => collect_expr_violations(inner, tracker, out),
        Expression::InterpolatedString { parts } => {
            for part in parts {
                if let ifa_types::ast::InterpolatedPart::Expression(e) = part {
                    collect_expr_violations(e, tracker, out);
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_var_is_alive() {
        let mut t = MoveTracker::new();
        t.declare("x");
        assert!(t.check_use("x").is_none());
    }

    #[test]
    fn use_after_move_detected() {
        let mut t = MoveTracker::new();
        t.declare("x");
        t.record_move("x", 5, 1);
        assert!(matches!(
            t.check_use("x"),
            Some(MoveCheckResult::UseAfterMove { .. })
        ));
    }

    #[test]
    fn revive_clears_move() {
        let mut t = MoveTracker::new();
        t.declare("x");
        t.record_move("x", 5, 1);
        t.revive("x");
        assert!(t.check_use("x").is_none());
    }

    #[test]
    fn branch_merge_both_moved_is_definitive() {
        let mut then_t = MoveTracker::new();
        then_t.declare("x");
        then_t.record_move("x", 3, 1);

        let mut else_t = MoveTracker::new();
        else_t.declare("x");
        else_t.record_move("x", 7, 1);

        let merged = MoveTracker::merge_branches(&then_t, &else_t);
        assert!(matches!(
            merged.check_use("x"),
            Some(MoveCheckResult::UseAfterMove { .. })
        ));
    }

    #[test]
    fn branch_merge_one_moved_is_maybe() {
        let mut then_t = MoveTracker::new();
        then_t.declare("x");
        then_t.record_move("x", 3, 1);

        let else_t = MoveTracker::new(); // x is alive in else branch

        let merged = MoveTracker::merge_branches(&then_t, &else_t);
        assert!(matches!(
            merged.check_use("x"),
            Some(MoveCheckResult::MaybeUseAfterMove { .. })
        ));
    }

    #[test]
    fn copy_eligible_not_moved() {
        assert!(is_copy_eligible(&Expression::Int(42)));
        assert!(is_copy_eligible(&Expression::Float(1.5)));
        assert!(is_copy_eligible(&Expression::Bool(true)));
        assert!(is_copy_eligible(&Expression::Nil));
        assert!(!is_copy_eligible(&Expression::String("hello".into())));
        assert!(!is_copy_eligible(&Expression::Identifier("x".into())));
    }
}
