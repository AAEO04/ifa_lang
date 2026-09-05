use super::*;
use ifa_types::ast::*;

/// Check an expression for issues
pub(crate) fn check_expression(
    expr: &Expression,
    ctx: &mut LintContext,
    baba: &mut Babalawo,
    file: &str,
    span: &Span,
) {
    check_expression_target(expr, &ctx.target, baba, file, span);

    match &expr.kind {
        ExprKind::Identifier(name) => {
            if ctx.iwa_engine.is_mutably_borrowed(name) {
                baba.error(
                    "BORROW_ERROR",
                    &format!("Cannot read '{}' because it is mutably borrowed", name),
                    file,
                    span.line,
                    span.column,
                );
            }
            ctx.use_var(name);

            // H1: Check for use-after-move before any other checks.
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
                                "Variable '{}' used after being moved (moved at {}:{})",
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
            if !ctx.defined_vars.contains_key(AsRef::<str>::as_ref(&name)) && !is_builtin(name) {
                let mut best_suggestion = None;
                let mut best_distance = usize::MAX;
                for existing_name in ctx.defined_vars.keys() {
                    let dist = levenshtein_distance(name, existing_name);
                    if dist < best_distance {
                        best_distance = dist;
                        best_suggestion = Some(existing_name.as_str());
                    }
                }

                let msg = if let Some(suggestion) = best_suggestion {
                    if best_distance <= 3 && best_distance <= name.len() / 2 {
                        format!(
                            "Variable '{}' used before declaration. Did you mean '{}'?",
                            name, suggestion
                        )
                    } else {
                        format!("Variable '{}' used before declaration", name)
                    }
                } else {
                    format!("Variable '{}' used before declaration", name)
                };

                baba.error("UNDEFINED_VARIABLE", &msg, file, span.line, span.column);
            } else if !is_builtin(name) {
                // Check visibility
                if let Some(visibility) = ctx.get_var_visibility(name) {
                    let target_domain = ctx.get_var_domain(name).as_ref().and_then(|d| d.as_ref());
                    if !ctx.is_accessible(visibility, target_domain) {
                        baba.error(
                            "VISIBILITY_VIOLATION",
                            &format!(
                                "Èèwọ̀: Cannot access private symbol '{}' from outside its domain",
                                name
                            ),
                            file,
                            span.line,
                            span.column,
                        );
                    }
                }
            }
        }

        ExprKind::BinaryOp {
            left, right, op, ..
        } => {
            check_expression(&left, ctx, baba, file, span);
            check_expression(&right, ctx, baba, file, span);

            // Check for division by zero in binary op
            if matches!(
                op,
                ifa_types::ast::BinaryOperator::Div | ifa_types::ast::BinaryOperator::Mod
            ) && let ExprKind::Int(0) = right.kind
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

        ExprKind::List(items) => {
            for item in items {
                check_expression(item, ctx, baba, file, span);
            }
        }

        ExprKind::Map(entries) => {
            for (k, v) in entries {
                check_expression(k, ctx, baba, file, span);
                check_expression(v, ctx, baba, file, span);
            }
        }

        ExprKind::Index { object, index, .. } => {
            check_expression(object, ctx, baba, file, span);
            check_expression(&index, ctx, baba, file, span);
        }

        ExprKind::Iso(inner) => {
            check_expression(inner, ctx, baba, file, span);
        }

        ExprKind::MoveExpr(inner) => {
            check_expression(inner, ctx, baba, file, span);
            let mut inner_peeled = &**inner;

            if let ExprKind::Identifier(name) = &inner_peeled.kind {
                if ctx.iwa_engine.is_borrowed(name) {
                    baba.error(
                        "MOVE_WHILE_BORROWED",
                        &format!("Cannot move '{}' while it is borrowed", name),
                        file,
                        span.line,
                        span.column,
                    );
                } else {
                    ctx.move_tracker.record_move(name, span.line, span.column);
                }
            }

            // H2: Enforce acyclic uniquely-owned data for yanda
            if let Some(inferred_type) = infer_expression_type(inner, ctx)
                && inferred_type.is_pointer_like()
            {
                baba.error(
                        "YANDA_SHARED_STATE",
                        "Cannot 'yanda' a shared pointer, reference, or channel. Only acyclic, uniquely-owned data can be transferred.",
                        file,
                        span.line,
                        span.column,
                    );
            }
        }

        ExprKind::MethodCall {
            object,
            method,
            args,
            ..
        } => {
            check_expression(object, ctx, baba, file, span);
            for arg in args {
                check_expression(&arg, ctx, baba, file, span);
            }

            let mut inner_obj = &**object;

            let var_name_opt = if let ExprKind::Identifier(n) = &inner_obj.kind {
                // Cross-module resolution for standard domains
                if ctx.imports.contains(n) && n.starts_with("std.") {
                    let domain_str = n.strip_prefix("std.").unwrap();
                    if let Ok(domain) = domain_str.parse::<ifa_types::domain::OduDomain>()
                        && let Some(_err_msg) = crate::metadata::validate_odu_call(&domain, method)
                    {
                        baba.error(
                            "UNKNOWN_MODULE_METHOD",
                            &format!("Èèwọ̀: Method '{}' does not exist in domain '{}'", method, n),
                            file,
                            span.line,
                            span.column,
                        );
                    }
                }
                Some(n.clone())
            } else {
                None
            };

            // Process yanda (move)
            if method == "yanda"
                && let Some(ref name) = var_name_opt
            {
                ctx.iwa_engine.resolve_debt_by_move(name);
                ctx.move_tracker.record_move(name, span.line, span.column);
            }

            // iwa_pele lifecycle tracking
            let obj_type = infer_expression_type(object, ctx);
            if let Some(TypeHint::Iwa(iwa_name)) = obj_type {
                let close_key = format!("{}.{}", iwa_name, method);

                // Try to resolve existing debt first
                if let Some(pos) = ctx.iwa_engine.debt_ledger.iter().position(|d| {
                    d.required == close_key
                        && (d.var_name == var_name_opt
                            || var_name_opt.is_none()
                            || d.var_name.is_none())
                }) {
                    ctx.iwa_engine.debt_ledger.remove(pos);
                }

                // Open new debt if the method has the attribute
                if let Some(iwa_def) = ctx.iwa_defs.get(&iwa_name)
                    && let Some(iwa_method) = iwa_def.methods.iter().find(|m| m.name == *method)
                {
                    for attr in &iwa_method.attributes {
                        if attr.starts_with("#[iwa_pele_pair(") {
                            let inner = attr
                                .trim_start_matches("#[iwa_pele_pair(")
                                .trim_end_matches(")]");
                            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                            if parts.len() == 2 {
                                let opener = parts[0];
                                let closer = parts[1];

                                let key = format!("{}.{}", iwa_name, method);
                                let req = format!("{}.{}", iwa_name, closer);

                                if method == opener {
                                    ctx.iwa_engine.debt_ledger.push(crate::iwa::ResourceDebt {
                                        var_name: var_name_opt.clone(),
                                        opener: key,
                                        required: req,
                                        line: span.line,
                                        column: span.column,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        ExprKind::OduCall(call) => {
            check_unsafe_ffi_call(call, baba, file, span);
            check_escape_hazards(call, ctx, baba, file, span);

            // Validate domain method
            if let Some(_err_msg) = crate::metadata::validate_odu_call(&call.domain, &call.method) {
                baba.error(
                    "UNKNOWN_MODULE_METHOD",
                    &format!(
                        "Èèwọ̀: Method '{}' does not exist in domain '{:?}'",
                        call.method, call.domain
                    ),
                    file,
                    span.line,
                    span.column,
                );
            }

            // Check for division by zero
            if (call.method == "pin" || call.method == "div")
                && matches!(call.args.get(1).map(|e| &e.kind), Some(ExprKind::Int(0)))
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

            // Effect system check
            let callee_effects = crate::effects::domain_effects(call.domain);
            ctx.effect_checker
                .check_call(&callee_effects, file, span.line, span.column);

            // H1: Enforce explicit actor-boundary moves (Osa domain requires explicit move).
            if call.domain == ifa_types::domain::OduDomain::Osa {
                for (idx, arg) in call.args.iter().enumerate() {
                    let is_payload = if call.method == "ran" || call.method == "post" {
                        idx == 1
                    } else {
                        true
                    };
                    if is_payload
                        && let ExprKind::Identifier(_) = &arg.kind
                        && !crate::movement::is_copy_eligible(arg)
                    {
                        baba.error(
                                    "EXPLICIT_MOVE_REQUIRED",
                                    "Cannot pass non-scalar variable to actor boundary. Use 'yanda' (or 'move') to explicitly transfer ownership.",
                                    file,
                                    call.span.line,
                                    call.span.column,
                                );
                    }
                }
            }

            // H4: Parallel-For Gate
            if ctx.in_parallel_body && call.domain.has_side_effects() {
                baba.error(
                    "PARALLEL_SIDE_EFFECT",
                    &format!(
                        "Cannot call side-effecting domain '{}' inside parallel body",
                        call.domain.yoruba_name()
                    ),
                    file,
                    span.line,
                    span.column,
                );
            }

            let is_parallel_for =
                call.domain == ifa_types::OduDomain::Iwori && call.method == "yipo.ori";
            if is_parallel_for {
                ctx.in_parallel_body = true;
            }

            for arg in &call.args {
                check_expression(&arg, ctx, baba, file, span);
            }

            if is_parallel_for {
                ctx.in_parallel_body = false;
                ctx.parallel_locals.clear();
            }
        }

        ExprKind::Await(inner) => {
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
                        &format!(
                            "Mutable borrow '{}' cannot be held across an await suspension point",
                            var_name
                        ),
                        file,
                        span.line,
                        span.column,
                    );
                }
            }

            check_expression(inner, ctx, baba, file, span);
        }

        ExprKind::Get { object, .. } => {
            check_expression(object, ctx, baba, file, span);
        }

        ExprKind::Call { name, args } => {
            ctx.use_var(name);

            // Check visibility
            if let Some(visibility) = ctx.get_var_visibility(name) {
                let target_domain = ctx.get_var_domain(name).as_ref().and_then(|d| d.as_ref());
                if !ctx.is_accessible(visibility, target_domain) {
                    baba.error(
                        "VISIBILITY_VIOLATION",
                        &format!(
                            "Èèwọ̀: Cannot call private function '{}' from outside its domain",
                            name
                        ),
                        file,
                        span.line,
                        span.column,
                    );
                }
            }

            for arg in args {
                check_expression(&arg, ctx, baba, file, span);
            }
        }

        ExprKind::InterpolatedString { parts } => {
            for part in parts {
                if let ifa_types::ast::InterpolatedPart::Expression(expr) = part {
                    check_expression(&expr, ctx, baba, file, span);
                }
            }
        }
        ExprKind::UnaryOp { expr, op } => {
            let mut is_address_of_id = false;
            let mut target_name = None;

            if matches!(
                op,
                ifa_types::ast::UnaryOperator::AddressOf
                    | ifa_types::ast::UnaryOperator::AddressOfMut
            ) {
                let mut inner = &**expr;

                if let ExprKind::Identifier(name) = &inner.kind {
                    is_address_of_id = true;
                    target_name = Some(name.clone());
                }
            }

            if !is_address_of_id {
                check_expression(&expr, ctx, baba, file, span);
            } else if let Some(name) = target_name {
                // We still need to check if it's undefined
                if !ctx.defined_vars.contains_key(AsRef::<str>::as_ref(&name)) && !is_builtin(&name)
                {
                    baba.error(
                        "UNDEFINED_VARIABLE",
                        &format!("Variable '{}' used before declaration", name),
                        file,
                        span.line,
                        span.column,
                    );
                }

                if ctx.iso_vars.contains(AsRef::<str>::as_ref(&name)) {
                    baba.error(
                        "ISO_ALIAS_HAZARD",
                        &format!("Cannot create reference to 'iso' variable '{}'", name),
                        file,
                        span.line,
                        span.column,
                    );
                } else {
                    if *op == ifa_types::ast::UnaryOperator::AddressOfMut {
                        if let Err(err) = ctx.iwa_engine.borrow_mut(&name, span.line, span.column) {
                            match err {
                                crate::iwa::BorrowError::AlreadyMutablyBorrowed {
                                    existing_line,
                                    ..
                                } => {
                                    baba.error(
                                        "BORROW_ERROR",
                                        &format!("Cannot mutably borrow '{}' because it is already mutably borrowed at line {}.\nState transition history:\n  - {} (line {})\n  - Borrowed (Mutable) (line {})", name, existing_line, name, span.line, existing_line),
                                        file,
                                        span.line,
                                        span.column,
                                    );
                                }
                                crate::iwa::BorrowError::ImmutableBorrowExists {
                                    existing_line,
                                    ..
                                } => {
                                    baba.error(
                                        "BORROW_ERROR",
                                        &format!("Cannot mutably borrow '{}' because it is already immutably borrowed at line {}.", name, existing_line),
                                        file,
                                        span.line,
                                        span.column,
                                    );
                                }
                                _ => {}
                            }
                        }
                    } else {
                        if let Err(crate::iwa::BorrowError::AlreadyMutablyBorrowed {
                            existing_line,
                            ..
                        }) = ctx.iwa_engine.borrow(&name, span.line, span.column)
                        {
                            baba.error(
                                "BORROW_ERROR",
                                &format!("Cannot borrow '{}' because it is already mutably borrowed at line {}.", name, existing_line),
                                file,
                                span.line,
                                span.column,
                            );
                        }
                    }
                }
            }
        }
        ExprKind::Lambda { params, body } => {
            for param in params {
                // If in parallel body, lambda params are parallel locals
                if ctx.in_parallel_body {
                    ctx.parallel_locals.insert(param.name.clone());
                }
                ctx.define_var(&param.name, span.clone(), Visibility::Private);
            }
            for stmt in body {
                check_statement(stmt, ctx, baba, file);
            }
        }

        _ => {}
    }
}

pub(crate) fn check_unsafe_ffi_call(
    call: &ifa_types::ast::OduCall,
    baba: &mut Babalawo,
    file: &str,
    span: &Span,
) {
    if call.domain == ifa_types::OduDomain::Coop
        && (call.method.eq_ignore_ascii_case("itumo") || call.method.eq_ignore_ascii_case("summon"))
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

pub(crate) fn check_escape_hazards(
    call: &ifa_types::ast::OduCall,
    ctx: &LintContext,
    baba: &mut Babalawo,
    file: &str,
    span: &Span,
) {
    let is_ffi = call.domain == ifa_types::OduDomain::Coop;
    let is_spawn = call.domain == ifa_types::OduDomain::Ogunda
        && (call.method == "run" || call.method == "bẹrẹ");
    if is_ffi || is_spawn {
        if ctx.is_strict {
            if !ctx.in_ailewu {
                baba.error(
                    "UNAUTHORIZED_ESCAPE",
                    &format!(
                        "Unauthorized {} escape outside 'ailewu' block",
                        if is_ffi { "FFI" } else { "Process spawn" }
                    ),
                    file,
                    span.line,
                    span.column,
                );
            }
        } else if !ctx.in_ailewu {
            baba.warning(
                "UNSAFE_ESCAPE_WARNING",
                &format!(
                    "{} escape should be enclosed in 'ailewu' block",
                    if is_ffi { "FFI" } else { "Process spawn" }
                ),
                file,
                span.line,
                span.column,
            );
        }
    }
}

/// Check if an expression uses a variable
pub(crate) fn expression_uses_var(expr: &Expression, var_name: &str) -> bool {
    match &expr.kind {
        ExprKind::Identifier(name) => name == var_name,
        ExprKind::BinaryOp { left, right, .. } => {
            expression_uses_var(left, var_name) || expression_uses_var(right, var_name)
        }
        ExprKind::List(items) => items.iter().any(|i| expression_uses_var(i, var_name)),
        ExprKind::Index { object, index, .. } => {
            expression_uses_var(object, var_name) || expression_uses_var(index, var_name)
        }
        _ => false,
    }
}
