// Shared modules (always available)
pub mod generations;
pub mod mutations;
pub mod scoring;
pub mod types;

// CLI-only modules (use imageproc)
#[cfg(feature = "cli")]
pub mod algorithms;
#[cfg(feature = "cli")]
pub mod renderer;

// WASM-only modules
#[cfg(feature = "wasm")]
pub mod algorithms_wasm;
#[cfg(feature = "wasm")]
pub mod renderer_wasm;
#[cfg(feature = "wasm")]
pub mod wasm_bindings;

// Re-export WASM bindings for wasm-pack
#[cfg(feature = "wasm")]
pub use wasm_bindings::*;
