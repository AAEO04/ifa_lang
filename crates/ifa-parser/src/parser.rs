//! # Ifá-Lang Parser
//!
//! Parses Ifá-Lang source code into an AST using pest.

use pest::Parser;
use pest_derive::Parser;

use crate::lexer::OduDomain;
use ifa_types::ast::*;
use ifa_types::{IfaError, IfaResult};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct IfaParser;

/// Parse source code into a Program AST
pub fn parse(source: &str) -> IfaResult<Program> {
    let pairs =
        IfaParser::parse(Rule::program, source).map_err(|e| IfaError::Parse(format!("{}", e)))?;

    let mut statements = Vec::new();

    for pair in pairs {
        if pair.as_rule() == Rule::program {
            for inner in pair.into_inner() {
                if let Some(stmt) = parse_statement(inner)? {
                    statements.push(stmt);
                }
            }
        }
    }

    Ok(Program { statements })
}

fn parse_statement(pair: pest::iterators::Pair<Rule>) -> IfaResult<Option<Statement>> {
    let span = make_span(&pair);

    match pair.as_rule() {
        Rule::statement => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| IfaError::Parse("Statement rule cannot be empty".to_string()))?;
            parse_statement(inner)
        }

        Rule::var_decl => {
            let mut inner = pair.into_inner();
            let mut visibility = Visibility::Private;

            let first = inner
                .next()
                .ok_or_else(|| IfaError::Parse("VarDecl missing name".to_string()))?;
            let (name, mut inner) = if first.as_rule() == Rule::public_mod {
                visibility = Visibility::Public;
                let name = inner
                    .next()
                    .ok_or_else(|| IfaError::Parse("VarDecl missing name".to_string()))?
                    .as_str()
                    .to_string();
                (name, inner)
            } else {
                (first.as_str().to_string(), inner)
            };

            let mut type_hint = None;
            let mut value_pair = inner
                .next()
                .ok_or_else(|| IfaError::Parse("VarDecl missing value/type".to_string()))?;

            if value_pair.as_rule() == Rule::type_hint {
                type_hint = Some(parse_type_hint(value_pair)?);
                value_pair = inner.next().ok_or_else(|| {
                    IfaError::Parse("VarDecl missing value after type hint".to_string())
                })?;
            }

            let value = parse_expression(value_pair)?;

            Ok(Some(Statement::VarDecl {
                name,
                type_hint,
                value,
                visibility,
                span,
            }))
        }

        Rule::const_stmt => {
            let mut inner = pair.into_inner();
            let mut visibility = Visibility::Private;
            let first = inner
                .next()
                .ok_or_else(|| IfaError::Parse("Const missing name".to_string()))?;
            let (name, mut inner) = if first.as_rule() == Rule::public_mod {
                visibility = Visibility::Public;
                inner.next(); // Skip const keyword
                let name = inner
                    .next()
                    .ok_or_else(|| IfaError::Parse("Const missing name".to_string()))?
                    .as_str()
                    .to_string();
                (name, inner)
            } else {
                // first is const keyword
                let name = inner
                    .next()
                    .ok_or_else(|| IfaError::Parse("Const missing name".to_string()))?
                    .as_str()
                    .to_string();
                (name, inner)
            };
            let value_pair = inner
                .next()
                .ok_or_else(|| IfaError::Parse("Const missing value".to_string()))?;
            let value = parse_expression(value_pair)?;
            Ok(Some(Statement::Const {
                name,
                value,
                visibility,
                span,
            }))
        }
        Rule::assignment_stmt => {
            let mut inner = pair.into_inner();
            let lvalue_pair = inner
                .next()
                .ok_or_else(|| IfaError::Parse("Assignment missing lvalue".to_string()))?;
            let value = parse_expression(
                inner
                    .next()
                    .ok_or_else(|| IfaError::Parse("Assignment missing value".to_string()))?,
            )?;

            let target = parse_lvalue(lvalue_pair)?;

            Ok(Some(Statement::Assignment {
                target,
                value,
                span,
            }))
        }

        Rule::update_stmt => {
            let mut inner = pair.into_inner();
            let first = inner
                .next()
                .ok_or_else(|| IfaError::Parse("Update missing target".to_string()))?;

            let target = match first.as_rule() {
                Rule::ident => AssignTarget::Variable(first.as_str().to_string()),
                Rule::lvalue => parse_lvalue(first)?,
                _ => {
                    return Err(IfaError::Parse(format!(
                        "Invalid update target: {:?}",
                        first.as_rule()
                    )));
                }
            };

            let op_pair = inner
                .next()
                .ok_or_else(|| IfaError::Parse("Update missing op".to_string()))?;
            let op = match op_pair.as_str() {
                "+=" => UpdateOp::AddAssign,
                "-=" => UpdateOp::SubAssign,
                "*=" => UpdateOp::MulAssign,
                "/=" => UpdateOp::DivAssign,
                "%=" => UpdateOp::ModAssign,
                _ => {
                    return Err(IfaError::Parse(format!(
                        "Unknown update operator: {}",
                        op_pair.as_str()
                    )));
                }
            };

            let value = if let Some(val_pair) = inner.next() {
                Some(parse_expression(val_pair)?)
            } else {
                None
            };

            Ok(Some(Statement::Update {
                target,
                op,
                value: Some(value.ok_or_else(|| IfaError::Parse("Update missing value".into()))?),
                span,
            }))
        }

        Rule::import_stmt => {
            let mut inner = pair.into_inner();
            let path_pair = inner
                .next()
                .ok_or_else(|| IfaError::Parse("Import missing path".to_string()))?;
            let path: Vec<String> = path_pair
                .into_inner()
                .map(|p| p.as_str().to_string())
                .collect();

            Ok(Some(Statement::Import {
                path,
                names: None,
                span,
            }))
        }

        Rule::from_import_stmt => {
            let inner = pair.into_inner();
            let mut names = Vec::new();
            let mut path: Option<Vec<String>> = None;

            for p in inner {
                match p.as_rule() {
                    Rule::ident => names.push(p.as_str().to_string()),
                    Rule::module_path => {
                        path = Some(p.into_inner().map(|seg| seg.as_str().to_string()).collect());
                    }
                    _ => {}
                }
            }

            let path = path.ok_or_else(|| IfaError::Parse("Import missing path".to_string()))?;
            Ok(Some(Statement::Import {
                path,
                names: Some(names),
                span,
            }))
        }

        Rule::instruction => {
            let call_pair = pair
                .into_inner()
                .next()
                .ok_or_else(|| IfaError::Parse("Instruction missing body".to_string()))?;
            match call_pair.as_rule() {
                Rule::odu_call => {
                    let call = parse_odu_call(call_pair)?;
                    Ok(Some(Statement::Instruction { call, span }))
                }
                _ => {
                    let expr = parse_expression(call_pair)?;
                    Ok(Some(Statement::Expr { expr, span }))
                }
            }
        }

        Rule::if_stmt => {
            let mut inner = pair.into_inner();
            let condition = parse_expression(
                inner
                    .next()
                    .ok_or_else(|| IfaError::Parse("If stmt missing condition".to_string()))?,
            )?;

            let mut then_body = Vec::new();
            let mut else_body = None;

            for p in inner {
                match p.as_rule() {
                    Rule::statement => {
                        if let Some(stmt) = parse_statement(p)? {
                            then_body.push(stmt);
                        }
                    }
                    Rule::else_clause => {
                        else_body = Some(parse_else_clause(p)?);
                    }
                    _ => {}
                }
            }

            Ok(Some(Statement::If {
                condition,
                then_body,
                else_body,
                span,
            }))
        }

        Rule::while_stmt => {
            let mut inner = pair.into_inner();
            let condition = parse_expression(
                inner
                    .next()
                    .ok_or(IfaError::Parse("While missing condition".into()))?,
            )?;

            let mut body = Vec::new();
            for p in inner {
                if let Some(stmt) = parse_statement(p)? {
                    body.push(stmt);
                }
            }

            Ok(Some(Statement::While {
                condition,
                body,
                span,
            }))
        }

        Rule::for_stmt => {
            let mut inner = pair.into_inner();
            let var = inner
                .next()
                .ok_or(IfaError::Parse("For missing var".into()))?
                .as_str()
                .to_string();
            let iterable = parse_expression(
                inner
                    .next()
                    .ok_or(IfaError::Parse("For missing iterable".into()))?,
            )?;

            let mut body = Vec::new();
            for p in inner {
                if let Some(stmt) = parse_statement(p)? {
                    body.push(stmt);
                }
            }

            Ok(Some(Statement::For {
                var,
                iterable,
                body,
                span,
            }))
        }

        Rule::return_stmt => {
            let inner = pair.into_inner().next();
            let value = inner.map(parse_expression).transpose()?;
            Ok(Some(Statement::Return { value, span }))
        }

        Rule::break_stmt => Ok(Some(Statement::Break { span })),

        Rule::continue_stmt => Ok(Some(Statement::Continue { span })),

        Rule::ase_stmt => Ok(Some(Statement::Ase { span })),

        Rule::abo_stmt => Ok(Some(Statement::Abo { span })),

        Rule::taboo_stmt => {
            let mut inner = pair.into_inner();
            // Skip the taboo keyword
            inner.next();
            let source = inner
                .next()
                .ok_or(IfaError::Parse("Taboo missing source".into()))?
                .as_str()
                .to_string();
            let target = inner
                .next()
                .ok_or(IfaError::Parse("Taboo missing target".into()))?
                .as_str()
                .to_string();
            Ok(Some(Statement::Taboo {
                source,
                target,
                span,
            }))
        }

        Rule::ewo_stmt => {
            let mut inner = pair.into_inner();
            // Skip the ewo keyword
            inner.next();
            let condition = parse_expression(
                inner
                    .next()
                    .ok_or(IfaError::Parse("Ewo missing condition".into()))?,
            )?;
            let message = inner
                .next()
                .map(|p| p.as_str().trim_matches('"').to_string());
            Ok(Some(Statement::Ewo {
                condition,
                message,
                span,
            }))
        }

        Rule::opon_stmt => {
            let mut inner = pair.into_inner();
            // Skip the opon keyword
            inner.next();
            let size = inner
                .next()
                .ok_or(IfaError::Parse("Opon missing size".into()))?
                .as_str()
                .to_string();
            Ok(Some(Statement::Opon { size, span }))
        }

        Rule::ebo_stmt => {
            let mut inner = pair.into_inner();
            // inner[0] = offering expression, inner[1..] = optional block body
            let offering = parse_expression(
                inner
                    .next()
                    .ok_or(IfaError::Parse("Ebo missing offering".into()))?,
            )?;
            let body = if let Some(block) = inner.next() {
                let mut stmts = Vec::new();
                for p in block.into_inner() {
                    if let Some(stmt) = parse_statement(p)? {
                        stmts.push(stmt);
                    }
                }
                Some(stmts)
            } else {
                None
            };
            Ok(Some(Statement::Ebo {
                offering,
                body,
                span,
            }))
        }

        Rule::defer_stmt => {
            let mut body = Vec::new();
            for p in pair.into_inner() {
                if let Some(stmt) = parse_statement(p)? {
                    body.push(stmt);
                }
            }
            Ok(Some(Statement::Defer { body, span }))
        }

        Rule::ailewu_stmt => {
            let mut body = Vec::new();
            for p in pair.into_inner() {
                if let Some(stmt) = parse_statement(p)? {
                    body.push(stmt);
                }
            }
            Ok(Some(Statement::Ailewu { body, span }))
        }

        Rule::yield_stmt => {
            let mut inner = pair.into_inner();
            // Keyword is silent in grammar, so first item IS the expression
            let duration = parse_expression(
                inner
                    .next()
                    .ok_or(IfaError::Parse("Yield missing duration".into()))?,
            )?;
            Ok(Some(Statement::Yield { duration, span }))
        }

        Rule::ese_def => {
            let mut inner = pair.into_inner();
            let mut visibility = Visibility::Private;
            let mut effects = Vec::new();

            let first = inner
                .next()
                .ok_or(IfaError::Parse("Ese missing name or modifier".into()))?;
            let mut current = first;
            if current.as_rule() == Rule::public_mod {
                visibility = Visibility::Public;
                current = inner
                    .next()
                    .ok_or(IfaError::Parse("Ese missing name".into()))?;
            }
            if current.as_rule() == Rule::async_mod {
                effects.push(ifa_types::ast::Effect::Async);
                current = inner
                    .next()
                    .ok_or(IfaError::Parse("Ese missing name".into()))?;
            }
            let name = current.as_str().to_string();
            let remaining = inner;

            let mut params = Vec::new();
            let mut body = Vec::new();

            for p in remaining {
                match p.as_rule() {
                    Rule::params => {
                        for param_pair in p.into_inner() {
                            let mut param_inner = param_pair.into_inner();
                            let param_name = param_inner
                                .next()
                                .ok_or(IfaError::Parse("Param missing name".into()))?
                                .as_str()
                                .to_string();
                            let param_type = param_inner.next().map(parse_type_hint).transpose()?;
                            params.push(Param {
                                name: param_name,
                                type_hint: param_type,
                            });
                        }
                    }
                    Rule::effects_decl => {
                        for decl_inner in p.into_inner() {
                            if decl_inner.as_rule() == Rule::effect_list {
                                for effect_ident in decl_inner.into_inner() {
                                    let effect = match effect_ident.as_str() {
                                        "Network" | "Otura" => ifa_types::ast::Effect::Network,
                                        "Async" | "Osa" => ifa_types::ast::Effect::Async,
                                        "FileIO" | "Odi" => ifa_types::ast::Effect::FileIO,
                                        "State" => ifa_types::ast::Effect::State,
                                        "Impure" => ifa_types::ast::Effect::Impure,
                                        "Pure" => ifa_types::ast::Effect::Pure,
                                        _ => return Err(IfaError::Parse(format!("Unknown effect: {}", effect_ident.as_str()))),
                                    };
                                    if !effects.contains(&effect) {
                                        effects.push(effect);
                                    }
                                }
                            }
                        }
                    }
                    Rule::statement => {
                        if let Some(stmt) = parse_statement(p)? {
                            body.push(stmt);
                        }
                    }
                    _ => {}
                }
            }

            Ok(Some(Statement::EseDef {
                name,
                visibility,
                params,
                body,
                effects,
                span,
            }))
        }

        Rule::odu_def => {
            let mut inner = pair.into_inner();
            let mut visibility = Visibility::Private;

            let first = inner
                .next()
                .ok_or(IfaError::Parse("Odu missing name or modifier".into()))?;
            let (name, remaining) = if first.as_rule() == Rule::public_mod {
                visibility = Visibility::Public;
                (
                    inner
                        .next()
                        .ok_or(IfaError::Parse("Odu missing name".into()))?
                        .as_str()
                        .to_string(),
                    inner,
                )
            } else {
                (first.as_str().to_string(), inner)
            };

            let mut body = Vec::new();
            for p in remaining {
                if p.as_rule() == Rule::odu_body {
                    for bp in p.into_inner() {
                        if let Some(stmt) = parse_statement(bp)? {
                            body.push(stmt);
                        }
                    }
                }
            }

            Ok(Some(Statement::OduDef {
                name,
                visibility,
                body,
                span,
            }))
        }

        Rule::match_stmt => {
            let mut inner = pair.into_inner();
            let condition = parse_expression(
                inner
                    .next()
                    .ok_or(IfaError::Parse("Match missing condition".into()))?,
            )?;
            let mut arms = Vec::new();

            for arm_pair in inner {
                let mut arm_inner = arm_pair.into_inner();
                let pattern_pair = arm_inner
                    .next()
                    .ok_or(IfaError::Parse("Match arm missing pattern".into()))?;
                let pattern = parse_match_pattern(pattern_pair)?;

                let mut body = Vec::new();
                let body_pair = arm_inner
                    .next()
                    .ok_or(IfaError::Parse("Match arm missing body".into()))?;
                match body_pair.as_rule() {
                    Rule::statement => {
                        if let Some(stmt) = parse_statement(body_pair)? {
                            body.push(stmt);
                        }
                    }
                    _ => {
                        // Block
                        for stmt_pair in body_pair.into_inner() {
                            if let Some(stmt) = parse_statement(stmt_pair)? {
                                body.push(stmt);
                            }
                        }
                    }
                }

                arms.push(MatchArm { pattern, body });
            }

            Ok(Some(Statement::Match {
                condition,
                arms,
                span,
            }))
        }

        Rule::try_stmt => {
            let inner = pair.into_inner();

            // Try body
            let mut try_body = Vec::new();
            // Pest pair.into_inner() will give statements directly if they are in { statement* }
            // Wait, grammar: try_stmt = { try_kw ~ "{" ~ statement* ~ "}" ~ catch_clause }
            // Keyword is silent/atomic.
            // But statements are direct children of try_stmt?
            // Pest flattens? No.
            // Let's iterate.
            // We need to distinguish try_body statements from catch_clause.
            // catch_clause is a rule itself.

            let mut catch_clause_pair = None;
            let mut nipari_clause_pair = None;

            for p in inner {
                match p.as_rule() {
                    Rule::statement => {
                        if let Some(stmt) = parse_statement(p)? {
                            try_body.push(stmt);
                        }
                    }
                    Rule::catch_clause => {
                        catch_clause_pair = Some(p);
                    }
                    Rule::nipari_clause => {
                        nipari_clause_pair = Some(p);
                    }
                    _ => {}
                }
            }

            let catch_pair = catch_clause_pair
                .ok_or_else(|| IfaError::Parse("Try missing catch clause".to_string()))?;
            let mut catch_inner = catch_pair.into_inner();

            // catch_clause = { catch_kw ~ "(" ~ ident ~ ")" ~ "{" ~ statement* ~ "}" }
            // catch_kw is silent.
            // first is ident.
            let catch_var = catch_inner
                .next()
                .ok_or(IfaError::Parse("Catch missing var".into()))?
                .as_str()
                .to_string();

            let mut catch_body = Vec::new();
            for p in catch_inner {
                if let Some(stmt) = parse_statement(p)? {
                    catch_body.push(stmt);
                }
            }
            // Parse optional nipari (finally) clause
            let finally_body = if let Some(nipari_pair) = nipari_clause_pair {
                let mut finally_stmts = Vec::new();
                for p in nipari_pair.into_inner() {
                    if p.as_rule() == Rule::statement {
                        #[allow(clippy::collapsible_if)]
                        if let Some(stmt) = parse_statement(p)? {
                            finally_stmts.push(stmt);
                        }
                    }
                }
                Some(finally_stmts)
            } else {
                None
            };

            Ok(Some(Statement::Try {
                try_body,
                catch_var,
                catch_body,
                finally_body,
                span,
            }))
        }

        Rule::throw_stmt => {
            let mut inner = pair.into_inner();
            let expr_pair = inner
                .next()
                .ok_or_else(|| IfaError::Parse("Throw missing expression".to_string()))?;
            let expr = parse_expression(expr_pair)?;
            Ok(Some(Statement::Throw { value: expr, span }))
        }

        Rule::EOI => Ok(None),

        _ => Ok(None),
    }
}

