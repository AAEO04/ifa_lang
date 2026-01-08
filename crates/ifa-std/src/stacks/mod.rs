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

pub mod iot;
pub mod backend;
pub mod ml;
pub mod gamedev;
pub mod crypto;
pub mod frontend;

// IoT re-exports
pub use iot::{EmbeddedError, EmbeddedResult, GpioPin, PinMode, PinState};
pub use iot::{EmbeddedGpio, EmbeddedTimer, EmbeddedSerial, EmbeddedI2C, EmbeddedSPI};

// Backend re-exports
pub use backend::{HttpServer, Request, Response, OrmClient, Middleware};

// ML re-exports
pub use ml::{Tensor, TensorError, TensorResult, Linear, loss, SGD, Optimizer};

// Gamedev re-exports
pub use gamedev::{Vec2, AABB, Entity, Transform, Velocity, World, SpatialGrid};
pub use gamedev::{Animation, GameTimer, Input, Audio, Collider, SpriteComponent};

// Crypto re-exports
pub use crypto::{CryptoError, SecureRng, SecretStore, hash, base64, hex, constant_time_compare, uuid_v4};

// Frontend re-exports
pub use frontend::{Element, Node, Style, Router, Store, Fetch, LocalStorage, SafeHtml, escape_html};




