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

pub mod backend;
pub mod crypto;
pub mod frontend;
pub mod gamedev;
pub mod iot;
pub mod ml;

// IoT re-exports
pub use iot::{EmbeddedError, EmbeddedResult, GpioPin, PinMode, PinState};
pub use iot::{EmbeddedGpio, EmbeddedI2C, EmbeddedSPI, EmbeddedSerial, EmbeddedTimer};

// Backend re-exports
pub use backend::{HttpServer, Middleware, OrmClient, Request, Response};

// ML re-exports
pub use ml::{Linear, Optimizer, SGD, Tensor, TensorError, TensorResult, loss};

// Gamedev re-exports
pub use gamedev::{AABB, Entity, SpatialGrid, Transform, Vec2, Velocity, World};
pub use gamedev::{Animation, Audio, Collider, GameTimer, Input, SpriteComponent};

// Crypto re-exports
pub use crypto::{
    CryptoError, SecretStore, SecureRng, base64, constant_time_compare, hash, hex, uuid_v4,
};

// Frontend re-exports
pub use frontend::{
    Element, Fetch, LocalStorage, Node, Router, SafeHtml, Store, Style, escape_html,
};
