use ifa_types::ast::{Expression, Program, Statement};
use ifa_types::capability::{CapabilitySet, Ofun};
use std::path::PathBuf;

/// Infer required capabilities from the program AST
pub fn infer_capabilities(program: &Program) -> CapabilitySet {
    let mut caps = CapabilitySet::new();

    // Always grant stdio (usually safe/expected)
    caps.grant(Ofun::Stdio);

    // Initial scan
    for stmt in &program.statements {
        scan_statement(stmt, &mut caps);
    }

    caps
}

fn scan_odu_call(call: &ifa_types::ast::OduCall, caps: &mut CapabilitySet) {
    let domain = format!("{:?}", call.domain).to_lowercase();
    match domain.as_str() {
        "coop" => {
            caps.grant(Ofun::Bridge {
                language: call.method.clone(),
            });
        }
        "odi" => {
            // File I/O
            // Odi.ka("file") -> Read
            // Odi.ko("file") -> Write
            if call.method == "ka" || call.method == "read" {
                if let Some(path) = extract_string_arg(&call.args, 0) {
                    caps.grant(Ofun::ReadFiles {
                        root: PathBuf::from(path),
                    });
                } else {
                    // If dynamic path, grant generous permission
                    caps.grant(Ofun::ReadFiles {
                        root: PathBuf::from("/"),
                    });
                }
            } else if call.method == "ko" || call.method == "write" {
                if let Some(path) = extract_string_arg(&call.args, 0) {
                    caps.grant(Ofun::WriteFiles {
                        root: PathBuf::from(path),
                    });
                } else {
                    caps.grant(Ofun::WriteFiles {
                        root: PathBuf::from("/"),
                    });
                }
            }
        }
        "otura" => {
            // Networking
            caps.grant(Ofun::Network {
                domains: vec!["*".to_string()],
            });
        }
        "ogunda" => {
            // Processes
            if call.method == "bẹrẹ" || call.method == "run" {
                if let Some(prog) = extract_string_arg(&call.args, 0) {
                    caps.grant(Ofun::Execute {
                        programs: vec![prog],
                    });
                } else {
                    caps.grant(Ofun::Execute {
                        programs: vec!["*".to_string()],
                    });
                }
            }
        }
        "iwori" => {
            if call.method == "akoko" || call.method == "isisinyi" {
                caps.grant(Ofun::Time);
            }
        }
        "owonrin" => {
            caps.grant(Ofun::Random);
        }
        "ogbe" => {
            if call.method == "ayika" || call.method == "env" {
                caps.grant(Ofun::Environment {
                    keys: vec!["*".to_string()],
                });
            }
        }
        _ => {}
    }

    for arg in &call.args {
        scan_expression(arg, caps);
    }
}

fn scan_statement(stmt: &Statement, caps: &mut CapabilitySet) {
    match stmt {
        Statement::Instruction { call, .. } => {
            scan_odu_call(call, caps);
        }
        Statement::VarDecl { value, .. } => scan_expression(value, caps),
        Statement::Assignment { value, .. } => scan_expression(value, caps),
        Statement::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            scan_expression(condition, caps);
            for s in then_body {
                scan_statement(s, caps);
            }
            if let Some(else_stmts) = else_body {
                for s in else_stmts {
                    scan_statement(s, caps);
                }
            }
        }
        Statement::While {
            condition, body, ..
        } => {
            scan_expression(condition, caps);
            for s in body {
                scan_statement(s, caps);
            }
        }
        Statement::For { iterable, body, .. } => {
            scan_expression(iterable, caps);
            for s in body {
                scan_statement(s, caps);
            }
        }
        Statement::EseDef {
            body,
            return_type: _,
            effects: _,
            ..
        } => {
            for s in body {
                scan_statement(s, caps);
            }
        }
        Statement::OduDef { body, .. } => {
            for s in body {
                scan_statement(s, caps);
            }
        }
        Statement::Return { value: Some(v), .. } => scan_expression(v, caps),
        Statement::Throw { value, .. } => scan_expression(value, caps),
        _ => {}
    }
}

fn scan_expression(expr: &Expression, caps: &mut CapabilitySet) {
    match expr {
        Expression::OduCall(call) => {
            scan_odu_call(call, caps);
        }
        Expression::BinaryOp { left, right, .. } => {
            scan_expression(left, caps);
            scan_expression(right, caps);
        }
        Expression::List(items) => {
            for i in items {
                scan_expression(i, caps);
            }
        }
        Expression::Map(entries) => {
            for (k, v) in entries {
                scan_expression(k, caps);
                scan_expression(v, caps);
            }
        }
        Expression::MethodCall { object, args, .. } => {
            scan_expression(object, caps);
            for arg in args {
                scan_expression(arg, caps);
            }
        }
        _ => {}
    }
}

fn extract_string_arg(args: &[Expression], index: usize) -> Option<String> {
    if let Some(Expression::String(s)) = args.get(index) {
        Some(s.clone())
    } else {
        None
    }
}
