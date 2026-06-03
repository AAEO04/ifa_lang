//! DAP Adapter
//! Currently being migrated to the new Bytecode VM.
//! This stub implements just enough of the JSON-RPC DAP to gracefully fail
//! and tell VS Code what's going on, instead of crashing.

use serde_json::{Value, json};
use std::io::{self, BufRead, Read, Write};

pub fn run_debug_session(_file: std::path::PathBuf) -> color_eyre::Result<()> {
    eprintln!("Starting Ifá-Lang DAP Server (Stub)...");

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut stdout = io::stdout();

    loop {
        // Read headers
        let mut content_length = 0;
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => return Ok(()), // EOF
                Ok(_) => {
                    let line = line.trim();
                    if line.is_empty() {
                        break;
                    }
                    if line.to_lowercase().starts_with("content-length:") {
                        let parts: Vec<&str> = line.split(':').collect();
                        if parts.len() == 2 {
                            if let Ok(len) = parts[1].trim().parse::<usize>() {
                                content_length = len;
                            }
                        }
                    }
                }
                Err(_) => return Ok(()),
            }
        }

        if content_length == 0 {
            continue;
        }

        // Read payload
        let mut body = vec![0; content_length];
        if reader.read_exact(&mut body).is_err() {
            return Ok(());
        }

        let request: Value = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(type_val) = request.get("type") {
            if type_val == "request" {
                if let Some(command) = request.get("command").and_then(|v| v.as_str()) {
                    let seq = request.get("seq").unwrap_or(&json!(0)).clone();

                    if command == "initialize" {
                        let response = json!({
                            "type": "response",
                            "request_seq": seq,
                            "success": true,
                            "command": "initialize",
                            "body": {
                                "supportsConfigurationDoneRequest": true,
                            }
                        });
                        send_message(&mut stdout, &response)?;

                        // Send initialized event
                        let event = json!({
                            "type": "event",
                            "event": "initialized",
                            "seq": 0
                        });
                        send_message(&mut stdout, &event)?;
                    } else if command == "launch" || command == "attach" {
                        // Send Error Response
                        let response = json!({
                            "type": "response",
                            "request_seq": seq,
                            "success": false,
                            "command": command,
                            "message": "DAP debugging is currently being migrated from the AST Interpreter to the Bytecode VM. Please use 'ifa run' for standard execution.",
                        });
                        send_message(&mut stdout, &response)?;

                        // Send Terminated Event
                        let event = json!({
                            "type": "event",
                            "event": "terminated",
                            "seq": 0
                        });
                        send_message(&mut stdout, &event)?;
                    } else if command == "disconnect"
                        || command == "terminate"
                        || command == "configurationDone"
                    {
                        let response = json!({
                            "type": "response",
                            "request_seq": seq,
                            "success": true,
                            "command": command
                        });
                        send_message(&mut stdout, &response)?;
                        if command == "disconnect" {
                            break;
                        }
                    } else {
                        // Acknowledge unknown requests to avoid hanging
                        let response = json!({
                            "type": "response",
                            "request_seq": seq,
                            "success": false,
                            "command": command,
                            "message": "Not supported"
                        });
                        send_message(&mut stdout, &response)?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn send_message(stdout: &mut io::Stdout, message: &Value) -> io::Result<()> {
    let payload = serde_json::to_string(message)?;
    write!(
        stdout,
        "Content-Length: {}\r\n\r\n{}",
        payload.len(),
        payload
    )?;
    stdout.flush()
}
