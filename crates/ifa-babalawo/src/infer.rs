
use ifa_core::ast::{Expression, Program, Statement};
use ifa_sandbox::{CapabilitySet, Ofun};
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

fn scan_statement(stmt: &Statement, caps: &mut CapabilitySet) {
    match stmt {
        Statement::Instruction { call, .. } => {
            let domain = format!("{:?}", call.domain).to_lowercase();
            match domain.as_str() {
                "odi" => {
                    // File I/O
                    // Odi.ka("file") -> Read
                    // Odi.ko("file") -> Write
                    if call.method == "ka" || call.method == "read" {
                         if let Some(path) = extract_string_arg(&call.args, 0) {
                             caps.grant(Ofun::ReadFiles { root: PathBuf::from(path) });
                         } else {
                             // If dynamic path, grant generous permission or warn?
                             // zero-config philosophy -> broad permission if unknown
                             caps.grant(Ofun::ReadFiles { root: PathBuf::from("/") }); 
                         }
                    } else if call.method == "ko" || call.method == "write" {
                        if let Some(path) = extract_string_arg(&call.args, 0) {
                             caps.grant(Ofun::WriteFiles { root: PathBuf::from(path) });
                         } else {
                             caps.grant(Ofun::WriteFiles { root: PathBuf::from("/") }); 
                         }
                    }
                },
                "otura" => {
                    // Networking
                    caps.grant(Ofun::Network { domains: vec!["*".to_string()] });
                },
                "ogunda" => {
                    // Processes
                    if call.method == "bẹrẹ" || call.method == "run" {
                        // Spawning processes requires capabilities
                        // How is this represented in Ofun? Need to check Capability definitions.
                        // Assuming Ofun::Command or similar checking Ofun enum.
                    }
                },
                "iwori" => {
                     if call.method == "akoko" || call.method == "isisinyi" {
                         caps.grant(Ofun::Time);
                     }
                },
                "owonrin" => {
                    caps.grant(Ofun::Random);
                },
                "ogbe" => {
                    if call.method == "ayika" || call.method == "env" {
                        caps.grant(Ofun::Environment { keys: vec!["*".to_string()] });
                    }
                }
                _ => {}
            }

            for arg in &call.args {
                scan_expression(arg, caps);
            }
        },
        Statement::VarDecl { value, .. } => scan_expression(value, caps),
        Statement::Assignment { value, .. } => scan_expression(value, caps),
        Statement::If { condition, then_body, else_body, .. } => {
            scan_expression(condition, caps);
            for s in then_body { scan_statement(s, caps); }
            if let Some(else_stmts) = else_body {
                for s in else_stmts { scan_statement(s, caps); }
            }
        },
        Statement::While { condition, body, .. } => {
            scan_expression(condition, caps);
            for s in body { scan_statement(s, caps); }
        },
        Statement::For { iterable, body, .. } => {
             scan_expression(iterable, caps);
             for s in body { scan_statement(s, caps); }
        },
        Statement::EseDef { body, .. } => {
             for s in body { scan_statement(s, caps); }
        },
         Statement::OduDef { body, .. } => {
             for s in body { scan_statement(s, caps); }
        },
        Statement::Return { value: Some(v), .. } => scan_expression(v, caps),
        _ => {}
    }
}

fn scan_expression(expr: &Expression, caps: &mut CapabilitySet) {
    match expr {
        Expression::OduCall(call) => {
             // Re-use logic for instructions but for expr calls (e.g. nested calls)
             // Copy-paste logic from scan_instruction for now or refactor
             // For OduCall in Expression, it's the exact same structure as Instruction's OduCall
             // We can synthesise a Statement::Instruction check, or just extract logic.
             // Simpler: Just recursively check args.
             
             let domain = format!("{:?}", call.domain).to_lowercase();
             if domain == "odi" {
                  // If reading file in expression
                  if call.method == "ka" || call.method == "read" {
                       if let Some(path) = extract_string_arg(&call.args, 0) {
                            caps.grant(Ofun::ReadFiles { root: PathBuf::from(path) });
                        } else {
                            caps.grant(Ofun::ReadFiles { root: PathBuf::from("/") }); 
                        }
                  }
             }
             // ... other domains
             
             for arg in &call.args {
                scan_expression(arg, caps);
            }
        },
        Expression::BinaryOp { left, right, .. } => {
            scan_expression(left, caps);
            scan_expression(right, caps);
        },
        Expression::List(items) => {
            for i in items { scan_expression(i, caps); }
        },
        Expression::Map(entries) => {
            for (k, v) in entries {
                scan_expression(k, caps);
                scan_expression(v, caps);
            }
        },
        Expression::MethodCall { object, args, .. } => {
            scan_expression(object, caps);
            for arg in args { scan_expression(arg, caps); }
        },
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