/// Parse an `else_clause` node, which in the new grammar can be either:
///   `else { stmts }` → returns Vec<Statement>
///   `else if cond { stmts } else_clause?` → returns Vec<Statement> wrapping a nested If
fn parse_else_clause(pair: pest::iterators::Pair<Rule>) -> IfaResult<Vec<Statement>> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    // Peek at the first child to determine which branch we're in.
    let first = match inner.next() {
        Some(p) => p,
        None => return Ok(vec![]),
    };

    // Grammar: else_clause = { else_kw ~ if_kw ~ expr ~ "{" ~ stmt* ~ "}" ~ else_clause? }
    //        |               { else_kw ~ "{" ~ stmt* ~ "}" }
    // After we consumed first (which is the condition expression for else-if, or the
    // first statement for a plain else block), we determine the branch by its rule.
    if first.as_rule() == Rule::expression {
        // else-if branch: first is the condition
        let condition = parse_expression(first)?;
        let mut then_body = Vec::new();
        let mut else_body = None;

        for p in inner {
            match p.as_rule() {
                Rule::statement => {
                    if let Some(s) = parse_statement(p)? {
                        then_body.push(s);
                    }
                }
                Rule::else_clause => {
                    else_body = Some(parse_else_clause(p)?);
                }
                _ => {}
            }
        }

        Ok(vec![Statement::If {
            condition,
            then_body,
            else_body,
            span,
        }])
    } else {
        // plain else block: first is a statement
        let mut stmts = Vec::new();
        if let Some(s) = parse_statement(first)? {
            stmts.push(s);
        }
        for p in inner {
            if let Some(s) = parse_statement(p)? {
                stmts.push(s);
            }
        }
        Ok(stmts)
    }
}

