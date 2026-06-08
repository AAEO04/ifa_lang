//! Debug Adapter Protocol Implementation for Ifá-Lang Bytecode VM.
use serde_json::{Value, json};
use std::io::{self, BufRead, Read, Write};

use ifa_types::Bytecode;
use ifa_vm::IfaVM;
use std::collections::HashMap;

struct DebugServer {
    vm: Option<IfaVM>,
    bytecode: Option<Bytecode>,
    breakpoints: HashMap<String, Vec<usize>>,
    step_mode: StepMode,
    target_depth: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StepMode {
    None,
    StepIn,
    StepOver,
    StepOut,
    Continue,
}

pub fn run_debug_session(file: std::path::PathBuf) -> color_eyre::Result<()> {
    eprintln!("Starting Ifá-Lang Bytecode DAP Server...");

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut stdout = io::stdout();

    let mut server = DebugServer {
        vm: None,
        bytecode: None,
        breakpoints: HashMap::new(),
        step_mode: StepMode::None,
        target_depth: 0,
    };

    // We pre-load the file
    let source_code = std::fs::read_to_string(&file)?;
    let program = ifa_parser::parse(&source_code)
        .map_err(|e| color_eyre::eyre::eyre!("Parse error: {}", e))?;

    // Compile
    let compiler = ifa_vm::Compiler::new(&file.display().to_string());
    let bytecode = compiler
        .compile(&program)
        .map_err(|e| color_eyre::eyre::eyre!("Compile error: {}", e))?;

    let mut registry = ifa_std::vm_registry::StdRegistry::new();
    let mut caps = ifa_sandbox::CapabilitySet::new();
    caps.grant(ifa_sandbox::Ofun::Stdio);

    registry.set_capabilities(caps);

    let mut vm = IfaVM::new().with_registry(Box::new(registry));

    // Initialise VM (simulate execute startup)
    vm.ctx.ip = 0;
    vm.ctx.halted = false;
    let _ = vm.ikin.load_from_bytecode(&bytecode);

    let (stack_cap, _frame_cap) = bytecode.opon_size.limits();
    if let Some(cap) = stack_cap {
        vm.ctx.stack.reserve(cap);
    }

    server.vm = Some(vm);
    server.bytecode = Some(bytecode);

    loop {
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
                        if parts.len() == 2
                            && let Ok(len) = parts[1].trim().parse::<usize>()
                        {
                            content_length = len;
                        }
                    }
                }
                Err(_) => return Ok(()),
            }
        }

        if content_length == 0 {
            continue;
        }

        let mut body = vec![0; content_length];
        if reader.read_exact(&mut body).is_err() {
            return Ok(());
        }

        let request: Value = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(type_val) = request.get("type")
            && type_val == "request"
            && let Some(command) = request.get("command").and_then(|v| v.as_str())
        {
            let seq = request.get("seq").unwrap_or(&json!(0)).clone();
            handle_request(command, seq, &request, &mut stdout, &mut server)?;
        }
    }
}

