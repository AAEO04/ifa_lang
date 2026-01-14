use lsp_server::{Connection, Message, RequestId, Response, Notification};
use lsp_types::{
    ServerCapabilities, InitializeParams, TextDocumentSyncKind, TextDocumentSyncCapability,
    TextDocumentSyncOptions, CompletionOptions, CompletionItem, CompletionItemKind,
    PublishDiagnosticsParams, Diagnostic, DiagnosticSeverity, Range, Position,
    Url,
};
use std::error::Error;
use ifa_core::parse;

/// Run the LSP server
pub fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    eprintln!("Starting Ifá-Lang LSP Server...");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger shutdown request).
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::FULL), 
                ..Default::default()
            }
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
                        eprintln!("Got completion request for: {}", params.text_document_position.text_document.uri);
                        let result = Some(lsp_types::CompletionResponse::Array(vec![
                            completion_item("fun", "Function definition (fún)", CompletionItemKind::KEYWORD),
                            completion_item("ninu", "Loop (nínú)", CompletionItemKind::KEYWORD),
                            completion_item("ti", "Conditional (tí)", CompletionItemKind::KEYWORD),
                            completion_item("tabi", "Else (tàbí)", CompletionItemKind::KEYWORD),
                            completion_item("se", "Do/Execute (ṣe)", CompletionItemKind::KEYWORD),
                            completion_item("gbe", "Variable declaration (gbé)", CompletionItemKind::KEYWORD),
                            completion_item("pads", "Return (padà)", CompletionItemKind::KEYWORD),
                            completion_item("Ogbè", "The Supporter (System)", CompletionItemKind::MODULE),
                            completion_item("Ọyẹ̀kú", "The Mother (Exit/Sleep)", CompletionItemKind::MODULE),
                            completion_item("Ìwòrì", "The Viewer (Time)", CompletionItemKind::MODULE),
                            completion_item("Ìrosùn", "The Sound (Sound)", CompletionItemKind::MODULE),
                        ]));
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
                 match cast_not::<lsp_types::notification::DidOpenTextDocument>(Message::Notification(not)) {
                    Ok(params) => {
                        eprintln!("DidOpen: {}", params.text_document.uri);
                        publish_diagnostics(&connection, params.text_document.uri, &params.text_document.text)?;
                    }
                    Err(Message::Notification(not)) => match cast_not::<lsp_types::notification::DidChangeTextDocument>(Message::Notification(not)) {
                        Ok(params) => {
                             eprintln!("DidChange: {}", params.text_document.uri);
                             // Since we requested FULL sync, content_changes[0].text is the full content
                             if let Some(change) = params.content_changes.into_iter().next() {
                                 publish_diagnostics(&connection, params.text_document.uri, &change.text)?;
                             }
                        }
                        Err(Message::Notification(not)) => {
                             eprintln!("Unknown notification: {:?}", not);
                        }
                        _ => {}
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn publish_diagnostics(connection: &Connection, uri: Url, text: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut diagnostics = Vec::new();

    // Use ifa_core parser to check syntax
    if let Err(e) = parse(text) {
        // e is IfaError, which might be SyntaxError
        // We need to extract line/col from the error message or error type if available.
        // Currently IfaError formatting is string-based. A real implementation would expose line/col struct.
        // For now, we'll try to parse the default formatted string or just put it on line 1.
        
        // Assuming parsing error text handling. 
        // In a real implementation: `e` should have `line` and `column`. 
        // Let's assume raw string for now and default to 0.
        
        let message = e.to_string();
        
        let diagnostic = Diagnostic {
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 1 }, 
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: Some("ifa-lsp".to_string()),
            message,
            related_information: None,
            tags: None,
            data: None,
        };
        diagnostics.push(diagnostic);
    }

    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    };
    let not = Notification {
        method: "textDocument/publishDiagnostics".to_string(),
        params: serde_json::to_value(&params)
            .unwrap_or_else(|_| serde_json::Value::Null),
    };
    connection.sender.send(Message::Notification(not))?;
    Ok(())
}

fn completion_item(label: &str, detail: &str, kind: CompletionItemKind) -> CompletionItem {
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