fn parse_lvalue(pair: pest::iterators::Pair<Rule>) -> IfaResult<AssignTarget> {
    let inner_lvalue = pair
        .into_inner()
        .next()
        .ok_or_else(|| IfaError::Parse("Empty lvalue".to_string()))?;

    match inner_lvalue.as_rule() {
        Rule::ident => Ok(AssignTarget::Variable(inner_lvalue.as_str().to_string())),
        Rule::index_lvalue => {
            let mut index_inner = inner_lvalue.into_inner();
            let name = index_inner
                .next()
                .ok_or_else(|| IfaError::Parse("Index lvalue missing name".to_string()))?
                .as_str()
                .to_string();
            let index_expr = parse_expression(
                index_inner
                    .next()
                    .ok_or_else(|| IfaError::Parse("Index lvalue missing index".to_string()))?,
            )?;
            Ok(AssignTarget::Index {
                name,
                index: Box::new(index_expr),
            })
        }
        Rule::deref_lvalue => {
            let mut deref_inner = inner_lvalue.into_inner();
            let expr =
                parse_expression(deref_inner.next().ok_or_else(|| {
                    IfaError::Parse("Deref lvalue missing expression".to_string())
                })?)?;
            Ok(AssignTarget::Dereference(Box::new(expr)))
        }
        _ => Err(IfaError::Parse(format!(
            "Unexpected lvalue rule: {:?}",
            inner_lvalue.as_rule()
        ))),
    }
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> IfaResult<Expression> {
    match pair.as_rule() {
        Rule::pipeline_expr => {
            let mut inner = pair.into_inner();
            let first = inner
                .next()
                .ok_or_else(|| IfaError::Parse("Empty pipeline expression".to_string()))?;
            let mut left = parse_expression(first)?;

            for next_pair in inner {
                let right = parse_expression(next_pair)?;
                left = desugar_pipeline(left, right)?;
            }
            Ok(left)
        }
        Rule::pow_expr => {
            let mut inner = pair.into_inner();
            let first = inner
                .next()
                .ok_or(IfaError::Parse("Empty expression group".into()))?;
            let mut exprs = vec![parse_expression(first)?];
            while let Some(_op_pair) = inner.next() {
                if let Some(right_pair) = inner.next() {
                    exprs.push(parse_expression(right_pair)?);
                }
            }
            let mut result = exprs.pop().unwrap();
            while let Some(left) = exprs.pop() {
                result = Expression::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOperator::Power,
                    right: Box::new(result),
                };
            }
            Ok(result)
        }
        Rule::expression
        | Rule::null_coalesce_expr
        | Rule::or_expr
        | Rule::and_expr
        | Rule::not_expr
        | Rule::comparison
        | Rule::arith_expr
        | Rule::term
        | Rule::factor => {
            let mut inner = pair.into_inner();
            let first = inner
                .next()
                .ok_or(IfaError::Parse("Empty expression group".into()))?;

            // Check for unary op
            if first.as_rule() == Rule::unary_op {
                // Determine ops recursively below
                // let op_str = first.as_str(); ... (removed unused code)

                // Recursively parse the rest of the factor (which might be another unary op or atom)
                // Since `factor = { unary_op* ~ atom }`, the next pair is either another unary_op or atom.
                // But `unary_op*` means they are siblings in the `factor` pair children.
                // parse_expression logic needs to handle this sibling list.

                // If we are in `factor`, inner contains: [unary_op, unary_op... atom]
                // We need to fold them right-to-left.

                // Actually, let's treat it recursively.
                // If we see unary_op, we consume it, convert to Op, then parse the REST of the expression.
                // But the rest of the expression is not a single pair if there are multiple ops.
                // Wait, if `factor` produced [Op, Op, Atom], `pair` is the parent `factor`.
                // We can iterate the children.

                // REWRITE:
                let mut ops = Vec::new();
                let mut current = first;

                // Collect all leading unary ops
                while current.as_rule() == Rule::unary_op {
                    let op_str = current.as_str();
                    let op = match op_str {
                        "-" => UnaryOperator::Neg,
                        "!" | "kii" | "not" => UnaryOperator::Not,
                        "&" => UnaryOperator::AddressOf,
                        "*" => UnaryOperator::Dereference,
                        "+" => {
                            // Unary plus is identity, ignore? Or error?
                            // Usually ignore.
                            // But strict parser might want it.
                            // For now, let's allow it as identity (no opcode).
                            // But we must continue loop.
                            // Let's just not add to ops list if we want to ignore.
                            // But we need to define UnaryOperator::Plus?
                            // User didn't request it. Let's error for now or treat as error.
                            // Wait, grammar says unary_op = { "+" | ... }.
                            return Err(IfaError::Parse("Unary plus not supported".to_string()));
                        }
                        _ => {
                            return Err(IfaError::Parse(format!(
                                "Unknown unary operator: {}",
                                op_str
                            )));
                        }
                    };
                    ops.push(op);

                    if let Some(next) = inner.next() {
                        current = next;
                    } else {
                        return Err(IfaError::Parse(
                            "Unexpected end of expression after unary op".to_string(),
                        ));
                    }
                }

                // Now `current` is the atom (or anything else that `factor` allows at end, which is `atom`)
                let mut expr = parse_expression(current)?;

                // Apply ops in reverse (right-to-left association)
                // ex: -*p  ->  -(*(p))
                for op in ops.into_iter().rev() {
                    expr = Expression::UnaryOp {
                        op,
                        expr: Box::new(expr),
                    };
                }

                return Ok(expr);
            }

            // Binary Op Handling (Term, Arith, Comp, etc)
            let mut left = parse_expression(first)?;

            while let Some(op_pair) = inner.next() {
                // Check for postfix `?` (try_op) — not a binary infix, just a postfix marker.
                if op_pair.as_rule() == Rule::try_op {
                    left = Expression::Try(Box::new(left));
                    continue;
                }
                if let Some(right_pair) = inner.next() {
                    let op = parse_binary_op(&op_pair)?;
                    let right = parse_expression(right_pair)?;
                    left = Expression::BinaryOp {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    };
                }
            }

            Ok(left)
        }

        Rule::unary_op => {
            // pest usually nests unary ops inside `factor` or `term` depending on grammar
            // In grammar.pest: factor = { unary_op? ~ atom }
            // parse_expression recurses.
            // If this rule is hit directly, it might be an error or recursive call
            // Actually, `parse_expression` handles `factor` rule block below?
            // No, `Rule::factor` is in the first match block. It should handle unary_op there.
            Err(IfaError::Parse(
                "Unexpected unary_op rule at top level".to_string(),
            ))
        }

        Rule::atom => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or(IfaError::Parse("Atom missing content".into()))?;
            parse_expression(inner)
        }

        Rule::property_access => {
            let mut inner = pair.into_inner();
            let mut obj = parse_expression(
                inner
                    .next()
                    .ok_or(IfaError::Parse("Property access missing object".into()))?,
            )?;

            while let Some(op_pair) = inner.next() {
                let is_optional = op_pair.as_str() == "?.";
                let name = inner
                    .next()
                    .ok_or(IfaError::Parse("Property access missing field name".into()))?
                    .as_str()
                    .to_string();

                obj = Expression::Get {
                    object: Box::new(obj),
                    name,
                    is_optional,
                };
            }
            Ok(obj)
        }

        Rule::number => {
            let s = pair.as_str().replace('_', "");
            if let Some(hex) = s.strip_prefix("0x") {
                let val = i64::from_str_radix(hex, 16)
                    .map_err(|_| IfaError::Parse("Invalid hex literal".to_string()))?;
                Ok(Expression::Int(val))
            } else if let Some(bin) = s.strip_prefix("0b") {
                let val = i64::from_str_radix(bin, 2)
                    .map_err(|_| IfaError::Parse("Invalid binary literal".to_string()))?;
                Ok(Expression::Int(val))
            } else if s.contains('.') {
                Ok(Expression::Float(s.parse().unwrap_or(0.0)))
            } else {
                Ok(Expression::Int(s.parse().unwrap_or(0)))
            }
        }

        Rule::string => {
            let s = pair.as_str();
            Ok(Expression::String(s[1..s.len() - 1].to_string()))
        }

        Rule::boolean => {
            let s = pair.as_str();
            Ok(Expression::Bool(s == "true" || s == "otito"))
        }

        Rule::nil => Ok(Expression::Nil),

        Rule::ident => Ok(Expression::Identifier(pair.as_str().to_string())),

        Rule::odu_call => Ok(Expression::OduCall(parse_odu_call(pair)?)),

        Rule::method_call => {
            let mut inner = pair.into_inner();
            let object_name = inner
                .next()
                .ok_or(IfaError::Parse("Method call missing object".into()))?
                .as_str()
                .to_string();
            let op = inner
                .next()
                .ok_or(IfaError::Parse("Method call missing operator".into()))?;
            let is_optional = op.as_str() == "?.";
            let method = inner
                .next()
                .ok_or(IfaError::Parse("Method call missing method name".into()))?
                .as_str()
                .to_string();

            let mut args = Vec::new();
            if let Some(args_pair) = inner.next() {
                for arg in args_pair.into_inner() {
                    args.push(parse_expression(arg)?);
                }
            }

            Ok(Expression::MethodCall {
                object: Box::new(Expression::Identifier(object_name)),
                method,
                args,
                is_optional,
            })
        }

        Rule::function_call => {
            let mut inner = pair.into_inner();
            let name = inner
                .next()
                .ok_or(IfaError::Parse("Function call missing name".into()))?
                .as_str()
                .to_string();

            let mut args = Vec::new();
            if let Some(args_pair) = inner.next() {
                for arg in args_pair.into_inner() {
                    args.push(parse_expression(arg)?);
                }
            }

            Ok(Expression::Call { name, args })
        }

        Rule::await_expr => {
            let mut inner = pair.into_inner();
            let expr = parse_expression(
                inner
                    .next()
                    .ok_or(IfaError::Parse("Await missing expression".into()))?,
            )?;
            Ok(Expression::Await(Box::new(expr)))
        }

        Rule::move_expr => {
            let mut inner = pair.into_inner();
            let expr = parse_expression(
                inner
                    .next()
                    .ok_or(IfaError::Parse("Move missing expression".into()))?,
            )?;
            Ok(Expression::MoveExpr(Box::new(expr)))
        }

        Rule::index_access => {
            let mut inner = pair.into_inner();
            let object_name = inner
                .next()
                .ok_or(IfaError::Parse("Index access missing object".into()))?
                .as_str()
                .to_string();

            let mut is_optional = false;
            let next_pair = inner
                .next()
                .ok_or(IfaError::Parse("Index access missing index".into()))?;
            let index_expr_pair = if next_pair.as_rule() == Rule::optional_chain_op {
                is_optional = true;
                inner.next().ok_or(IfaError::Parse(
                    "Index access missing index after ?.".into(),
                ))?
            } else {
                next_pair
            };

            let index = parse_expression(index_expr_pair)?;
            Ok(Expression::Index {
                object: Box::new(Expression::Identifier(object_name)),
                index: Box::new(index),
                is_optional,
            })
        }

        Rule::list_literal => {
            let mut items = Vec::new();
            for item in pair.into_inner() {
                items.push(parse_expression(item)?);
            }
            Ok(Expression::List(items))
        }

        Rule::map_literal => {
            let mut entries = Vec::new();
            for entry in pair.into_inner() {
                let mut inner = entry.into_inner();
                let key = parse_expression(
                    inner
                        .next()
                        .ok_or(IfaError::Parse("Map entry missing key".into()))?,
                )?;
                let value = parse_expression(
                    inner
                        .next()
                        .ok_or(IfaError::Parse("Map entry missing value".into()))?,
                )?;
                entries.push((key, value));
            }
            Ok(Expression::Map(entries))
        }

        Rule::interpolated_string => {
            let mut parts = Vec::new();
            for part_pair in pair.into_inner() {
                let inner = part_pair.into_inner().next().ok_or_else(|| {
                    IfaError::Parse("Interpolated string part missing content".to_string())
                })?;
                match inner.as_rule() {
                    Rule::interp_text => {
                        parts.push(InterpolatedPart::Literal(inner.as_str().to_string()));
                    }
                    Rule::interp_expr => {
                        let expr_pair = inner.into_inner().next().ok_or_else(|| {
                            IfaError::Parse("Interpolated expression missing".to_string())
                        })?;
                        let expr = parse_expression(expr_pair)?;
                        parts.push(InterpolatedPart::Expression(Box::new(expr)));
                    }
                    _ => {}
                }
            }
            Ok(Expression::InterpolatedString { parts })
        }

        Rule::lambda_expr => {
            // lambda_expr = { ese_kw ~ "(" ~ params? ~ ")" ~ "{" ~ statement* ~ "}" }
            let mut params: Vec<String> = Vec::new();
            let mut body: Vec<Statement> = Vec::new();
            for p in pair.into_inner() {
                match p.as_rule() {
                    Rule::params => {
                        for param_pair in p.into_inner() {
                            let mut pi = param_pair.into_inner();
                            let name = pi
                                .next()
                                .ok_or(IfaError::Parse("Lambda param missing name".into()))?
                                .as_str()
                                .to_string();
                            params.push(name);
                        }
                    }
                    Rule::statement => {
                        if let Some(s) = parse_statement(p)? {
                            body.push(s);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Expression::Lambda { params, body })
        }

        _ => Err(IfaError::Parse(format!(
            "Unexpected rule: {:?}",
            pair.as_rule()
        ))),
    }
}

fn parse_odu_call(pair: pest::iterators::Pair<Rule>) -> IfaResult<OduCall> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    let domain_str = inner
        .next()
        .ok_or(IfaError::Parse("Odu call missing domain".into()))?
        .as_str();
    let domain = parse_odu_domain(domain_str)?;

    let op = inner
        .next()
        .ok_or(IfaError::Parse("Odu call missing operator".into()))?;
    let is_optional = op.as_str() == "?.";

    let method = inner
        .next()
        .ok_or(IfaError::Parse("Odu call missing method".into()))?
        .as_str()
        .to_string();

    let mut args = Vec::new();
    if let Some(args_pair) = inner.next() {
        for arg in args_pair.into_inner() {
            args.push(parse_expression(arg)?);
        }
    }

    Ok(OduCall {
        domain,
        method,
        args,
        is_optional,
        resolved_domain: None,
        resolved_method_id: None,
        span,
    })
}

fn parse_odu_domain(s: &str) -> IfaResult<OduDomain> {
    let lower = s
        .to_lowercase()
        .replace('ọ', "o")
        .replace('ẹ', "e")
        .replace('ṣ', "s")
        .replace(['à', 'á'], "a")
        .replace(['è', 'é'], "e")
        .replace(['ì', 'í'], "i")
        .replace(['ò', 'ó'], "o")
        .replace(['ù', 'ú'], "u");

    match lower.as_str() {
        "ogbe" => Ok(OduDomain::Ogbe),
        "oyeku" => Ok(OduDomain::Oyeku),
        "iwori" => Ok(OduDomain::Iwori),
        "odi" => Ok(OduDomain::Odi),
        "irosu" => Ok(OduDomain::Irosu),
        "owonrin" => Ok(OduDomain::Owonrin),
        "obara" => Ok(OduDomain::Obara),
        "okanran" => Ok(OduDomain::Okanran),
        "ogunda" => Ok(OduDomain::Ogunda),
        "osa" => Ok(OduDomain::Osa),
        "ika" => Ok(OduDomain::Ika),
        "oturupon" => Ok(OduDomain::Oturupon),
        "otura" => Ok(OduDomain::Otura),
        "irete" => Ok(OduDomain::Irete),
        "ose" => Ok(OduDomain::Ose),
        "ofun" => Ok(OduDomain::Ofun),
        // Reserved pseudo-domains (no dispatch ID yet)
        "coop" | "ajose" => Ok(OduDomain::Coop),
        "opele" | "oracle" => Ok(OduDomain::Opele),

        // Infrastructure
        "sys" | "system" => Ok(OduDomain::Sys),
        "cpu" => Ok(OduDomain::Cpu),
        "gpu" => Ok(OduDomain::Gpu),
        "storage" | "store" | "db" => Ok(OduDomain::Storage),

        // Note: Audio (ohun) belongs to Irosu dispatch.
        // Video (fidio), Backend, Frontend, Crypto, Ml, GameDev, IoT
        // are external packages — use 'iba' imports, not Odù domain calls.
        _ => Err(IfaError::Parse(format!(
            "Unknown Odù domain: '{}'. If this is a library (crypto, ml, gamedev, \
             backend, frontend, iot, video), use 'iba' to import it as a package.",
            s
        ))),
    }
}

fn parse_type_hint(pair: pest::iterators::Pair<Rule>) -> IfaResult<TypeHint> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or(IfaError::Parse("Type hint missing inner".into()))?;
    match inner.as_str() {
        "Int" | "Number" | "int" => Ok(TypeHint::Int),
        "Float" | "float" => Ok(TypeHint::Float),
        "Str" | "String" | "str" => Ok(TypeHint::Str),
        "Bool" | "bool" => Ok(TypeHint::Bool),
        "List" | "Array" | "list" => Ok(TypeHint::List),
        "Map" | "Dict" | "map" => Ok(TypeHint::Map),
        "Any" | "any" => Ok(TypeHint::Any),
        "i8" => Ok(TypeHint::I8),
        "i16" => Ok(TypeHint::I16),
        "i32" => Ok(TypeHint::I32),
        "i64" => Ok(TypeHint::I64),
        "u8" => Ok(TypeHint::U8),
        "u16" => Ok(TypeHint::U16),
        "u32" => Ok(TypeHint::U32),
        "u64" => Ok(TypeHint::U64),
        "f32" => Ok(TypeHint::F32),
        "f64" => Ok(TypeHint::F64),
        "void" => Ok(TypeHint::Void),
        other => Ok(TypeHint::Custom(other.to_string())),
    }
}

