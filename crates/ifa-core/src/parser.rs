//! # Ifá-Lang Parser
//!
//! Parses Ifá-Lang source code into an AST using pest.

use pest::Parser;
use pest_derive::Parser;

use crate::ast::*;
use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;

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
            let inner = pair.into_inner().next().unwrap();
            parse_statement(inner)
        }

        Rule::var_decl => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();

            let mut type_hint = None;
            let mut value_pair = inner.next().unwrap();

            if value_pair.as_rule() == Rule::type_hint {
                type_hint = Some(parse_type_hint(value_pair)?);
                value_pair = inner.next().unwrap();
            }

            let value = parse_expression(value_pair)?;

            Ok(Some(Statement::VarDecl {
                name,
                type_hint,
                value,
                visibility: Visibility::default(),
                span,
            }))
        }

        Rule::assignment_stmt => {
            let mut inner = pair.into_inner();
            let target_str = inner.next().unwrap().as_str().to_string();
            let value = parse_expression(inner.next().unwrap())?;

            Ok(Some(Statement::Assignment {
                target: AssignTarget::Variable(target_str),
                value,
                span,
            }))
        }

        Rule::import_stmt => {
            let mut inner = pair.into_inner();
            let path_pair = inner.next().unwrap();
            let path: Vec<String> = path_pair
                .into_inner()
                .map(|p| p.as_str().to_string())
                .collect();

            Ok(Some(Statement::Import { path, span }))
        }

        Rule::instruction => {
            let call_pair = pair.into_inner().next().unwrap();
            let call = parse_odu_call(call_pair)?;
            Ok(Some(Statement::Instruction { call, span }))
        }

        Rule::if_stmt => {
            let mut inner = pair.into_inner();
            let condition = parse_expression(inner.next().unwrap())?;

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
                        let mut else_stmts = Vec::new();
                        for ep in p.into_inner() {
                            if let Some(stmt) = parse_statement(ep)? {
                                else_stmts.push(stmt);
                            }
                        }
                        else_body = Some(else_stmts);
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
            let condition = parse_expression(inner.next().unwrap())?;

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
            let var = inner.next().unwrap().as_str().to_string();
            let iterable = parse_expression(inner.next().unwrap())?;

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

        Rule::ase_stmt => Ok(Some(Statement::Ase { span })),

        Rule::taboo_stmt => {
            let mut inner = pair.into_inner();
            // Skip the taboo keyword
            inner.next();
            let source = inner.next().unwrap().as_str().to_string();
            let target = inner.next().unwrap().as_str().to_string();
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
            let condition = parse_expression(inner.next().unwrap())?;
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
            let size = inner.next().unwrap().as_str().to_string();
            Ok(Some(Statement::Opon { size, span }))
        }

        Rule::ese_def => {
            let mut inner = pair.into_inner();
            let mut visibility = Visibility::Private;

            let first = inner.next().unwrap();
            let (name, remaining) = if first.as_rule() == Rule::public_mod {
                visibility = Visibility::Public;
                (inner.next().unwrap().as_str().to_string(), inner)
            } else {
                (first.as_str().to_string(), inner)
            };

            let mut params = Vec::new();
            let mut body = Vec::new();

            for p in remaining {
                match p.as_rule() {
                    Rule::params => {
                        for param_pair in p.into_inner() {
                            let mut param_inner = param_pair.into_inner();
                            let param_name = param_inner.next().unwrap().as_str().to_string();
                            let param_type = param_inner.next().map(parse_type_hint).transpose()?;
                            params.push(Param {
                                name: param_name,
                                type_hint: param_type,
                            });
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
                span,
            }))
        }

        Rule::odu_def => {
            let mut inner = pair.into_inner();
            let mut visibility = Visibility::Private;

            let first = inner.next().unwrap();
            let (name, remaining) = if first.as_rule() == Rule::public_mod {
                visibility = Visibility::Public;
                (inner.next().unwrap().as_str().to_string(), inner)
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

        Rule::EOI => Ok(None),

        _ => Ok(None),
    }
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> IfaResult<Expression> {
    match pair.as_rule() {
        Rule::expression
        | Rule::or_expr
        | Rule::and_expr
        | Rule::not_expr
        | Rule::comparison
        | Rule::arith_expr
        | Rule::term
        | Rule::factor => {
            let mut inner = pair.into_inner();
            let first = inner.next().unwrap();
            let mut left = parse_expression(first)?;

            while let Some(op_pair) = inner.next() {
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

        Rule::atom => {
            let inner = pair.into_inner().next().unwrap();
            parse_expression(inner)
        }

        Rule::number => {
            let s = pair.as_str();
            if s.contains('.') {
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

        Rule::ident => Ok(Expression::Identifier(pair.as_str().to_string())),

        Rule::odu_call => Ok(Expression::OduCall(parse_odu_call(pair)?)),

        Rule::method_call => {
            let mut inner = pair.into_inner();
            let object_name = inner.next().unwrap().as_str().to_string();
            let method = inner.next().unwrap().as_str().to_string();

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
                let key = parse_expression(inner.next().unwrap())?;
                let value = parse_expression(inner.next().unwrap())?;
                entries.push((key, value));
            }
            Ok(Expression::Map(entries))
        }

        Rule::index_access => {
            let mut inner = pair.into_inner();
            let object = Expression::Identifier(inner.next().unwrap().as_str().to_string());
            let index = parse_expression(inner.next().unwrap())?;
            Ok(Expression::Index {
                object: Box::new(object),
                index: Box::new(index),
            })
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

    let domain_str = inner.next().unwrap().as_str();
    let domain = parse_odu_domain(domain_str)?;
    let method = inner.next().unwrap().as_str().to_string();

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
        // Pseudo-domains
        "coop" | "ajose" => Ok(OduDomain::Coop),
        "opele" | "oracle" => Ok(OduDomain::Opele),
        _ => Err(IfaError::Parse(format!("Unknown Odù domain: {}", s))),
    }
}

fn parse_type_hint(pair: pest::iterators::Pair<Rule>) -> IfaResult<TypeHint> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_str() {
        "Int" | "Number" => Ok(TypeHint::Int),
        "Float" => Ok(TypeHint::Float),
        "Str" | "String" => Ok(TypeHint::Str),
        "Bool" => Ok(TypeHint::Bool),
        "List" | "Array" => Ok(TypeHint::List),
        "Map" | "Dict" => Ok(TypeHint::Map),
        "Any" => Ok(TypeHint::Any),
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

fn make_span(pair: &pest::iterators::Pair<Rule>) -> Span {
    let pest_span = pair.as_span();
    Span {
        start: pest_span.start(),
        end: pest_span.end(),
        line: pest_span.start_pos().line_col().0,
        column: pest_span.start_pos().line_col().1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_var_decl() {
        let program = parse("ayanmo x = 42;").unwrap();
        assert_eq!(program.statements.len(), 1);

        if let Statement::VarDecl { name, value, .. } = &program.statements[0] {
            assert_eq!(name, "x");
            assert!(matches!(value, Expression::Int(42)));
        } else {
            panic!("Expected VarDecl");
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
            panic!("Expected Instruction");
        }
    }

    #[test]
    fn test_parse_if() {
        let program = parse("ti x { ayanmo y = 1; }").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(&program.statements[0], Statement::If { .. }));
    }
}
