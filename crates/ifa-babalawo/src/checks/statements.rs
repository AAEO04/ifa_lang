use super::*;
use ifa_types::ast::*;

/// Check a statement for issues
pub(crate) fn check_statement(
    stmt: &Statement,
    ctx: &mut LintContext,
    baba: &mut Babalawo,
    file: &str,
) {
    check_statement_target(stmt, &ctx.target, baba, file);

    match stmt {
        Statement::Ori { .. } => {}
        Statement::Opon { .. } => {}
        Statement::VarDecl {
            name,
            value,
            span,
            type_hint,
            ..
        } => {
            check_expression(&value, ctx, baba, file, span);

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
                if let Some(inferred) = infer_expression_type(&value, ctx)
                    && !types_compatible(th, &inferred)
                {
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

                check_iwa_compliance(th, value, ctx, baba, file, span);
            }

            // Track iso variable
            if is_iso_expression(value, ctx) {
                ctx.iso_vars.insert(name.clone());
            } else {
                ctx.iso_vars.remove(name);
            }
        }

        Statement::Assignment {
            target,
            value,
            span,
        } => {
            check_expression(&value, ctx, baba, file, span);
            check_assign_target(target, ctx, baba, file, span);

            if let ifa_types::ast::AssignTarget::Variable(name) = target {
                // Check type compatibility for static types
                if let Some(declared_type) = ctx.get_var_type(name) {
                    if let Some(inferred_type) = infer_expression_type(&value, ctx)
                        && !types_compatible(declared_type, &inferred_type)
                    {
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
                    check_iwa_compliance(declared_type, value, ctx, baba, file, span);
                }

                // Check visibility
                if let Some(visibility) = ctx.get_var_visibility(name) {
                    let target_domain = ctx.get_var_domain(name).as_ref().and_then(|d| d.as_ref());
                    if !ctx.is_accessible(visibility, target_domain) {
                        baba.error(
                            "VISIBILITY_VIOLATION",
                            &format!(
                                "Èèwọ̀: Cannot access private variable '{}' from outside its domain",
                                name
                            ),
                            file,
                            span.line,
                            span.column,
                        );
                    }
                }

                if ctx.iwa_engine.is_borrowed(name) {
                    baba.error(
                        "BORROW_ERROR",
                        &format!("Cannot mutate '{}' because it is borrowed", name),
                        file,
                        span.line,
                        span.column,
                    );
                }

                // Revive it now that it has a new value
                ctx.move_tracker.revive(name);

                if is_iso_expression(value, ctx) {
                    ctx.iso_vars.insert(name.clone());
                } else {
                    ctx.iso_vars.remove(name);
                }
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
                if ctx.iwa_engine.is_borrowed(name) {
                    baba.error(
                        "BORROW_ERROR",
                        &format!("Cannot mutate '{}' because it is borrowed", name),
                        file,
                        span.line,
                        span.column,
                    );
                }

                if let Some(result) = ctx.move_tracker.check_use(name) {
                    match result {
                        MoveCheckResult::UseAfterMove {
                            moved_at_line,
                            moved_at_col,
                            ..
                        } => {
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
                        MoveCheckResult::MaybeUseAfterMove {
                            moved_at_line,
                            moved_at_col,
                            ..
                        } => {
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
                            &format!(
                                "Èèwọ̀: Cannot access private variable '{}' from outside its domain",
                                name
                            ),
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
            check_expression(&value, ctx, baba, file, span);
        }

        Statement::Alias { target, span, .. } => {
            check_expression(&target, ctx, baba, file, span);
        }

        Statement::Instruction { call, span } => {
            let tmp =
                ifa_types::ast::Expression::new(ExprKind::OduCall(call.clone()), span.clone());
            check_expression(&tmp, ctx, baba, file, span);
        }

        Statement::EseDef {
            name,
            params,
            body,
            span,
            visibility: _,
            effects,
            return_type: _,
            is_iranti: _,
        } => {
            // Register params as used (they are implicitly used by the caller)
            let mut seen_optional = false;
            for param in params {
                ctx.use_var(&param.name);
                if param.default_value.is_some() {
                    seen_optional = true;
                } else if seen_optional {
                    baba.error(
                        "INVALID_PARAM_ORDER",
                        &format!(
                            "Required parameter '{}' cannot follow optional parameters",
                            param.name
                        ),
                        file,
                        span.line,
                        span.column,
                    );
                }
            }

            let is_async = effects.contains(&ifa_types::ast::Effect::Async);
            ctx.enter_function(name, is_async);
            ctx.effect_checker.enter_function(effects.clone());

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

            for err in &ctx.effect_checker.errors {
                baba.error(
                    "EFFECT_VIOLATION",
                    &err.to_string(),
                    file,
                    span.line,
                    span.column,
                );
            }
            ctx.effect_checker.errors.clear();

            ctx.effect_checker.leave_function();
            ctx.exit_function();
        }

        Statement::OduDef { name, body, .. } => {
            ctx.enter_domain(name);
            for s in body {
                check_statement(s, ctx, baba, file);
            }
            ctx.exit_domain();
        }

        Statement::IwaDef(def) => {
            // Register Iwa definition
            ctx.iwa_defs.insert(def.name.clone(), def.clone());
            // Inside Iwa, parameters and types can be checked for validity
        }

        Statement::If {
            condition,
            then_body,
            else_body,
            span,
        } => {
            check_expression(&condition, ctx, baba, file, span);

            // H1: Snapshot move tracker before each branch, then merge.
            // A move on only one branch produces MaybeMoved; on both produces Moved.
            let pre_if_move = ctx.move_tracker.snapshot();
            let pre_if_iwa = ctx.iwa_engine.snapshot();
            let pre_if_div = ctx.is_divergent;

            // Then branch
            let mut then_move = pre_if_move.clone();
            let mut then_iwa = pre_if_iwa.clone();
            std::mem::swap(&mut ctx.move_tracker, &mut then_move);
            std::mem::swap(&mut ctx.iwa_engine, &mut then_iwa);
            ctx.is_divergent = false;

            ctx.iwa_engine.enter_scope();
            for s in then_body {
                check_statement(s, ctx, baba, file);
            }
            ctx.iwa_engine.exit_scope();

            let then_div = ctx.is_divergent;
            std::mem::swap(&mut ctx.move_tracker, &mut then_move);
            std::mem::swap(&mut ctx.iwa_engine, &mut then_iwa);

            // Else branch
            let mut else_move = pre_if_move.clone();
            let mut else_iwa = pre_if_iwa.clone();
            let mut else_div = false;

            if let Some(else_stmts) = else_body {
                std::mem::swap(&mut ctx.move_tracker, &mut else_move);
                std::mem::swap(&mut ctx.iwa_engine, &mut else_iwa);
                ctx.is_divergent = false;

                ctx.iwa_engine.enter_scope();
                for s in else_stmts {
                    check_statement(s, ctx, baba, file);
                }
                ctx.iwa_engine.exit_scope();

                else_div = ctx.is_divergent;
                std::mem::swap(&mut ctx.move_tracker, &mut else_move);
                std::mem::swap(&mut ctx.iwa_engine, &mut else_iwa);
            }

            // CFG Merge
            if then_div && else_div {
                // Both branches diverge, code after if is unreachable
                ctx.move_tracker.apply(&then_move);
                ctx.iwa_engine.apply(&then_iwa);
                ctx.is_divergent = true;
            } else if then_div {
                // Then diverges, fallthrough is only Else
                ctx.move_tracker.apply(&else_move);
                ctx.iwa_engine.apply(&else_iwa);
                ctx.is_divergent = pre_if_div;
            } else if else_div {
                // Else diverges, fallthrough is only Then
                ctx.move_tracker.apply(&then_move);
                ctx.iwa_engine.apply(&then_iwa);
                ctx.is_divergent = pre_if_div;
            } else {
                // Neither diverges, merge normally
                let merged_move =
                    crate::movement::MoveTracker::merge_branches(&then_move, &else_move);
                ctx.move_tracker.apply(&merged_move);

                let merged_iwa = crate::iwa::IwaEngine::merge_branches(&then_iwa, &else_iwa);
                ctx.iwa_engine.apply(&merged_iwa);

                ctx.is_divergent = pre_if_div;
            }
        }

        Statement::While {
            condition,
            body,
            span,
        } => {
            check_expression(&condition, ctx, baba, file, span);

            for s in body {
                check_statement(s, ctx, baba, file);
            }
        }

        Statement::Match {
            condition,
            arms,
            span,
        } => {
            check_expression(&condition, ctx, baba, file, span);

            let mut has_wildcard = false;
            for arm in arms {
                if matches!(arm.pattern, ifa_types::ast::MatchPattern::Wildcard) {
                    has_wildcard = true;
                }
                for s in &arm.body {
                    check_statement(s, ctx, baba, file);
                }
            }

            if !has_wildcard {
                baba.warning(
                    "NON_EXHAUSTIVE_MATCH",
                    "Match block may not be exhaustive. Consider adding a '_' wildcard arm.",
                    file,
                    span.line,
                    span.column,
                );
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
            ctx.is_divergent = true;
            if let Some(v) = value {
                check_expression(v, ctx, baba, file, span);
            }
        }

        Statement::Ebo {
            offering: _offering,
            body: None,
            ..
        } => {
            // Ebo without body: semantic directive, no checks needed
        }

        Statement::Ebo {
            offering: _offering,
            body: Some(body),
            span,
        } => {
            // Ebo with body: scoped memory epoch — warn if return/break/continue
            // could bypass epoch cleanup
            for stmt in body {
                if let Statement::Return { .. } = stmt {
                    baba.warning(
                        "EBO_RETURN",
                        "return inside ẹbọ epoch will release epoch memory before returning",
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
            check_expression(&expr, ctx, baba, file, span);
        }

        Statement::Throw { value, span } => {
            check_expression(&value, ctx, baba, file, span);
        }

        Statement::Ase { .. }
        | Statement::Abo { .. }
        | Statement::Taboo { .. }
        | Statement::Opon { .. }
        | Statement::Import { .. } => {
            // These are top-level declarations handled during scan_statement_for_defs
        }

        Statement::Ewo {
            condition,
            message: _,
            span,
        } => {
            check_expression(&condition, ctx, baba, file, span);
        }

        Statement::AssertType {
            value,
            type_hint: _,
            span,
        } => {
            check_expression(&value, ctx, baba, file, span);
        }

        Statement::Yield { duration, span } => {
            check_expression(duration, ctx, baba, file, span);
        }

        Statement::Try {
            try_body,
            catch_var: _,
            catch_body,
            finally_body,
            span: _,
        } => {
            for s in try_body {
                check_statement(s, ctx, baba, file);
            }
            for s in catch_body {
                check_statement(s, ctx, baba, file);
            }
            if let Some(fb) = finally_body {
                for s in fb {
                    check_statement(s, ctx, baba, file);
                }
            }
        }

        Statement::Break { .. } | Statement::Continue { .. } => {
            ctx.is_divergent = true;
            // Should be validated to be inside loops
        } // Catch-all removed. All statement types must be explicitly handled.
    }
}

pub(crate) fn check_assign_target(
    target: &AssignTarget,
    ctx: &mut LintContext,
    baba: &mut Babalawo,
    file: &str,
    span: &Span,
) {
    match target {
        AssignTarget::Variable(name) => {
            if !ctx.defined_vars.contains_key(AsRef::<str>::as_ref(&name)) {
                baba.error(
                    "UNDEFINED_VARIABLE",
                    &format!("Variable '{}' assigned before declaration", name),
                    file,
                    span.line,
                    span.column,
                );
            } else if ctx.in_parallel_body
                && !ctx.parallel_locals.contains(AsRef::<str>::as_ref(&name))
            {
                baba.error(
                    "PARALLEL_MUTATION",
                    &format!(
                        "Cannot mutate captured variable '{}' inside parallel body",
                        name
                    ),
                    file,
                    span.line,
                    span.column,
                );
            }
        }
        AssignTarget::Index { name, index } => {
            check_expression(&index, ctx, baba, file, span);
            if !ctx.defined_vars.contains_key(AsRef::<str>::as_ref(&name)) {
                baba.error(
                    "UNDEFINED_VARIABLE",
                    &format!("Variable '{}' used before declaration", name),
                    file,
                    span.line,
                    span.column,
                );
            } else if ctx.in_parallel_body
                && !ctx.parallel_locals.contains(AsRef::<str>::as_ref(&name))
            {
                baba.error(
                    "PARALLEL_MUTATION",
                    &format!(
                        "Cannot mutate captured variable '{}' inside parallel body",
                        name
                    ),
                    file,
                    span.line,
                    span.column,
                );
            }
        }
        AssignTarget::Dereference(expr) => {
            check_expression(&expr, ctx, baba, file, span);
        }
    }
}

/// Check if a function body suggests it should return something
pub(crate) fn function_should_return(body: &[Statement]) -> bool {
    // Simple heuristic: if there's any expression statement, it probably should return
    for stmt in body {
        if matches!(stmt, Statement::Return { value: Some(_), .. }) {
            return true;
        }
    }
    false
}

/// Check if an expression evaluates to an iso capability
pub(crate) fn is_iso_expression(expr: &Expression, ctx: &LintContext) -> bool {
    match &expr.kind {
        ExprKind::Iso(_) => true,
        ExprKind::MoveExpr(inner) => {
            if let ExprKind::Identifier(name) = &inner.kind {
                ctx.iso_vars.contains(AsRef::<str>::as_ref(&name))
            } else {
                is_iso_expression(inner, ctx)
            }
        }
        ExprKind::Identifier(name) => {
            // Technically a direct identifier read doesn't move it unless it's explicitly moved via yanda/MoveExpr
            // But if it's assigned: `y = x` we are copying the pointer.
            // Wait, for iso, implicit copy/alias is forbidden!
            // So if `x` is iso, `y = x` should be a compile error!
            // We handle this in check_expression!
            ctx.iso_vars.contains(AsRef::<str>::as_ref(&name))
        }
        _ => false,
    }
}
