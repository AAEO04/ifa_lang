//! # Priority Stacks
//!
//! Feature-gated extensions for specific domains.
//!
//! Each stack provides specialized functionality:
//! - **IoT**: Embedded systems, GPIO, sensors
//! - **Backend**: HTTP servers, databases
//! - **ML**: Tensors, neural networks
//! - **Gamedev**: ECS, physics, sprites
//! - **Crypto**: Hashing, encoding, secrets
//! - **Frontend**: Virtual DOM, routing, state

#[cfg(feature = "backend")]
pub mod backend;
#[cfg(feature = "crypto")]
pub mod crypto;
#[cfg(feature = "frontend")]
pub mod frontend;
#[cfg(feature = "game")]
pub mod gamedev;
#[cfg(feature = "iot")]
pub mod iot;
#[cfg(feature = "ml")]
pub mod ml;
#[cfg(any(feature = "fullstack", feature = "fusion", feature = "backend"))]
pub mod fusion;

// IoT re-exports
#[cfg(feature = "iot")]
pub use iot::{EmbeddedError, EmbeddedResult, GpioPin, PinMode, PinState};
#[cfg(feature = "iot")]
pub use iot::{EmbeddedGpio, EmbeddedI2C, EmbeddedSPI, EmbeddedSerial, EmbeddedTimer};

// Backend re-exports
#[cfg(feature = "backend")]
pub use backend::{HttpServer, Middleware, OrmClient, Request, Response};

// ML re-exports
#[cfg(feature = "ml")]
pub use ml::{Linear, Optimizer, SGD, Tensor, TensorError, TensorResult, loss};

// Gamedev re-exports
#[cfg(feature = "game")]
pub use gamedev::{AABB, Entity, SpatialGrid, Transform, Vec2, Velocity, World};
#[cfg(feature = "game")]
pub use gamedev::{Animation, Audio, Collider, GameTimer, Input, SpriteComponent};

// Crypto re-exports
#[cfg(feature = "crypto")]
pub use crypto::{
    CryptoError, SecretStore, SecureRng, base64, constant_time_compare, hash, hex, uuid_v4,
};

// Frontend re-exports
#[cfg(feature = "frontend")]
pub use frontend::{
    Element, Fetch, LocalStorage, Node, Router, SafeHtml, Store, Style, escape_html,
};

// Fusion re-exports
#[cfg(any(feature = "fullstack", feature = "fusion", feature = "backend"))]
pub use fusion::{FusionRuntime, FusionContext, FusionRole, IpcMessage};