fn handle_request(
    command: &str,
    seq: Value,
    request: &Value,
    stdout: &mut io::Stdout,
    server: &mut DebugServer,
) -> color_eyre::Result<()> {
    match command {
        "initialize" => {
            let response = json!({
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": "initialize",
                "body": {
                    "supportsConfigurationDoneRequest": true,
                }
            });
            send_message(stdout, &response)?;
            let event = json!({ "type": "event", "event": "initialized", "seq": 0 });
            send_message(stdout, &event)?;
        }
        "launch" | "attach" => {
            let response = json!({ "type": "response", "request_seq": seq, "success": true, "command": command });
            send_message(stdout, &response)?;
        }
        "setBreakpoints" => {
            if let Some(args) = request.get("arguments")
                && let Some(source) = args
                    .get("source")
                    .and_then(|s| s.get("path"))
                    .and_then(|p| p.as_str())
            {
                let mut bps = Vec::new();
                if let Some(arr) = args.get("breakpoints").and_then(|b| b.as_array()) {
                    for bp in arr {
                        if let Some(line) = bp.get("line").and_then(|l| l.as_u64()) {
                            bps.push(line as usize);
                        }
                    }
                }
                server.breakpoints.insert(source.to_string(), bps.clone());

                let mut bp_responses = Vec::new();
                for bp in bps {
                    bp_responses.push(json!({"verified": true, "line": bp}));
                }
                let response = json!({
                    "type": "response",
                    "request_seq": seq,
                    "success": true,
                    "command": "setBreakpoints",
                    "body": { "breakpoints": bp_responses }
                });
                send_message(stdout, &response)?;
                return Ok(());
            }
            let response = json!({ "type": "response", "request_seq": seq, "success": true, "command": "setBreakpoints" });
            send_message(stdout, &response)?;
        }
        "configurationDone" => {
            let response = json!({ "type": "response", "request_seq": seq, "success": true, "command": "configurationDone" });
            send_message(stdout, &response)?;

            // Start execution
            server.step_mode = StepMode::Continue;
            run_vm(server, stdout)?;
        }
        "threads" => {
            let response = json!({
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": "threads",
                "body": {
                    "threads": [ { "id": 1, "name": "Main Thread" } ]
                }
            });
            send_message(stdout, &response)?;
        }
        "stackTrace" => {
            let mut frames_json = Vec::new();
            if let Some(vm) = &server.vm {
                frames_json.push(json!({
                    "id": vm.ctx.frames.len(),
                    "name": "Current",
                    "line": 1,
                    "column": 0,
                    "source": { "name": "source", "path": "file" }
                }));

                for (i, _frame) in vm.ctx.frames.iter().enumerate().rev() {
                    frames_json.push(json!({
                        "id": i,
                        "name": format!("Frame {}", i),
                        "line": 1,
                        "column": 0
                    }));
                }
            }
            let response = json!({
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": "stackTrace",
                "body": { "stackFrames": frames_json, "totalFrames": frames_json.len() }
            });
            send_message(stdout, &response)?;
        }
        "scopes" => {
            let response = json!({
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": "scopes",
                "body": {
                    "scopes": [
                        { "name": "Locals", "variablesReference": 1, "expensive": false },
                        { "name": "Globals", "variablesReference": 2, "expensive": false }
                    ]
                }
            });
            send_message(stdout, &response)?;
        }
        "variables" => {
            let mut vars_json = Vec::new();
            if let Some(vm) = &server.vm {
                let ref_id = request
                    .get("arguments")
                    .and_then(|a| a.get("variablesReference"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                if ref_id == 1 {
                    // Locals
                    // Output stack summary
                    vars_json.push(json!({
                        "name": "stack_depth",
                        "value": format!("{}", vm.ctx.stack.len()),
                        "variablesReference": 0
                    }));
                } else if ref_id == 2 {
                    // Globals
                    vars_json.push(json!({
                        "name": "globals",
                        "value": "Unavailable",
                        "variablesReference": 0
                    }));
                }
            }
            let response = json!({
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": "variables",
                "body": { "variables": vars_json }
            });
            send_message(stdout, &response)?;
        }
        "continue" => {
            let response = json!({ "type": "response", "request_seq": seq, "success": true, "command": "continue" });
            send_message(stdout, &response)?;
            server.step_mode = StepMode::Continue;
            run_vm(server, stdout)?;
        }
        "next" => {
            let response = json!({ "type": "response", "request_seq": seq, "success": true, "command": "next" });
            send_message(stdout, &response)?;
            if let Some(vm) = &server.vm {
                server.target_depth = vm.ctx.frames.len();
            }
            server.step_mode = StepMode::StepOver;
            run_vm(server, stdout)?;
        }
        "stepIn" => {
            let response = json!({ "type": "response", "request_seq": seq, "success": true, "command": "stepIn" });
            send_message(stdout, &response)?;
            server.step_mode = StepMode::StepIn;
            run_vm(server, stdout)?;
        }
        "stepOut" => {
            let response = json!({ "type": "response", "request_seq": seq, "success": true, "command": "stepOut" });
            send_message(stdout, &response)?;
            if let Some(vm) = &server.vm {
                if vm.ctx.frames.is_empty() {
                    server.target_depth = 0;
                } else {
                    server.target_depth = vm.ctx.frames.len() - 1;
                }
            }
            server.step_mode = StepMode::StepOut;
            run_vm(server, stdout)?;
        }
        "disconnect" | "terminate" => {
            let response = json!({ "type": "response", "request_seq": seq, "success": true, "command": command });
            send_message(stdout, &response)?;
            std::process::exit(0);
        }
        _ => {
            let response = json!({ "type": "response", "request_seq": seq, "success": false, "command": command, "message": "Unknown command" });
            send_message(stdout, &response)?;
        }
    }
    Ok(())
}

fn run_vm(server: &mut DebugServer, stdout: &mut io::Stdout) -> color_eyre::Result<()> {
    let vm = match server.vm.as_mut() {
        Some(v) => v,
        None => return Ok(()),
    };
    let bytecode = match server.bytecode.as_ref() {
        Some(b) => b,
        None => return Ok(()),
    };

    let start_ip = vm.ctx.ip;

    loop {
        if vm.ctx.halted || vm.ctx.ip >= bytecode.code.len() {
            let event = json!({ "type": "event", "event": "terminated", "seq": 0 });
            send_message(stdout, &event)?;
            return Ok(());
        }

        // Stepping logic
        match server.step_mode {
            StepMode::StepIn => {
                if vm.ctx.ip != start_ip {
                    server.step_mode = StepMode::None;
                    let event = json!({ "type": "event", "event": "stopped", "body": { "reason": "step", "threadId": 1 } });
                    send_message(stdout, &event)?;
                    return Ok(());
                }
            }
            StepMode::StepOver => {
                if vm.ctx.ip != start_ip && vm.ctx.frames.len() <= server.target_depth {
                    server.step_mode = StepMode::None;
                    let event = json!({ "type": "event", "event": "stopped", "body": { "reason": "step", "threadId": 1 } });
                    send_message(stdout, &event)?;
                    return Ok(());
                }
            }
            StepMode::StepOut => {
                if vm.ctx.frames.len() <= server.target_depth {
                    server.step_mode = StepMode::None;
                    let event = json!({ "type": "event", "event": "stopped", "body": { "reason": "step", "threadId": 1 } });
                    send_message(stdout, &event)?;
                    return Ok(());
                }
            }
            _ => {}
        }

        // Step the VM
        if let Err(e) = vm.step(bytecode) {
            let error_msg = format!("Runtime Error: {:?}", e);
            let event = json!({ "type": "event", "event": "output", "body": { "category": "stderr", "output": error_msg } });
            send_message(stdout, &event)?;

            let event = json!({ "type": "event", "event": "terminated", "seq": 0 });
            send_message(stdout, &event)?;
            return Ok(());
        }
    }
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