fn parse_binary_op(pair: &pest::iterators::Pair<Rule>) -> IfaResult<BinaryOperator> {
    match pair.as_rule() {
        Rule::add_op => Ok(BinaryOperator::Add),
        Rule::sub_op => Ok(BinaryOperator::Sub),
        Rule::mul_op => Ok(BinaryOperator::Mul),
        Rule::div_op => Ok(BinaryOperator::Div),
        Rule::mod_op => Ok(BinaryOperator::Mod),
        Rule::pow_op => Ok(BinaryOperator::Power),
        Rule::null_coalesce_op => Ok(BinaryOperator::NullCoalesce),
        Rule::comp_op => match pair.as_str() {
            "==" => Ok(BinaryOperator::Eq),
            "!=" => Ok(BinaryOperator::NotEq),
            "<" => Ok(BinaryOperator::Lt),
            "<=" => Ok(BinaryOperator::LtEq),
            ">" => Ok(BinaryOperator::Gt),
            ">=" => Ok(BinaryOperator::GtEq),
            _ => Err(IfaError::Parse("Unknown comparison operator".to_string())),
        },
        Rule::and_op => Ok(BinaryOperator::And),
        Rule::or_op => Ok(BinaryOperator::Or),
        _ => Err(IfaError::Parse(format!(
            "Unknown operator: {:?}",
            pair.as_rule()
        ))),
    }
}

