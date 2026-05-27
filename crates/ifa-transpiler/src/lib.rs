#![allow(
    clippy::manual_contains,
    clippy::useless_format
)]
pub mod project_generator;
pub mod transpiler;

pub use project_generator::{ProjectConfig, generate_project, generate_project_with_types};
pub use transpiler::core::RustTranspiler;
pub use transpiler::transpile_to_rust;
