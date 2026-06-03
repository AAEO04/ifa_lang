#![allow(clippy::collapsible_if)]

use ifa_babalawo::{
    BabalawoConfig, LintContext, Severity as IfaSeverity, analyze_program, list_methods_for_domain,
};
use ifa_vm::parse;
use lsp_server::{Connection, Message, Notification, RequestId, Response};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, Diagnostic, DiagnosticSeverity,
    InitializeParams, Position, PublishDiagnosticsParams, Range, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions, Url,
};
use std::error::Error;

/// Run the LSP server
pub fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    eprintln!("Starting Ifá-Lang LSP Server...");

    let (connection, io_threads) = Connection::stdio();

    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::FULL),
                ..Default::default()
            },
        )),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
            ..Default::default()
        }),
        code_action_provider: Some(lsp_types::CodeActionProviderCapability::Simple(true)),
        ..Default::default()
    })?;

    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    eprintln!("Ifá-Lang LSP Server shutting down.");
    Ok(())
}

fn main_loop(
    connection: Connection,
    params: serde_json::Value,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let _params: InitializeParams = serde_json::from_value(params)
        .map_err(|e| format!("Failed to parse InitializeParams: {}", e))?;
    eprintln!("Client connected!");

    // Track the latest valid analysis context
    let mut context: Option<LintContext> = None;
    let mut documents: std::collections::HashMap<Url, String> = std::collections::HashMap::new();

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                let req_msg = Message::Request(req);
                if let Ok((id, params)) =
                    cast_req::<lsp_types::request::Completion>(req_msg.clone())
                {
                    eprintln!(
                        "Got completion request for: {}",
                        params.text_document_position.text_document.uri
                    );
                    let doc_text = documents
                        .get(&params.text_document_position.text_document.uri)
                        .map(|s| s.as_str());
                    let result = Some(lsp_types::CompletionResponse::Array(get_completions(
                        &context,
                        doc_text,
                        params.text_document_position.position,
                    )));
                    let result = serde_json::to_value(&result)
                        .map_err(|e| format!("Failed to serialize completion: {}", e))?;
                    let resp = Response {
                        id,
                        result: Some(result),
                        error: None,
                    };
                    connection.sender.send(Message::Response(resp))?;
                } else if let Ok((id, params)) =
                    cast_req::<lsp_types::request::CodeActionRequest>(req_msg.clone())
                {
                    let mut actions = Vec::new();
                    for diagnostic in params.context.diagnostics {
                        if diagnostic.code
                            == Some(lsp_types::NumberOrString::String(
                                "UNUSED_VARIABLE".to_string(),
                            ))
                        {
                            if let Some(data) = &diagnostic.data {
                                if let Some(context_val) = data.get("context") {
                                    if let Some(name) = context_val.as_str() {
                                        let new_name = format!("_{}", name);

                                        let mut changes = std::collections::HashMap::new();
                                        changes.insert(
                                            params.text_document.uri.clone(),
                                            vec![lsp_types::TextEdit {
                                                range: diagnostic.range,
                                                new_text: new_name.clone(),
                                            }],
                                        );

                                        actions.push(lsp_types::CodeActionOrCommand::CodeAction(
                                            lsp_types::CodeAction {
                                                title: format!(
                                                    "Sanctify (prefix with _): {}",
                                                    new_name
                                                ),
                                                kind: Some(lsp_types::CodeActionKind::QUICKFIX),
                                                diagnostics: Some(vec![diagnostic]),
                                                edit: Some(lsp_types::WorkspaceEdit {
                                                    changes: Some(changes),
                                                    ..Default::default()
                                                }),
                                                is_preferred: Some(true),
                                                ..Default::default()
                                            },
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    let result = serde_json::to_value(&actions).unwrap_or(serde_json::Value::Null);
                    let resp = Response {
                        id,
                        result: Some(result),
                        error: None,
                    };
                    connection.sender.send(Message::Response(resp))?;
                } else {
                    // Unknown or unhandled request
                }
            }
            Message::Response(resp) => {
                eprintln!("Got response: {:?}", resp);
            }
            Message::Notification(not) => {
                if not.method == "exit" {
                    return Ok(());
                }
                match cast_not::<lsp_types::notification::DidOpenTextDocument>(
                    Message::Notification(not),
                ) {
                    Ok(params) => {
                        eprintln!("DidOpen: {}", params.text_document.uri);
                        documents.insert(
                            params.text_document.uri.clone(),
                            params.text_document.text.clone(),
                        );
                        if let Ok(Some(new_ctx)) = publish_diagnostics(
                            &connection,
                            params.text_document.uri,
                            &params.text_document.text,
                        ) {
                            context = Some(new_ctx);
                        }
                    }
                    Err(Message::Notification(not)) => {
                        match cast_not::<lsp_types::notification::DidChangeTextDocument>(
                            Message::Notification(not),
                        ) {
                            Ok(params) => {
                                eprintln!("DidChange: {}", params.text_document.uri);
                                if let Some(change) = params.content_changes.into_iter().next() {
                                    documents.insert(
                                        params.text_document.uri.clone(),
                                        change.text.clone(),
                                    );
                                    if let Ok(Some(new_ctx)) = publish_diagnostics(
                                        &connection,
                                        params.text_document.uri,
                                        &change.text,
                                    ) {
                                        context = Some(new_ctx);
                                    }
                                }
                            }
                            Err(Message::Notification(not)) => {
                                eprintln!("Unknown notification: {:?}", not);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn publish_diagnostics(
    connection: &Connection,
    uri: Url,
    text: &str,
) -> Result<Option<LintContext>, Box<dyn Error + Send + Sync>> {
    let mut diagnostics = Vec::new();
    let mut context = None;

    let source_lines: Vec<&str> = text.lines().collect();

    // 1. Parse Syntax
    match parse(text) {
        Ok(program) => {
            // 2. Run Babalawo Analyzer (get diagnostics + symbols)
            let (baba, ctx) = analyze_program(&program, uri.path(), BabalawoConfig::default());
            context = Some(ctx);

            for diag in baba.diagnostics {
                let severity = match diag.severity {
                    IfaSeverity::Error => DiagnosticSeverity::ERROR,
                    IfaSeverity::Warning => DiagnosticSeverity::WARNING,
                    IfaSeverity::Info => DiagnosticSeverity::INFORMATION,
                    IfaSeverity::Style => DiagnosticSeverity::HINT,
                };

                let range = if let Some(span) = &diag.error.span {
                    let start_line = span.line.saturating_sub(1);
                    let start_col = span.column.saturating_sub(1);
                    let len = (span.end as u32).saturating_sub(span.start as u32);

                    let max_col = if start_line < source_lines.len() {
                        source_lines[start_line].len() as u32
                    } else {
                        (start_col as u32) + len
                    };

                    let end_col = std::cmp::min((start_col as u32) + len, max_col);

                    Range {
                        start: Position {
                            line: start_line as u32,
                            character: start_col as u32,
                        },
                        end: Position {
                            line: start_line as u32,
                            character: end_col,
                        },
                    }
                } else {
                    let start_line = (diag.error.line).saturating_sub(1);
                    let start_col = (diag.error.column).saturating_sub(1);

                    let max_col = if start_line < source_lines.len() {
                        source_lines[start_line].len() as u32
                    } else {
                        (start_col + 5) as u32
                    };
                    let end_col = std::cmp::min((start_col + 5) as u32, max_col);

                    Range {
                        start: Position {
                            line: start_line as u32,
                            character: start_col as u32,
                        },
                        end: Position {
                            line: start_line as u32,
                            character: end_col,
                        },
                    }
                };

                let message = if let Some(wisdom) = &diag.wisdom {
                    format!("[{}] {} (Wisdom: {})", diag.odu, diag.error.message, wisdom)
                } else {
                    format!("[{}] {}", diag.odu, diag.error.message)
                };

                let data = if let Some(ctx_val) = diag.error.context {
                    Some(serde_json::json!({ "context": ctx_val }))
                } else {
                    None
                };

                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(severity),
                    code: Some(lsp_types::NumberOrString::String(diag.error.code)),
                    code_description: None,
                    source: Some("ifa-babalawo".to_string()),
                    message,
                    related_information: None,
                    tags: None,
                    data,
                });
            }
        }
        Err(e) => {
            let msg = e.to_string();

            // Try to extract line and column from Pest error string
            let mut line = 0;
            let mut column = 0;

            if let Some(idx) = msg.find("--> ") {
                let rest = &msg[idx + 4..];
                if let Some(nl) = rest.find('\n') {
                    let loc_str = &rest[..nl];
                    let parts: Vec<&str> = loc_str.split(':').collect();
                    if parts.len() == 2 {
                        if let Ok(l) = parts[0].trim().parse::<u32>() {
                            line = l.saturating_sub(1);
                        }
                        if let Ok(c) = parts[1].trim().parse::<u32>() {
                            column = c.saturating_sub(1);
                        }
                    }
                }
            }

            let end_col = if (line as usize) < source_lines.len() {
                source_lines[line as usize].len() as u32
            } else {
                column + 1
            };

            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line,
                        character: column,
                    },
                    end: Position {
                        line,
                        character: end_col,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: None,
                code_description: None,
                source: Some("ifa-parser".to_string()),
                message: msg,
                related_information: None,
                tags: None,
                data: None,
            });
        }
    }

    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    };
    let not = Notification {
        method: "textDocument/publishDiagnostics".to_string(),
        params: serde_json::to_value(&params).unwrap_or(serde_json::Value::Null),
    };
    connection.sender.send(Message::Notification(not))?;
    Ok(context)
}

fn get_completions(
    context: &Option<LintContext>,
    doc_text: Option<&str>,
    pos: Position,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Check if we are completing after a module (e.g. Obara.)
    if let Some(text) = doc_text {
        let lines: Vec<&str> = text.lines().collect();
        if (pos.line as usize) < lines.len() {
            let line = lines[pos.line as usize];
            let before_cursor = &line[..std::cmp::min(pos.character as usize, line.len())];

            if let Some(dot_idx) = before_cursor.rfind('.') {
                let domain_str = before_cursor[..dot_idx]
                    .trim()
                    .split(|c: char| !c.is_alphanumeric())
                    .last();
                if let Some(domain_name) = domain_str {
                    let odu_opt = match domain_name {
                        "Ogbe" => Some(ifa_babalawo::Odu::Ogbe),
                        "Oyeku" => Some(ifa_babalawo::Odu::Oyeku),
                        "Iwori" => Some(ifa_babalawo::Odu::Iwori),
                        "Odi" => Some(ifa_babalawo::Odu::Odi),
                        "Irosu" => Some(ifa_babalawo::Odu::Irosu),
                        "Owonrin" => Some(ifa_babalawo::Odu::Owonrin),
                        "Obara" => Some(ifa_babalawo::Odu::Obara),
                        "Okanran" => Some(ifa_babalawo::Odu::Okanran),
                        "Ogunda" => Some(ifa_babalawo::Odu::Ogunda),
                        "Osa" => Some(ifa_babalawo::Odu::Osa),
                        "Ika" => Some(ifa_babalawo::Odu::Ika),
                        "Oturupon" => Some(ifa_babalawo::Odu::Oturupon),
                        "Otura" => Some(ifa_babalawo::Odu::Otura),
                        "Irete" => Some(ifa_babalawo::Odu::Irete),
                        "Ose" => Some(ifa_babalawo::Odu::Ose),
                        "Ofun" => Some(ifa_babalawo::Odu::Ofun),
                        _ => None,
                    };
                    if let Some(odu) = odu_opt {
                        let methods = list_methods_for_domain(&odu);
                        if !methods.is_empty() {
                            for method_name in methods {
                                items.push(CompletionItem {
                                    label: method_name.to_string(),
                                    detail: Some(format!("fn {}()", method_name)),
                                    kind: Some(CompletionItemKind::METHOD),
                                    ..Default::default()
                                });
                            }
                            // Return early if we matched a domain correctly
                            return items;
                        }
                    }
                }
            }
        }
    }

    // Yoruba Keywords
    let yoruba_keywords = [
        ("gbiyanju", "Try block"),
        ("gba", "Catch block"),
        ("nipari", "Finally block"),
        ("ayanfe", "Constant"),
        ("daro", "Async function"),
        ("reti", "Await"),
        ("ta", "Throw error"),
        ("jowo", "Yield"),
        ("ofo", "Null"),
        ("ohunkohun", "Any"),
        ("fun", "Function definition (fn)"),
        ("ninu", "Loop (in/for)"),
        ("ti", "Conditional (if)"),
        ("tabi", "Else (else)"),
        ("pada", "Return values"),
        ("ailewu", "Unsafe block"),
        ("da", "Break loop"),
        ("tesiwaju", "Continue loop"),
        ("lati", "From"),
        ("abo", "Strict module"),
        ("fi", "Export (pub)"),
        ("ikoko", "Private"),
        ("gbangba", "Public"),
        ("ese", "Function definition"),
        ("ewo", "Assert/Taboo check"),
    ];

    for (k, desc) in yoruba_keywords.iter() {
        items.push(ci(k, desc, CompletionItemKind::KEYWORD));
    }

    // English Keywords
    let english_keywords = [
        ("try", "Try block"),
        ("catch", "Catch block"),
        ("finally", "Finally block"),
        ("const", "Constant"),
        ("async", "Async function"),
        ("await", "Await"),
        ("throw", "Throw error"),
        ("yield", "Yield"),
        ("null", "Null"),
        ("nil", "Nil"),
        ("fn", "Function definition"),
        ("in", "Loop iterator"),
        ("if", "Conditional"),
        ("else", "Else"),
        ("return", "Return values"),
        ("unsafe", "Unsafe block"),
        ("break", "Break loop"),
        ("continue", "Continue loop"),
        ("from", "From"),
        ("strict", "Strict module"),
        ("pub", "Export (pub)"),
        ("export", "Export"),
        ("private", "Private"),
        ("public", "Public"),
        ("assert", "Assert check"),
        ("taboo", "Taboo check"),
        ("defer", "Defer cleanup block"),
        ("let", "Variable declaration"),
        ("while", "While loop"),
        ("for", "For loop"),
        ("import", "Import module"),
        ("match", "Match expression"),
    ];

    for (k, desc) in english_keywords.iter() {
        items.push(ci(k, desc, CompletionItemKind::KEYWORD));
    }

    // Types
    let types = [
        "Int", "Float", "Str", "Bool", "List", "Map", "Any", "void", "i8", "i16", "i32", "i64",
        "u8", "u16", "u32", "u64", "f32", "f64",
    ];
    for t in types.iter() {
        items.push(ci(t, "Type", CompletionItemKind::CLASS));
    }

    // Modules (Odu) (Static)
    let modules = [
        ("Ogbe", "The Light (System/Lifecycle)"),
        ("Oyeku", "The Darkness (Exit/Cleanup)"),
        ("Iwori", "The Mirror (Time/Loops)"),
        ("Odi", "The Vessel (File I/O)"),
        ("Irosu", "The Speaker (Log/Print)"),
        ("Owonrin", "The Chaotic (Random)"),
        ("Obara", "The King (Math)"),
        ("Okanran", "The Troublemaker (Errors)"),
        ("Ogunda", "The Cutter (Arrays)"),
        ("Osa", "The Wind (Flow/Concurrency)"),
        ("Ika", "The Constrictor (Strings)"),
        ("Oturupon", "The Bearer (Reduce/Div)"),
        ("Otura", "The Messenger (Network)"),
        ("Irete", "The Crusher (Crypto)"),
        ("Ose", "The Beautifier (UI/Graphics)"),
        ("Ofun", "The Creator (Root/Perms)"),
    ];
    for (m, desc) in modules.iter() {
        items.push(ci(m, desc, CompletionItemKind::MODULE));
    }

    // Std Functions (Static)
    let std_fns = [
        ("ka", "Read (read)"),
        ("ko", "Write (write)"),
        ("so", "Speak/Print (print)"),
        ("gbo", "Listen/Input (input)"),
        ("sun", "Sleep (sleep)"),
        ("ji", "Wake/Start"),
        ("mo", "Clean/Clear"),
        ("ya", "Draw/Render"),
        ("pin", "Divide/Split"),
    ];
    for (f, desc) in std_fns.iter() {
        items.push(ci(f, desc, CompletionItemKind::FUNCTION));
    }

    // Dynamic Completions from Context
    if let Some(ctx) = context {
        for var in ctx.defined_vars.keys() {
            let detail = if let Some(type_hint) = ctx.get_var_type(var) {
                format!("Variable: {:?}", type_hint)
            } else {
                "Variable".to_string()
            };

            items.push(ci(var, &detail, CompletionItemKind::VARIABLE));
        }
    }

    items
}

fn ci(label: &str, detail: &str, kind: CompletionItemKind) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        detail: Some(detail.to_string()),
        kind: Some(kind),
        ..Default::default()
    }
}

fn cast_req<R>(req: Message) -> Result<(RequestId, R::Params), Message>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    match req {
        Message::Request(req) if req.method == R::METHOD => {
            let params = serde_json::from_value(req.params.clone())
                .map_err(|_| Message::Request(req.clone()))?;
            Ok((req.id, params))
        }
        _ => Err(req),
    }
}

fn cast_not<N>(not: Message) -> Result<N::Params, Message>
where
    N: lsp_types::notification::Notification,
    N::Params: serde::de::DeserializeOwned,
{
    match not {
        Message::Notification(not) if not.method == N::METHOD => {
            let params = serde_json::from_value(not.params.clone())
                .map_err(|_| Message::Notification(not.clone()))?;
            Ok(params)
        }
        _ => Err(not),
    }
}
