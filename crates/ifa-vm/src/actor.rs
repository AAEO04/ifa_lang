//! # H2 — Actor Runtime
//!
//! Each actor is a fully isolated `IfaVM` running on its own OS thread,
//! communicating exclusively via async message channels.
//!
//! ## Invariants
//! - Actor VMs **never share** globals, opon, or registry state.
//! - The only crossing-boundary values are `IfaValue` messages, which
//!   are already `Clone + Send` because they contain only `Arc`-wrapped
//!   or scalar data.
//! - `ActorHandle` is the only reference a caller holds. It is `Clone + Send`.
//! - Shutdown is cooperative: sending `ActorMsg::Shutdown` causes the actor
//!   loop to exit cleanly after processing in-flight messages.

use crate::error::{IfaError, IfaResult};
use ifa_types::value_union::IfaValue;
use std::collections::HashMap;
use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicU64, Ordering},
    mpsc::{self, SyncSender, TrySendError},
};

#[cfg(not(target_arch = "wasm32"))]
static ACTOR_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
fn get_actor_runtime() -> &'static tokio::runtime::Runtime {
    ACTOR_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .thread_name("ifa-actor-pool")
            .enable_all()
            .build()
            .expect("Failed to build actor tokio runtime")
    })
}

pub fn spawn_actor_task<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn_blocking(f);
        } else {
            get_actor_runtime().spawn_blocking(f);
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = f;
        panic!("spawn_actor_task is not supported on WASM");
    }
}

struct ActorIdGuard;

impl ActorIdGuard {
    fn new(id: u64) -> Self {
        ifa_types::set_current_actor_id(Some(id));
        ActorIdGuard
    }
}

impl Drop for ActorIdGuard {
    fn drop(&mut self) {
        ifa_types::set_current_actor_id(None);
    }
}

// ─── Message type ────────────────────────────────────────────────────────────

/// Messages that can be sent to an actor's inbox.
#[derive(Clone, Debug)]
pub enum ActorMsg {
    /// A value sent via `Osa.ran(handle, value)`.
    Value(IfaValue),
    /// Orderly shutdown request.
    Shutdown,
}

// Safety: `IfaValue` contains only `Arc`-wrapped heap data and scalars.
// The `#[cfg(feature = "vm")]` variants (Upvalue/Closure/Future) hold
// `Arc<Mutex<...>>` which is also `Send`.
unsafe impl Send for ActorMsg {}

// ─── Handle ──────────────────────────────────────────────────────────────────

/// A lightweight, cloneable reference to a running actor.
///
/// Cheaply cloneable — backed by `Arc`. Sending a message is a single
/// `SyncSender::try_send` call with no locks on the actor VM.
#[derive(Clone, Debug)]
pub struct ActorHandle {
    /// Unique actor identifier (monotonic u64).
    pub id: u64,
    /// Bounded channel transmit end. Bounded = back-pressure is free.
    tx: Arc<SyncSender<ActorMsg>>,
    /// The isolated resource registry for this actor.
    pub resource_registry: Arc<ifa_types::registry::ResourceRegistry>,
}

impl ActorHandle {
    /// Send a value to this actor's inbox. Non-blocking — returns an error
    /// if the channel is full or the actor has exited.
    pub fn send(&self, value: IfaValue) -> IfaResult<()> {
        self.tx
            .try_send(ActorMsg::Value(value))
            .map_err(|e| match e {
                TrySendError::Full(_) => IfaError::Runtime(format!(
                    "Actor {} inbox is full — apply back-pressure or increase buffer",
                    self.id
                )),
                TrySendError::Disconnected(_) => {
                    IfaError::Runtime(format!("Actor {} has exited", self.id))
                }
            })
    }

    /// Request the actor to shut down. Non-blocking.
    pub fn shutdown(&self) -> IfaResult<()> {
        self.tx
            .try_send(ActorMsg::Shutdown)
            .map_err(|_| IfaError::Runtime(format!("Actor {} already exited", self.id)))
    }
}

// ─── ID generator ────────────────────────────────────────────────────────────

static NEXT_ACTOR_ID: AtomicU64 = AtomicU64::new(1);

fn next_actor_id() -> u64 {
    NEXT_ACTOR_ID.fetch_add(1, Ordering::Relaxed)
}

// ─── Table ───────────────────────────────────────────────────────────────────