fn parse_match_pattern(pair: pest::iterators::Pair<Rule>) -> IfaResult<MatchPattern> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or(IfaError::Parse("Match pattern missing inner".into()))?;
    match inner.as_rule() {
        Rule::literal_pattern => {
            let expr = parse_expression(
                inner
                    .into_inner()
                    .next()
                    .ok_or(IfaError::Parse("Literal pattern missing expr".into()))?,
            )?;
            Ok(MatchPattern::Literal(expr))
        }
        Rule::range_pattern => {
            let mut range_inner = inner.into_inner();
            let start = parse_expression(
                range_inner
                    .next()
                    .ok_or(IfaError::Parse("Range pattern missing start".into()))?,
            )?;
            let end = parse_expression(
                range_inner
                    .next()
                    .ok_or(IfaError::Parse("Range pattern missing end".into()))?,
            )?;
            Ok(MatchPattern::Range {
                start: Box::new(start),
                end: Box::new(end),
            })
        }
        Rule::wildcard_pattern => Ok(MatchPattern::Wildcard),
        _ => Err(IfaError::Parse(format!(
            "Unexpected pattern rule: {:?}",
            inner.as_rule()
        ))),
    }
}

fn make_span(pair: &pest::iterators::Pair<Rule>) -> Span {
    let pest_span = pair.as_span();
    Span {
        start: pest_span.start(),
        end: pest_span.end(),
        line: pest_span.start_pos().line_col().0,
        column: pest_span.start_pos().line_col().1,
    }
}

