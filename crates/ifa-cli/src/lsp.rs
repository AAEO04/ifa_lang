use ifa_babalawo::{Severity as IfaSeverity, check_program};
use ifa_core::parse;
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
            work_done_progress_options: Default::default(),
            all_commit_characters: None,
            completion_item: None,
        }),
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

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                match cast_req::<lsp_types::request::Completion>(Message::Request(req)) {
                    Ok((id, params)) => {
                        eprintln!(
                            "Got completion request for: {}",
                            params.text_document_position.text_document.uri
                        );
                        let result = Some(lsp_types::CompletionResponse::Array(get_completions()));
                        let result = serde_json::to_value(&result)
                            .map_err(|e| format!("Failed to serialize completion: {}", e))?;
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;
                    }
                    Err(Message::Request(req)) => {
                        eprintln!("Unknown request: {:?}", req);
                    }
                    _ => {}
                }
            }
            Message::Response(resp) => {
                eprintln!("Got response: {:?}", resp);
            }
            Message::Notification(not) => {
                match cast_not::<lsp_types::notification::DidOpenTextDocument>(
                    Message::Notification(not),
                ) {
                    Ok(params) => {
                        eprintln!("DidOpen: {}", params.text_document.uri);
                        publish_diagnostics(
                            &connection,
                            params.text_document.uri,
                            &params.text_document.text,
                        )?;
                    }
                    Err(Message::Notification(not)) => {
                        match cast_not::<lsp_types::notification::DidChangeTextDocument>(
                            Message::Notification(not),
                        ) {
                            Ok(params) => {
                                eprintln!("DidChange: {}", params.text_document.uri);
                                if let Some(change) = params.content_changes.into_iter().next() {
                                    publish_diagnostics(
                                        &connection,
                                        params.text_document.uri,
                                        &change.text,
                                    )?;
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
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut diagnostics = Vec::new();

    // 1. Parse Syntax
    match parse(text) {
        Ok(program) => {
            // 2. Run Babalawo Linter
            let baba = check_program(&program, uri.path());

            for diag in baba.diagnostics {
                let severity = match diag.severity {
                    IfaSeverity::Error => DiagnosticSeverity::ERROR,
                    IfaSeverity::Warning => DiagnosticSeverity::WARNING,
                    IfaSeverity::Info => DiagnosticSeverity::INFORMATION,
                    IfaSeverity::Style => DiagnosticSeverity::HINT,
                };

                let range = Range {
                    start: Position {
                        line: (diag.error.line).saturating_sub(1) as u32,
                        character: (diag.error.column).saturating_sub(1) as u32,
                    },
                    end: Position {
                        line: (diag.error.line).saturating_sub(1) as u32,
                        character: (diag.error.column + 5) as u32,
                    }, // Approx length
                };

                let message = if let Some(wisdom) = &diag.wisdom {
                    format!("[{}] {} (Wisdom: {})", diag.odu, diag.error.message, wisdom)
                } else {
                    format!("[{}] {}", diag.odu, diag.error.message)
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
                    data: None,
                });
            }
        }
        Err(e) => {
            // Simple syntax error handling
            // Note: IfaError display format is typically "Error at line L: Msg" or similar
            // For robust LSP we should parse the error or have parser return structured error.
            // Using a fallback for now.
            let msg = e.to_string();
            // Try to extract line? simplistic heuristic
            let line = 0;

            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position { line, character: 0 },
                    end: Position { line, character: 1 },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: None,
                code_description: None,
                source: Some("ifa-core".to_string()),
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
    Ok(())
}

fn get_completions() -> Vec<CompletionItem> {
    vec![
        // Keywords
        ci(
            "fun",
            "Function definition (fn)",
            CompletionItemKind::KEYWORD,
        ),
        ci("ninu", "Loop (in/for)", CompletionItemKind::KEYWORD),
        ci("ti", "Conditional (if)", CompletionItemKind::KEYWORD),
        ci("tabi", "Else (else)", CompletionItemKind::KEYWORD),
        ci("se", "Do/Execute", CompletionItemKind::KEYWORD),
        ci(
            "gbe",
            "Variable declaration (let)",
            CompletionItemKind::KEYWORD,
        ),
        ci("pada", "Return values", CompletionItemKind::KEYWORD),
        ci("nla", "Large / True", CompletionItemKind::KEYWORD),
        ci("kekere", "Small / False", CompletionItemKind::KEYWORD),
        // Modules (Odu)
        ci(
            "Ogbe",
            "The Supporter (Lifecycle)",
            CompletionItemKind::MODULE,
        ),
        ci(
            "Oyeku",
            "The Mother (Death/Exit)",
            CompletionItemKind::MODULE,
        ),
        ci(
            "Iwori",
            "The Viewer (Time/Date)",
            CompletionItemKind::MODULE,
        ),
        ci("Odi", "The Sealer (Files/IO)", CompletionItemKind::MODULE),
        ci("Irosu", "The Sound (Log/Print)", CompletionItemKind::MODULE),
        ci(
            "Owonrin",
            "The Reverse (Random)",
            CompletionItemKind::MODULE,
        ),
        ci("Obara", "The Resting (Math +)", CompletionItemKind::MODULE),
        ci(
            "Okanran",
            "The Striker (Strings)",
            CompletionItemKind::MODULE,
        ),
        ci("Ogunda", "The Creator (Arrays)", CompletionItemKind::MODULE),
        ci(
            "Osa",
            "The Spirit (Concurrency)",
            CompletionItemKind::MODULE,
        ),
        ci(
            "Ika",
            "The Controller (Control)",
            CompletionItemKind::MODULE,
        ),
        ci(
            "Oturupon",
            "The Bearer (Math -)",
            CompletionItemKind::MODULE,
        ),
        ci("Otura", "The Vision (Network)", CompletionItemKind::MODULE),
        ci("Irete", "The Crusher (Crypto)", CompletionItemKind::MODULE),
        ci("Ose", "The Conqueror (UI/Docs)", CompletionItemKind::MODULE),
        ci(
            "Ofun",
            "The Giver (Permissions)",
            CompletionItemKind::MODULE,
        ),
        // Std Functions
        ci("ka", "Read (read)", CompletionItemKind::FUNCTION),
        ci("ko", "Write (write)", CompletionItemKind::FUNCTION),
        ci("so", "Speak/Print (print)", CompletionItemKind::FUNCTION),
        ci("gbo", "Listen/Input (input)", CompletionItemKind::FUNCTION),
        ci("sun", "Sleep (sleep)", CompletionItemKind::FUNCTION),
        ci("ji", "Wake/Start", CompletionItemKind::FUNCTION),
        ci("mo", "Clean/Clear", CompletionItemKind::FUNCTION),
        ci("ya", "Draw/Render", CompletionItemKind::FUNCTION),
        ci("roi", "Report/Log", CompletionItemKind::FUNCTION),
        ci("pin", "Divide/Split", CompletionItemKind::FUNCTION),
        ci("dapo", "Join/Merge", CompletionItemKind::FUNCTION),
    ]
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