/// Process-wide registry of live actor handles.
///
/// Stored on `ResourceRegistry` so it survives VM resets. Actors
/// deregister themselves on shutdown.
#[derive(Debug, Default)]
pub struct ActorTable {
    inner: Mutex<HashMap<u64, ActorHandle>>,
}

impl ActorTable {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Register a new actor and return its handle.
    // SAFETY [Poisoning]: ActorTable mutations are simple HashMap insertions and removals.
    // If a thread panics while holding the lock, the HashMap itself is not structurally corrupted.
    // We use `unwrap_or_else(|e| e.into_inner())` to safely recover the lock and prevent the entire VM registry from freezing.
    pub fn insert(&self, handle: ActorHandle) {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(handle.id, handle);
    }

    /// Remove a dead actor.
    pub fn remove(&self, id: u64) {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&id);
    }

    /// Look up a handle by ID.
    pub fn get(&self, id: u64) -> Option<ActorHandle> {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&id)
            .cloned()
    }
}

// ─── Spawn ───────────────────────────────────────────────────────────────────

/// Inbox capacity per actor. Back-pressure kicks in when the sender
/// outpaces the receiver by more than this many messages.
const ACTOR_INBOX_CAPACITY: usize = 64;

/// H2 actor spawn. Creates a fresh, fully-isolated VM on an OS thread.
///
/// # Returns
/// An `IfaValue::Actor` the caller can store and pass to `Osa.ran`.
pub fn spawn_actor(
    init_fn: IfaValue,
    bytecode: Arc<crate::bytecode::Bytecode>,
    table: Arc<ActorTable>,
    registry: Option<Box<dyn crate::native::OduRegistry>>,
    resource_registry: std::sync::Arc<ifa_types::registry::ResourceRegistry>,
) -> IfaResult<IfaValue> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let id = next_actor_id();
        let (tx, rx) = mpsc::sync_channel::<ActorMsg>(ACTOR_INBOX_CAPACITY);
        let tx = Arc::new(tx);
        let handle = ActorHandle {
            id,
            tx: tx.clone(),
            resource_registry: resource_registry.clone(),
        };
        table.insert(handle.clone());

        let table_clone = table.clone();
        spawn_actor_task(move || {
            actor_loop(
                id,
                init_fn,
                rx,
                &bytecode,
                &table_clone,
                registry,
                resource_registry,
            );
        });

        // Wrap the full handle as a type-erased Arc so IfaValue::Actor can hold it
        // without ifa-types depending on ifa-vm.
        let erased: Arc<dyn std::any::Any + Send + Sync> = Arc::new(handle);
        Ok(IfaValue::Actor { id, handle: erased })
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (init_fn, bytecode, table, registry, resource_registry);
        Err(IfaError::Runtime(
            "Actors (multi-threading) are not supported on WASM targets.".into(),
        ))
    }
}

/// Scans a value recursively and transfers ownership of any `ResourceToken` to the recipient actor ID.
pub fn transfer_resources(
    value: &IfaValue,
    sender_registry: &ifa_types::ResourceRegistry,
    recipient_registry: &ifa_types::ResourceRegistry,
    recipient_actor_id: u64,
) {
    match value {
        IfaValue::Resource(token) => {
            if let Some(res) = sender_registry.take(**token) {
                recipient_registry.insert_raw(**token, res, recipient_actor_id);
            }
        }
        IfaValue::List(l) => {
            for v in l.iter() {
                transfer_resources(v, sender_registry, recipient_registry, recipient_actor_id);
            }
        }
        IfaValue::Map(m) => {
            for v in m.values() {
                transfer_resources(v, sender_registry, recipient_registry, recipient_actor_id);
            }
        }
        IfaValue::Result(payload) => match payload.as_ref() {
            ifa_types::value_union::ResultPayload::Ok(v) => {
                transfer_resources(v, sender_registry, recipient_registry, recipient_actor_id)
            }
            ifa_types::value_union::ResultPayload::Err(v) => {
                transfer_resources(v, sender_registry, recipient_registry, recipient_actor_id)
            }
        },
        _ => {}
    }
}

