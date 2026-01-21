use ifa_core::ast::Statement;
use ifa_core::interpreter::{CapabilitySet, Debugger, Environment, Interpreter, Ofun};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, BufRead, Read, Write};
use std::sync::{Arc, Mutex};

// =============================================================================
// Minimal DAP Types
// =============================================================================

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum ProtocolMessage {
    #[serde(rename = "request")]
    Request(Request),
    #[serde(rename = "response")]
    Response(Response),
    #[serde(rename = "event")]
    Event(Event),
}

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    seq: i64,
    command: String,
    arguments: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    seq: i64,
    request_seq: i64,
    success: bool,
    command: String,
    message: Option<String>,
    body: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Event {
    seq: i64,
    event: String,
    body: Option<serde_json::Value>,
}

// =============================================================================
// Debugger State
// =============================================================================

#[derive(Debug)]
struct Breakpoint {
    line: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum StopReason {
    Step,
    Breakpoint,
    Entry,
}

#[derive(Debug)]
pub struct DapAdapter {
    breakpoints: Arc<Mutex<HashMap<String, Vec<Breakpoint>>>>,
    paused: Arc<Mutex<bool>>,
    stop_reason: Arc<Mutex<Option<StopReason>>>,
    // IO channels for DAP communication
    seq: Arc<Mutex<i64>>,
}

impl DapAdapter {
    pub fn new() -> Self {
        Self {
            breakpoints: Arc::new(Mutex::new(HashMap::new())),
            paused: Arc::new(Mutex::new(true)), // Start paused to wait for config
            stop_reason: Arc::new(Mutex::new(Some(StopReason::Entry))),
            seq: Arc::new(Mutex::new(1)),
        }
    }

    fn next_seq(&self) -> i64 {
        let mut seq = self.seq.lock().unwrap();
        *seq += 1;
        *seq
    }

    fn send_event(&self, event: String, body: Option<serde_json::Value>) {
        let msg = ProtocolMessage::Event(Event {
            seq: self.next_seq(),
            event,
            body,
        });
        self.send_message(&msg);
    }

    fn send_message(&self, msg: &ProtocolMessage) {
        let json = serde_json::to_string(msg).unwrap();
        let content_length = json.len();
        print!("Content-Length: {}\r\n\r\n{}", content_length, json);
        io::stdout().flush().unwrap();
        // Log to stderr for debugging the debugger
        // eprintln!("[DAP-OUT] {}", json);
    }

    fn wait_for_command(&self) {
        let stdin = io::stdin();
        let mut handle = stdin.lock();

        loop {
            // Check if we should unpause
            if !*self.paused.lock().unwrap() {
                break;
            }

            // Read header
            let mut header = String::new();
            if handle.read_line(&mut header).unwrap() == 0 {
                break; // EOF
            }

            // eprintln!("[DAP-IN-HEADER] {}", header.trim());

            if header.starts_with("Content-Length: ") {
                let len_str = header.trim().trim_start_matches("Content-Length: ");
                let len: usize = len_str.parse().unwrap();

                // Read empty line
                let mut empty = String::new();
                handle.read_line(&mut empty).unwrap();

                // Read body
                let mut buffer = vec![0; len];
                handle.read_exact(&mut buffer).unwrap();
                let body = String::from_utf8(buffer).unwrap();

                // eprintln!("[DAP-IN-BODY] {}", body);

                if let Ok(ProtocolMessage::Request(req)) = serde_json::from_str::<ProtocolMessage>(&body) {
                    self.handle_request(req);
                }
            }
        }
    }