fn desugar_pipeline(lhs: Expression, rhs: Expression) -> IfaResult<Expression> {
    match rhs {
        Expression::Call { name, mut args } => {
            args.insert(0, lhs);
            Ok(Expression::Call { name, args })
        }
        Expression::Identifier(name) => Ok(Expression::Call {
            name,
            args: vec![lhs],
        }),
        Expression::MethodCall {
            object,
            method,
            mut args,
            is_optional,
        } => {
            args.insert(0, lhs);
            Ok(Expression::MethodCall {
                object,
                method,
                args,
                is_optional,
            })
        }
        Expression::OduCall(mut call) => {
            call.args.insert(0, lhs);
            Ok(Expression::OduCall(call))
        }
        Expression::Get {
            object,
            name,
            is_optional,
        } => Ok(Expression::MethodCall {
            object,
            method: name,
            args: vec![lhs],
            is_optional,
        }),
        other => Err(IfaError::Parse(format!(
            "RHS of pipeline operator must be a function, method, or domain call. Found: {:?}",
            other
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pipeline_operator() {
        let program = parse("\"hello\" |> print;").unwrap();
        assert_eq!(program.statements.len(), 1);

        if let Statement::Expr { expr, .. } = &program.statements[0] {
            if let Expression::Call { name, args } = expr {
                assert_eq!(name, "print");
                assert_eq!(args.len(), 1);
                assert!(matches!(&args[0], Expression::String(s) if s == "hello"));
            } else {
                assert!(false, "Expected Expression::Call");
            }
        } else {
            assert!(false, "Expected Statement::Expr");
        }
    }

    #[test]
    fn test_parse_var_decl() {
        let program = parse("ayanmo x = 42;").unwrap();
        assert_eq!(program.statements.len(), 1);

        if let Statement::VarDecl { name, value, .. } = &program.statements[0] {
            assert_eq!(name, "x");
            assert!(matches!(value, Expression::Int(42)));
        } else {
            assert!(false, "Expected VarDecl");
        }
    }

    #[test]
    fn test_parse_odu_call() {
        let program = parse("Irosu.fo(\"Hello\");").unwrap();
        assert_eq!(program.statements.len(), 1);

        if let Statement::Instruction { call, .. } = &program.statements[0] {
            assert_eq!(call.domain, OduDomain::Irosu);
            assert_eq!(call.method, "fo");
        } else {
            assert!(false, "Expected Instruction");
        }
    }

    #[test]
    fn test_parse_if() {
        let program = parse("ti x { ayanmo y = 1; }").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(&program.statements[0], Statement::If { .. }));
    }
}
