use wasm_bindgen::prelude::*;

use crate::algorithms_wasm::{AlgorithmConfig, AlgorithmState, AlgorithmType};

/// Initialize panic hook for better error messages
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// WASM-exposed algorithm wrapper
#[wasm_bindgen]
pub struct WasmAlgorithm {
    state: AlgorithmState,
}

#[wasm_bindgen]
impl WasmAlgorithm {
    /// Create a new algorithm instance with target image data
    /// algorithm: 0 = Evolution Strategy, 1 = Simulated Annealing, 2 = Differential Evolution
    #[wasm_bindgen(constructor)]
    pub fn new(
        target_pixels: Vec<u8>,
        width: usize,
        height: usize,
        max_iterations: usize,
        target_accuracy: f32,
        algorithm: u8,
    ) -> WasmAlgorithm {
        let algorithm_type = match algorithm {
            1 => AlgorithmType::SimulatedAnnealing,
            2 => AlgorithmType::DifferentialEvolution,
            _ => AlgorithmType::EvolutionStrategy,
        };

        let config = AlgorithmConfig {
            max_iterations,
            target_accuracy,
            algorithm_type,
            ..Default::default()
        };

        WasmAlgorithm {
            state: AlgorithmState::new(target_pixels, width, height, config),
        }
    }

    /// Run a batch of iterations and return the current best image as RGBA bytes
    #[wasm_bindgen]
    pub fn step(&mut self, batch_size: usize) -> Vec<u8> {
        let (_, buffer) = self.state.step_batch(batch_size);
        buffer.data
    }

    /// Get current iteration count
    #[wasm_bindgen]
    pub fn get_iteration(&self) -> usize {
        self.state.get_iteration()
    }

    /// Get current best accuracy (0.0 to 1.0)
    #[wasm_bindgen]
    pub fn get_accuracy(&self) -> f32 {
        self.state.get_accuracy()
    }

    /// Check if algorithm has finished (reached max iterations or target accuracy)
    #[wasm_bindgen]
    pub fn is_finished(&self) -> bool {
        self.state.is_finished()
    }

    /// Get image width
    #[wasm_bindgen]
    pub fn get_width(&self) -> usize {
        self.state.get_dimensions().0
    }

    /// Get image height
    #[wasm_bindgen]
    pub fn get_height(&self) -> usize {
        self.state.get_dimensions().1
    }
}