    fn handle_request(&self, req: Request) {
        let success = true;
        let message = None;
        let mut body = None;

        match req.command.as_str() {
            "initialize" => {
                body = Some(serde_json::json!({
                    "supportsConfigurationDoneRequest": true,
                    "supportsFunctionBreakpoints": false,
                    "supportsConditionalBreakpoints": false
                }));
                self.send_event("initialized".to_string(), None);
            }
            "launch" => {
                // Launch happens outside, we just ack
                self.send_event(
                    "process".to_string(),
                    Some(serde_json::json!({
                        "name": "ifa-debug",
                        "systemProcessId": std::process::id()
                    })),
                );
            }
            "setBreakpoints" => {
                let args = req.arguments.as_ref().unwrap();
                let path = args["source"]["path"].as_str().unwrap_or("").to_string();
                let bps = args["breakpoints"].as_array();

                let mut final_bps = Vec::new();
                let mut confirmed_bps = Vec::new();

                if let Some(list) = bps {
                    for bp in list {
                        let line = bp["line"].as_u64().unwrap() as usize;
                        final_bps.push(Breakpoint { line });
                        confirmed_bps.push(serde_json::json!({
                            "verified": true,
                            "line": line
                        }));
                    }
                }

                self.breakpoints.lock().unwrap().insert(path, final_bps);
                body = Some(serde_json::json!({ "breakpoints": confirmed_bps }));
            }
            "configurationDone" => {
                // Ready to start
                *self.paused.lock().unwrap() = false;
            }
            "threads" => {
                body = Some(serde_json::json!({
                    "threads": [
                        { "id": 1, "name": "main" }
                    ]
                }));
            }
            "stackTrace" => {
                // Minimal stack trace (just current location if available?)
                // Since we don't have easy access to the Interpreter stack here without deeper changes,
                // we'll return a stub or empty frame.
                // NOTE: Real implementation needs Interpreter state exposure.
                // For now, on_statement will likely need to store current stack info in the adapter
                // before pausing.

                // We'll rely on the "Stopped" event details for location, but VS Code calls this for the list.
                // We will implement a simple "Current Statement" frame.
                body = Some(serde_json::json!({
                    "stackFrames": [],
                    "totalFrames": 0
                }));
            }
            "scopes" => {
                body = Some(serde_json::json!({
                    "scopes": [
                        { "name": "Locals", "variablesReference": 1, "expensive": false },
                        { "name": "Globals", "variablesReference": 2, "expensive": false }
                    ]
                }));
            }
            "variables" => {
                // TODO: Implement variable inspection
                // Requires capturing env.
                body = Some(serde_json::json!({ "variables": [] }));
            }
            "continue" => {
                *self.paused.lock().unwrap() = false;
                *self.stop_reason.lock().unwrap() = None;
            }
            "next" => {
                // Step Over
                *self.paused.lock().unwrap() = false;
                *self.stop_reason.lock().unwrap() = Some(StopReason::Step);
            }
            "disconnect" => {
                std::process::exit(0);
            }
            _ => {
                // success = false;
                // message = Some("Not implemented".to_string());
            }
        }

        let resp = ProtocolMessage::Response(Response {
            seq: self.next_seq(),
            request_seq: req.seq,
            success,
            command: req.command,
            message,
            body,
        });
        self.send_message(&resp);
    }
}

impl Debugger for DapAdapter {
    fn on_statement(&mut self, stmt: &Statement, _env: &Environment) {
        // extract location
        let (line, _file) = match stmt {
            Statement::VarDecl { span, .. }
            | Statement::Assignment { span, .. }
            | Statement::Instruction { span, .. }
            | Statement::If { span, .. }
            | Statement::While { span, .. }
            | Statement::For { span, .. }
            | Statement::Return { span, .. }
            | Statement::Expr { span, .. }
            | Statement::Import { span, .. }
            | Statement::Ebo { span, .. }
            | Statement::Ewo { span, .. }
            | Statement::Opon { span, .. }
            | Statement::Taboo { span, .. }
            | Statement::Match { span, .. }
            | Statement::Ase { span } => (span.line, "unknown.ifa"), // TODO: Span needs file path? Or assume single file for simple case
            _ => (0, ""),
        };

        // Check breakpoints
        // Note: Filename matching is tricky if we don't have full paths in Span.
        // We'll trust file name if span has it, or valid breakpoint globally if line matches?
        // Let's assume naive line match for now.

        // Logic:
        // 1. Check if we should stop (breakpoints or step mode)
        // 2. If yes, send Stopped event and enter wait_for_command loop

        let should_break = {
            let bps = self.breakpoints.lock().unwrap();
            // Iterate all files for now or assume active debug file
            bps.values().flatten().any(|bp| bp.line == line)
        };

        let stop_reason = self.stop_reason.lock().unwrap().clone();

        let should_stop = should_break || matches!(stop_reason, Some(StopReason::Step));

        if should_stop {
            *self.paused.lock().unwrap() = true;

            // Send Stopped Event
            self.send_event(
                "stopped".to_string(),
                Some(serde_json::json!({
                    "reason": if should_break { "breakpoint" } else { "step" },
                    "threadId": 1,
                    "allThreadsStopped": true
                })),
            );

            // Wait
            self.wait_for_command();
        }
    }
}

pub fn run_debug_session(file: std::path::PathBuf) -> color_eyre::Result<()> {
    // 1. Initialize Adapter
    let adapter = DapAdapter::new();
    let _adapter_box = Box::new(adapter);

    // We need to keep a reference to interact with main loop?
    // Actually the adapter implements Debugger which has the loop inside on_statement.
    // But we need to handle the initial handshake (initialize, launch) BEFORE starting execution.
    // The adapter needs to be mutable to be passed to interpreter properly?
    // Or we need shared state. DapAdapter uses Arc<Mutex> for state, so cloning it is fine.

    // Hack: We need to run the initial handshake loop *before* we pass control to Interpreter.execute
    // We'll create a clone for the Interpreter.

    // Wait, Debugger trait requires `&mut self`.
    // And on_statement is called by Interpreter.
    // So the Interpreter owns the Debugger.

    // Let's perform initial handshake here using a temporary instance or just raw processing?
    // Better: The adapter inside the interpreter will handle runtime events.
    // But we need to handle `initialize` and `launch`/`attach` configuration BEFORE we start `interpreter.execute`.

    // Let's construct the adapter.
    let adapter = DapAdapter::new();

    // Validating handshake
    adapter.wait_for_command(); // Will wait until configurationDone (paused = false)

    // Setup Interpreter
    let source = std::fs::read_to_string(&file)?;
    let program = ifa_core::parse(&source)?;

    let mut interpreter = Interpreter::with_file(&file);
    interpreter.set_capabilities(CapabilitySet::new()); // Full permissions for debug?
    interpreter.capabilities.grant(Ofun::Stdio);

    // We need to pass the adapter to the interpreter.
    // Because Debugger trait takes `&mut self`, `adapter` must be moved.
    interpreter.set_debugger(Box::new(adapter));

    interpreter
        .execute(&program)
        .map_err(|e| color_eyre::eyre::eyre!("Runtime error: {}", e))?;

    Ok(())
}
