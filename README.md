# Approximate Image Generation

A Rust application that approximates images using evolutionary algorithms and semi-transparent polygons. The program evolves a collection of polygons to recreate a target image, producing stylized abstract representations.

## How It Works

The application represents images as a collection of semi-transparent polygons. Starting from random polygons, it uses evolutionary optimization to iteratively improve the approximation by:

1. Generating mutations of current solutions (color changes, vertex adjustments, polygon additions/removals)
2. Scoring each candidate against the target image using pixel comparison
3. Selecting the best candidates to continue evolving

## Algorithms

Three optimization algorithms are available:

### Evolution Strategy (ES)

The default algorithm maintains a small population of solutions. Each generation:
- Generates mutated children from all parents
- Keeps the top 2 performers unchanged
- Retains the worst performer with heavy mutations to maintain diversity

### Simulated Annealing (SA)

A single-solution approach that accepts worse solutions with decreasing probability over time, allowing escape from local optima.

### Differential Evolution (DE)

A population-based algorithm that creates new candidates by combining differences between existing solutions, effective for continuous optimization problems.

## Building

### Prerequisites

- Rust toolchain (1.70+)
- For WASM: `wasm-pack`

### Native CLI

```bash
cargo build --release --features cli
```

### WebAssembly

```bash
wasm-pack build --target web --features wasm
```

## Usage

### Command Line

Place your target image in the `resources/` directory and update the path in `main.rs`:

```bash
cargo run --release --features cli
```

Output is saved to `resources/output.png`.

### Web Interface

1. Build the WASM module
2. Serve the `web/` directory with a local web server
3. Open in browser, upload an image, select algorithm and parameters, and start

```bash
# Example using Python's built-in server
python -m http.server 8080 --directory web
```

## Configuration

Key parameters (adjustable in code or web interface):

| Parameter | Description | Default |
|-----------|-------------|---------|
| Max Iterations | Maximum optimization steps | 100,000 |
| Target Accuracy | Stop when this similarity is reached | 0.95 |
| Children per Parent | Mutations generated per parent (ES) | 10 |
| Initial Temperature | Starting temperature (SA) | 1.0 |
| Cooling Rate | Temperature decay rate (SA) | 0.99995 |
| Population Size | Number of solutions (DE) | 6 |
| Mutation Factor | Differential weight (DE) | 0.8 |
| Crossover Rate | Recombination probability (DE) | 0.9 |

## Project Structure

```
src/
  algorithms.rs      # Native ES implementation
  algorithms_wasm.rs # WASM-compatible algorithms (ES, SA, DE)
  generations.rs     # Initial population generation
  mutations.rs       # Polygon mutation operations
  renderer.rs        # Native image rendering
  renderer_wasm.rs   # WASM image rendering
  scoring.rs         # Image comparison functions
  types.rs           # Core data structures
  wasm_bindings.js   # JavaScript bindings
web/
  index.html         # Web interface
  main.js            # Frontend logic
  style.css          # Styling
  worker.js          # Web Worker for background processing
resources/
  *.png              # Sample target images
```

