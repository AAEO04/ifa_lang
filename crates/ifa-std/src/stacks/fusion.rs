//! # Fusion Stack (The Distributed Monolith)
//!
//! Runtime support for Hybrid Executables (Fullstack).
//! Orchestrates the Backend (Thread 1) and Frontend (Thread 2 / WebView)
//! and provides secure IPC.
//!
//! "The network is the computer... but the computer is one binary."

use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
// Unused import removed
// use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

/// IPC Message Structure (JSON-RPC 2.0 style)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: Option<u64>,
}

impl IpcMessage {
    pub fn new(method: &str, params: serde_json::Value, id: Option<u64>) -> Self {
        IpcMessage {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id,
        }
    }
}

/// The Fusion Runtime
pub struct FusionRuntime;

impl Default for FusionRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl FusionRuntime {
    pub fn new() -> Self {
        FusionRuntime
    }
    
    /// Launch the Hybrid Application
    /// 
    /// Takes two closures:
    /// - `backend_fn`: The server-side logic
    /// - `frontend_fn`: The client-side logic (WebView controller)
    pub fn launch<B, F>(self, backend_fn: B, frontend_fn: F)
    where
        B: FnOnce(FusionContext) + Send + 'static,
        F: FnOnce(FusionContext) + Send + 'static,
    {
        println!("🚀 Launching Fusion Runtime...");

        // Setup Contexts
        // Backend needs to: Read from Frontend (brx), Write to Frontend (ftx)
        // We establish fresh channels for this session.
        
        // Re-creating channels to pass ownership properly
        let (to_backend_tx, to_backend_rx) = channel::<IpcMessage>();
        let (to_frontend_tx, to_frontend_rx) = channel::<IpcMessage>();

        let backend_ctx = FusionContext {
            role: FusionRole::Backend,
            tx: to_frontend_tx,
            rx: to_backend_rx,
        };

        let frontend_ctx = FusionContext {
            role: FusionRole::Frontend,
            tx: to_backend_tx,
            rx: to_frontend_rx,
        };

        // Spawn Backend Thread
        let backend_handle = thread::spawn(move || {
            println!("[Fusion] Backend Thread Started");
            backend_fn(backend_ctx);
        });

        // Run Frontend on Main Thread (usually required for WebView/UI)
        println!("[Fusion] Frontend Thread Started (Main)");
        frontend_fn(frontend_ctx);

        // Wait for backend (if frontend exits, backend should probably terminate)
        let _ = backend_handle.join();
        println!("[Fusion] Shutdown");
    }
}

/// Role of the current context
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FusionRole {
    Backend,
    Frontend,
}

/// Context passed to each domain
pub struct FusionContext {
    pub role: FusionRole,
    tx: Sender<IpcMessage>,
    rx: Receiver<IpcMessage>,
}

impl FusionContext {
    /// Send message to the other side
    pub fn send(&self, method: &str, params: serde_json::Value) -> Result<(), String> {
        let msg = IpcMessage::new(method, params, None); 
        self.tx.send(msg).map_err(|e| e.to_string())
    }

    /// Receive next message (blocking)
    pub fn recv(&self) -> Result<IpcMessage, String> {
        self.rx.recv().map_err(|e| e.to_string())
    }
    
    // Receive with timeout would need crossbeam-channel, keeping it simple for std
}