/// Send a message to an actor represented as `IfaValue::Actor`.
pub fn actor_send(
    actor: &IfaValue,
    value: IfaValue,
    sender_registry: &ifa_types::ResourceRegistry,
) -> IfaResult<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let IfaValue::Actor { id, handle } = actor {
            let actor_handle = handle
                .downcast_ref::<Arc<ActorHandle>>()
                .ok_or_else(|| IfaError::Runtime(format!("Actor {}: invalid handle type", id)))?;

            // 1. Enforce No-Shared-Mutability via Zero-Copy Ownership Transfer.
            // Babalawo statically guarantees the sender cannot access this value again.
            let safe_value = value.clone();

            // 2. Transfer ownership of any resources contained in the message payload
            transfer_resources(
                &safe_value,
                sender_registry,
                &actor_handle.resource_registry,
                *id,
            );

            actor_handle
                .tx
                .try_send(ActorMsg::Value(safe_value))
                .map_err(|e| match e {
                    mpsc::TrySendError::Full(_) => IfaError::Runtime(format!(
                        "Actor {} inbox full — back-pressure required",
                        id
                    )),
                    mpsc::TrySendError::Disconnected(_) => {
                        IfaError::Runtime(format!("Actor {} has exited", id))
                    }
                })
        } else {
            Err(IfaError::TypeError {
                expected: "Actor".into(),
                got: actor.type_name().into(),
            })
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (actor, value, sender_registry);
        Err(IfaError::Runtime(
            "Actors (multi-threading) are not supported on WASM targets.".into(),
        ))
    }
}

/// The actor's main loop. Runs on a dedicated OS thread.
///
/// Receives messages from its inbox, calls `handler(message)` for each
/// `Value` message, stops on `Shutdown` or channel close.
fn actor_loop(
    id: u64,
    handler: IfaValue,
    rx: mpsc::Receiver<ActorMsg>,
    bytecode: &crate::bytecode::Bytecode,
    table: &Arc<ActorTable>,
    registry: Option<Box<dyn crate::native::OduRegistry>>,
    resource_registry: std::sync::Arc<ifa_types::registry::ResourceRegistry>,
) {
    use crate::vm::IfaVM;

    // Use guard to set and automatically clear the thread-local actor ID on exit/panic
    let _guard = ActorIdGuard::new(id);

    // Fresh, isolated VM — zero shared state with the parent.
    let mut vm = IfaVM::new();
    vm.actor_id = Some(id);
    vm.registry = registry;
    vm.resource_registry = resource_registry.into();

    while let Ok(msg) = rx.recv() {
        match msg {
            ActorMsg::Shutdown => break,
            ActorMsg::Value(value) => {
                let args = vec![value];
                // spawn_task creates a cooperative task and returns a Future.
                match vm.spawn_task(handler.clone(), args) {
                    Ok(IfaValue::Future(cell)) => {
                        // Drive the task to completion. Since there's only one
                        // task in this actor's queue, this will run it fully.
                        if let Err(e) = vm.await_future(&cell, bytecode) {
                            eprintln!("[ifa actor {}] handler error: {}", id, e);
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("[ifa actor {}] spawn error: {}", id, e);
                    }
                }
            }
        }
    }

    table.remove(id);
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actor_ids_are_unique() {
        let a = next_actor_id();
        let b = next_actor_id();
        assert_ne!(a, b);
        assert!(b > a);
    }

    #[test]
    fn actor_table_insert_get_remove() {
        let table = ActorTable::new();
        let (tx, _rx) = mpsc::sync_channel::<ActorMsg>(1);
        let registry = Arc::new(ifa_types::registry::ResourceRegistry::new());
        let handle = ActorHandle {
            id: 999,
            tx: Arc::new(tx),
            resource_registry: registry,
        };
        table.insert(handle.clone());

        let got = table.get(999);
        assert!(got.is_some());
        assert_eq!(got.unwrap().id, 999);

        table.remove(999);
        assert!(table.get(999).is_none());
    }

    #[test]
    fn send_to_disconnected_actor_errors() {
        let (tx, _rx) = mpsc::sync_channel::<ActorMsg>(1);
        // Drop rx so the channel is disconnected.
        drop(_rx);
        let registry = Arc::new(ifa_types::registry::ResourceRegistry::new());
        let handle = ActorHandle {
            id: 1,
            tx: Arc::new(tx),
            resource_registry: registry,
        };
        let result = handle.send(IfaValue::null());
        assert!(result.is_err());
    }
}
